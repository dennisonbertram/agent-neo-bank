# Phase 1b: Agent Operations Architecture

**Goal:** Agent creation, spending policy, REST API, rate limiting, transaction history.

## TDD Requirement

Tests for this phase's components must be written FIRST before implementation. Write failing tests for spending policy engine, global policy engine, transaction processor, agent registry, invitation system, REST API endpoints, and rate limiter before writing any implementation code. See `docs/architecture/testing-specification.md` Sections 3.3-3.9 and 3.12 for all test cases.

## Implementation Tasks

| Task | Module | Details |
|---|---|---|
| Invitation code generation | `core/invitation.rs`, `commands/invitations.rs` | User generates codes in UI |
| Agent self-registration | `api/rest_routes`, `core/agent_registry` | POST /register with invitation code + rich metadata |
| Token delivery (encrypted cache) | `core/agent_registry` | 5-minute encrypted cache, poll-once-then-delete |
| Spending policy engine | `core::spending_policy` | Full validation logic, `BEGIN EXCLUSIVE` transactions |
| Global policy engine | `core::global_policy` | Global caps, min reserve, kill switch |
| Transaction processor (async) | `core::tx_processor` | Always 202 Accepted, async execution, webhook callback |
| Axum REST API | `api/rest_server` | `/v1/send`, `/v1/balance`, `/v1/health`, `/v1/transactions/{tx_id}` |
| Bearer token auth middleware | `api/auth_middleware` | Two-tier validation (SHA-256 cache + argon2 fallback) |
| Rate limiting | `api/rate_limiter.rs` | Invitation-code-based for registration, token bucket for API |
| Balance caching | `core::wallet_service` | 30s TTL, one CLI call per period |
| Agent balance visibility | `core::wallet_service` | Per-agent `balance_visible` flag |
| Amount parsing (Decimal) | `api/types.rs` | `SendRequest` uses `amount: Decimal`, parsed at API boundary |
| Transaction history UI | `pages/Transactions` | Table with basic filtering + pagination (limit/offset) |
| Stale approval cleanup | `core/approval_manager` | Background task every 5 min, `expires_at` column |
| Agent skill file | `skills/agent-neo-bank.md` | Registration and usage instructions for AI agents |
| Spending ledger (UTC) | `db/queries.rs` | Period determined at creation time, explicit UTC, first-tx upsert |
| Pagination everywhere | All list endpoints | `limit` and `offset` on agents, approvals, transactions |

## Deliverable

Full agent lifecycle: register with invitation code, get approved with token delivery, send USDC (async 202 model), view transactions. Global policy controls active. Rate limiting from day one.
