# Review: Tauri v2 Auto-Update Research

> **Reviewer**: Claude Opus 4.6
> **Date**: 2026-02-28
> **Document reviewed**: `docs/investigations/tauri-auto-update-research.md`
> **Verdict**: Solid research document with a few inaccuracies and gaps that should be addressed before using as an implementation guide.

---

## Overall Assessment

The document is well-structured, covers the major topics comprehensively, and provides practical code examples. It correctly identifies `tauri-plugin-updater` as a separate plugin (not built-in like v1), gets the configuration shape right, and includes important operational concerns like key backup and version sync. The CI/CD workflow is production-ready. However, there are several API accuracy issues, a few missing topics, and some code snippets that need corrections.

**Rating: 7.5/10** -- Good research foundation, needs corrections before implementation.

---

## 1. Accuracy

### Correct

- Plugin name (`tauri-plugin-updater`) and installation commands are accurate.
- `createUpdaterArtifacts` field location under `bundle` is correct for Tauri v2.
- The `tauri.conf.json` config shape with `plugins.updater.pubkey` and `plugins.updater.endpoints` is correct.
- Permission name `updater:default` is correct.
- The `latest.json` platform key format (`{os}-{arch}`) is correct.
- Environment variable names `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` are correct for v2 (v1 used `TAURI_PRIVATE_KEY`).
- The signing requirement (updates are mandatory signed) is correct.
- `tauri-action@v0` is the correct action version for Tauri v2.

### Issues Found

**Issue 1: Rust `Update` type is not `Clone`-friendly for `Mutex<Option<Update>>`**

In Section 4d (Tauri Command Pattern), the code stores an `Update` in a `Mutex<Option<Update>>`. The `Update` type in `tauri-plugin-updater` holds internal state (HTTP client, download state). While it can technically be stored in a Mutex, this pattern has a subtle problem: the `download_and_install` method takes `&self` but the code calls `.take()` which moves it out of the Mutex. This is fine for a single use, but the pattern should document that once `take()` is called, the update is consumed -- a second call to `install_update` would get `"No pending update"`. The pattern itself is from Tauri's official docs, so it works, but it deserves a comment about single-use semantics.

**Issue 2: `Update.body` field type**

In Section 4d, the code accesses `u.body.clone()` and wraps it in `Some()`. In `tauri-plugin-updater`, the `body` field on `Update` is already `Option<String>`, not `String`. The code should be:

```rust
body: u.body.clone(),  // already Option<String>, no need to wrap in Some()
```

**Issue 3: `contentLength` in the TypeScript progress callback**

In Section 1 (Frontend API), `event.data.contentLength` is used in the `Started` event. The actual field name is `contentLength` and it is `number | undefined` (not guaranteed). The code handles this implicitly but should note that `contentLength` can be undefined/zero if the server does not send a `Content-Length` header. The Section 5 progress indicator mentions this correctly, but Section 1 does not.

**Issue 4: `app.restart()` vs `process::restart()`**

The document uses `app.restart()` in the Rust examples. In Tauri v2, the restart function is `app.restart()` on `AppHandle`, which is correct. However, this requires the `process` plugin to be registered. The document should mention that `tauri-plugin-process` is also needed:

```bash
cargo add tauri-plugin-process
npm install @tauri-apps/plugin-process
```

And in `lib.rs`:
```rust
.plugin(tauri_plugin_process::init())
```

The frontend `relaunch()` import from `@tauri-apps/plugin-process` is shown correctly, but the Rust-side plugin registration requirement is not mentioned.

**Issue 5: `updater_builder()` method name**

The document uses `app.updater_builder()` in Sections 6 and 8. The correct method name in `tauri-plugin-updater` is accessed via the `UpdaterExt` trait and the method is indeed `updater_builder()`. This appears correct. However, the `endpoints` method on the builder takes `Vec<url::Url>`, not `Vec<String>`. The code in Section 8 passes a `vec![url]` where `url` is a `String` from `format!()`. This would need to be parsed:

