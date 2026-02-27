use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use crate::error::AppError;

struct TokenBucket {
    tokens: u32,
    last_refill: Instant,
}

pub struct RateLimiter {
    requests_per_minute: u32,
    buckets: RwLock<HashMap<String, TokenBucket>>,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            buckets: RwLock::new(HashMap::new()),
        }
    }

    /// Check if a request is allowed for the given key.
    /// Uses the current time.
    pub fn check(&self, key: &str) -> Result<(), AppError> {
        self.check_with_time(key, Instant::now())
    }

    /// Check if a request is allowed for the given key, using a controlled time.
    /// This allows tests to inject a specific instant for window reset testing.
    pub fn check_with_time(&self, key: &str, now: Instant) -> Result<(), AppError> {
        let mut buckets = self.buckets.write().map_err(|e| {
            AppError::Internal(format!("Rate limiter lock poisoned: {}", e))
        })?;

        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket {
                tokens: self.requests_per_minute,
                last_refill: now,
            });

        // Refill tokens if >= 60 seconds have elapsed since last refill
        if now.duration_since(bucket.last_refill) >= Duration::from_secs(60) {
            bucket.tokens = self.requests_per_minute;
            bucket.last_refill = now;
        }

        if bucket.tokens > 0 {
            bucket.tokens -= 1;
            Ok(())
        } else {
            Err(AppError::RateLimited)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(60);
        let now = Instant::now();

        for i in 0..30 {
            assert!(
                limiter.check_with_time("agent-1", now).is_ok(),
                "Request {} should be allowed within limit",
                i + 1
            );
        }
    }

    #[test]
    fn test_rate_limiter_blocks_at_limit() {
        let limiter = RateLimiter::new(60);
        let now = Instant::now();

        // First 60 requests should all pass
        for i in 0..60 {
            assert!(
                limiter.check_with_time("agent-1", now).is_ok(),
                "Request {} should be allowed",
                i + 1
            );
        }

        // 61st request should be blocked
        let result = limiter.check_with_time("agent-1", now);
        assert!(result.is_err(), "61st request should be blocked");
        match result.unwrap_err() {
            AppError::RateLimited => {}
            other => panic!("Expected RateLimited error, got: {:?}", other),
        }
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(60);
        let now = Instant::now();

        // First 60 should pass
        for i in 0..60 {
            assert!(
                limiter.check_with_time("agent-1", now).is_ok(),
                "Request {} should be allowed",
                i + 1
            );
        }

        // Requests 61-80 should all return RateLimited
        for i in 61..=80 {
            let result = limiter.check_with_time("agent-1", now);
            assert!(result.is_err(), "Request {} should be blocked", i);
            match result.unwrap_err() {
                AppError::RateLimited => {}
                other => panic!("Expected RateLimited for request {}, got: {:?}", i, other),
            }
        }
    }

    #[test]
    fn test_rate_limiter_window_reset() {
        let limiter = RateLimiter::new(60);
        let now = Instant::now();

        // Exhaust all 60 tokens
        for _ in 0..60 {
            limiter.check_with_time("agent-1", now).unwrap();
        }

        // Confirm blocked
        assert!(
            limiter.check_with_time("agent-1", now).is_err(),
            "Should be blocked after exhausting limit"
        );

        // Advance clock by 60 seconds
        let later = now + Duration::from_secs(60);

        // New request should pass after window reset
        assert!(
            limiter.check_with_time("agent-1", later).is_ok(),
            "Should be allowed after window reset"
        );
    }

    #[test]
    fn test_rate_limiter_per_agent_isolation() {
        let limiter = RateLimiter::new(60);
        let now = Instant::now();

        // Agent A makes 50 requests
        for i in 0..50 {
            assert!(
                limiter.check_with_time("agent-a", now).is_ok(),
                "Agent A request {} should be allowed",
                i + 1
            );
        }

        // Agent B makes 50 requests
        for i in 0..50 {
            assert!(
                limiter.check_with_time("agent-b", now).is_ok(),
                "Agent B request {} should be allowed",
                i + 1
            );
        }
    }
}
