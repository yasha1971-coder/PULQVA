# T064 — Typed bundled Deno selection in yt-dlp launch plans

Status: READY — start after PR #66 merges
Outcome: tested Rust launch-plan representation that selects an explicit bundled
Deno executable and prevents implicit runtime/component discovery.

Inspect crates/pulqva-privacy/src/ytdlp_launch.rs and request-plan consumers.
Use ADR-0006 and verified candidate identities, preserving the privacy readiness
token and SOCKS remote DNS route. No frontend networking or shell strings.

Acceptance:
- Explicit typed runtime input emits --no-js-runtimes before exactly one
  --js-runtimes deno:<absolute-path> argument and --no-remote-components.
- No caller-controlled extra flags, runtime discovery or component downloads.
- Reject missing/relative/invalid runtime paths; spaces stay one argument.
- Metadata/media launch plans preserve ignore-config, Tor proxy and existing
  fail-closed contracts, with regression tests for the full argument vector.
- Paths are not falsely treated as hash-verified binaries: document the boundary
  to owned materialization/prelaunch verification and process environment.

Scope: typed plan and unit tests, not automatic runtime enablement, shipping
materialization, process-environment enforcement or live YouTube test.
Do not introduce a default host-Deno fallback to preserve old behavior.