```rust
.endpoints(vec![url.parse().unwrap()])?
```

Or use `Url::parse()`.

**Issue 6: `check()` return type**

In the Rust API (Section 1), `app.updater()?.check().await?` returns `Option<Update>`, which is correctly handled with `if let Some(update)`. This is accurate.

**Issue 7: Windows `installMode` config location**

In Section 6 (Windows), the config shows:
```json
"plugins": {
  "updater": {
    "windows": {
      "installMode": "passive"
    }
  }
}
```

The actual Tauri v2 config for Windows install mode is set in `bundle.windows.nsis` or passed at runtime, not under `plugins.updater.windows`. The `installMode` for the updater is set via the `UpdaterExt` builder or as part of the install call, not in `tauri.conf.json`. This section needs correction. The install mode can be configured at runtime:

```rust
update.download_and_install(
    |_, _| {},
    || {},
).await?;
```

Or through the builder's `installer_args` if customization is needed. Check the latest `tauri-plugin-updater` docs for the exact API, as this has changed between v2 betas.

**Issue 8: `on_before_exit` hook**

Section 6 shows `app.updater_builder().on_before_exit(...)`. Verify this method exists on the builder -- in some versions of the plugin, the before-exit behavior is handled differently. The concept is correct (Windows needs cleanup before forced exit) but the exact API should be verified against the current `tauri-plugin-updater` version.

---

## 2. Completeness

### Topics Covered Well

- Plugin installation and configuration
- Multiple distribution methods (GitHub, custom server, CrabNebula, S3)
- Signing (Tauri-level and OS-level)
- Update checking strategies (launch, periodic, manual, command)
- UX patterns
- Platform differences
- CI/CD with GitHub Actions
- Version management
- Rollback strategies
- Common pitfalls (excellent section)

### Missing Topics

**Missing 1: `tauri-plugin-process` dependency**

The `relaunch()` / `app.restart()` functions require `tauri-plugin-process` to be installed and registered. This is a hard dependency that is never mentioned in the document. Without it, the app will not restart after update installation.

**Missing 2: Custom headers for private repositories**

If the GitHub repository is private, the update endpoint needs authentication headers. The document mentions `headers` in the `check()` options but does not explain how to handle private repo releases, which is a common scenario for commercial apps. The `GITHUB_TOKEN` approach or using a proxy endpoint should be discussed.

**Missing 3: Proxy/firewall considerations**

Enterprise users behind corporate proxies or firewalls may not be able to reach GitHub or custom update servers. The `proxy` option is mentioned in passing but deserves more attention for desktop apps deployed in enterprise environments.

**Missing 4: Testing the update flow locally**

The document has no section on how to test updates during development. This is one of the hardest parts of implementing auto-updates. Recommendations should include:
- Building v1, installing it, then building v2 with the update server pointing to the v2 artifacts
- Using a local HTTP server to serve `latest.json` during development
- The `TAURI_DEV` flag behavior (updates are typically skipped in dev mode)

**Missing 5: Migration from v1 updater**

The document mentions `"v1Compatible"` for `createUpdaterArtifacts` but does not explain the migration path in detail. If this project ever had a v1 build distributed, users on v1 would need v1-compatible artifacts for one transitional release.

**Missing 6: Update payload size optimization**

The document correctly notes Tauri does not support delta updates. It should also mention:
- Typical binary sizes for Tauri apps (50-150 MB depending on platform)
- Compression: `.tar.gz` for macOS is already compressed; NSIS installers can be configured for compression level
- CDN caching headers for update endpoints

**Missing 7: Automatic vs manual update install**

The document does not clearly distinguish between fully automatic updates (download + install without user interaction) versus user-confirmed updates. For a financial/wallet application like Tally, this is a critical UX and security decision. Users of a wallet app should arguably always confirm updates manually.

**Missing 8: Rate limiting and caching**

If many users check the same GitHub release endpoint simultaneously, GitHub may rate-limit. The document should mention:
- GitHub API rate limits for unauthenticated requests (60/hour per IP)
- Using a CDN or caching proxy in front of the update endpoint for high-traffic apps
- The static `latest.json` file on GitHub Releases is served via CDN and is less likely to be rate-limited than the API, but it is still worth noting

---

## 3. Practicality

### Would this work end-to-end?

**Mostly yes**, with the following caveats:

1. **The CI/CD workflow is production-ready.** The GitHub Actions matrix, certificate import, and tauri-action usage are all correct and would produce valid release artifacts.

2. **The Rust plugin registration is correct** and would compile. The `setup` hook with async update check is the standard pattern.

3. **The TypeScript frontend code would work** with the caveat about `tauri-plugin-process` needing to be installed for `relaunch()`.

4. **The version bump script has a portability issue.** The `sed -i.bak` syntax works on macOS but the script uses it in a way that assumes a specific `tauri.conf.json` structure. A more robust approach would use `jq` for JSON or `node -e` for programmatic updates. Also, `sed` on versions like `0.10.0` could accidentally match other occurrences. The `npm version` + `jq` approach is safer:

```bash
# Safer tauri.conf.json update
jq --arg v "$NEW_VERSION" '.version = $v' src-tauri/tauri.conf.json > tmp.json && mv tmp.json src-tauri/tauri.conf.json
```

5. **The Tauri Command Pattern (Section 4d)** is the most practical approach for this project and would integrate well with the existing Rust backend architecture. However, it needs the `body` field type fix noted above.

### Specific to Tally Agentic Wallet

The document is generic. For this project specifically:
- **Note:** This app does NOT hold private keys — all key management is handled by Coinbase's AWAL CLI. However, auto-updates should still require user confirmation as a general best practice for desktop apps.
- The `on_before_exit` hook for Windows is critical -- unsaved transaction state or pending approvals must be handled.
- Consider whether the update binary itself should be verified beyond Tauri's signature (e.g., checksum verification, build reproducibility).

---

## 4. Code Quality

### Rust Snippets

- **Generally correct.** The `async move` spawn pattern in `setup` is idiomatic.
- The `Mutex<Option<Update>>` pattern works but is not ideal for async code. Consider using `tokio::sync::Mutex` instead of `std::sync::Mutex` to avoid blocking the async runtime if the lock is ever contended. However, since the lock is only held briefly (to take/put the value), `std::sync::Mutex` is acceptable here.
- Error handling uses `.map_err(|e| e.to_string())` which is fine for Tauri commands but loses error context. For production, consider a custom error enum with `serde::Serialize`.

### TypeScript Snippets

- The progress callback pattern is correct and follows the official API.
- The JSX in Section 5 mixes pseudo-code with real React. The `showUpdateBanner` function returns JSX directly but is not a proper React component (no hooks, no state management for the banner). This is fine for illustration but should be noted as pseudo-code.
- Missing `async` keyword awareness: `handleInstall` in the progress component is correctly async, but there is no error handling (no try/catch).

### GitHub Actions

- The workflow is well-structured with proper matrix strategy.
- The `concurrency` group is a good practice.
- The `rust-cache` action targeting `src-tauri` workspace is correct.
- **One concern**: The macOS certificate import script creates a temporary keychain and sets it as default. This is correct for CI but the script should clean up the keychain after the build (though in CI this is less critical since the runner is ephemeral).
- The Linux dependencies list (`libwebkit2gtk-4.1-dev`) is correct for Tauri v2 (v1 used `4.0`).

---

## 5. Security

### Correct

- The document correctly emphasizes that update signing is mandatory.
- The private key backup warning is appropriate and important.
- The distinction between Tauri update signing and OS-level code signing is clear.
- The note about app-specific passwords (not Apple ID passwords) is a good security callout.

### Concerns

**Concern 1: Private key in environment variable**

The document shows `TAURI_SIGNING_PRIVATE_KEY` containing the key content directly. For CI, this means the full private key is stored as a GitHub secret, which is reasonable. However, the document should note:
- GitHub secrets are encrypted at rest but available to any workflow in the repo
- Consider restricting which workflows/environments can access the signing secrets
- GitHub Environments with protection rules can limit secret exposure

**Concern 2: No pinning of update endpoint TLS certificates**

The update check goes over HTTPS, which is good. But for a financial application, consider certificate pinning or at minimum noting that a MITM who compromises the TLS layer still cannot forge updates (because of the Tauri signature). The document should explicitly state: "Even if the HTTPS endpoint is compromised, attackers cannot serve malicious updates because they lack the private signing key." This is the key security property and it deserves emphasis.

**Concern 3: No integrity check on `latest.json` itself**

The `latest.json` file is not signed. An attacker who compromises the update endpoint could serve a `latest.json` pointing to a different (older, vulnerable) version's legitimate binary and signature. This is a downgrade attack vector. The version comparator (default: only newer versions) mitigates this, but if `allowDowngrades` or a custom comparator is used, this becomes a risk. The document should note this.

**Concern 4: Key rotation has no practical path**

The document mentions `updater_builder().pubkey()` for runtime key override but does not explain how to actually rotate keys. Since the public key is embedded in the binary, key rotation requires:
1. Release an intermediate version signed with the OLD key that contains the NEW public key
2. All subsequent releases use the NEW key
3. Users who skip the intermediate version are stuck

This is a known hard problem and should be called out explicitly.

**Concern 5: For a wallet application specifically**

- **Note:** This app does NOT store private keys (Coinbase AWAL CLI manages all keys remotely), so the attack surface is lower than a typical wallet. A compromised update could still inject malicious behavior but cannot steal keys directly.
- Consider requiring multiple signatures (e.g., two-of-three team members must sign a release).
- Consider a manual verification step where the user can verify the update hash against an independent source (website, social media).
- The `tauri-plugin-updater` does not support multi-sig natively, so this would need a custom verification layer.

---

## Summary of Required Corrections

| Priority | Section | Issue |
|----------|---------|-------|
| **High** | General | Add `tauri-plugin-process` as a required dependency for restart/relaunch |
| **High** | 6 (Windows) | Correct the `installMode` configuration location -- it is not under `plugins.updater.windows` in v2 |
| **Medium** | 4d | Fix `body` field type: `u.body.clone()` not `Some(u.body.clone())` |
| **Medium** | 8 | Fix `endpoints()` to accept `Vec<Url>` not `Vec<String>` |
| **Medium** | New | Add a section on testing updates locally during development |
| **Medium** | New | Add guidance specific to wallet/financial apps (always require user confirmation) |
| **Low** | 8 | Make the version bump script more robust (use `jq` instead of `sed` for JSON) |
| **Low** | 6 | Verify `on_before_exit` API exists on current plugin version |
| **Low** | New | Add note about downgrade attack vector when `allowDowngrades` is enabled |
| **Low** | New | Discuss key rotation challenges in more detail |

---

## Recommendations for Tally Agentic Wallet Implementation

1. **Never auto-install updates.** Always show a confirmation dialog with version number and release notes. This is a wallet app -- users must consent to code changes.

2. **Use the Tauri Command Pattern** (Section 4d) as the primary update mechanism. It gives full Rust-side control, which is appropriate for the existing architecture.

3. **Start with GitHub Releases** as the distribution method. It is free, reliable, and the `tauri-action` handles `latest.json` generation automatically.

4. **Add `tauri-plugin-process`** to the project dependencies alongside the updater plugin.

5. **Generate and securely back up the signing keypair** before the first release. Store the private key in a password manager or vault, not just on a developer machine.

6. **Test the full update cycle** (v0.1.0 installed -> v0.2.0 update detected -> user confirms -> download -> install -> restart) before shipping the first release. This is the most error-prone part.

7. **Consider adding a cryptographic hash** of the update binary to `latest.json` as an additional verification layer beyond the signature, especially given the financial nature of the app.
