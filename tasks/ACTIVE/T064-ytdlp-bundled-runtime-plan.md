# T064 — Typed bundled Deno selection in yt-dlp launch plans

Status: ACTIVE — A/B implemented; Linux verified; native Windows tests pending
Verified base: main@2b33eccc0b64b71bc133033e2b21f5562125683c (PR #68 merged).
Outcome: tested Rust launch-plan representation that selects an explicit bundled
Deno executable and prevents implicit runtime/component discovery.

Inspect crates/pulqva-privacy/src/ytdlp_launch.rs and request-plan consumers.
Use ADR-0006 and verified candidate identities, preserving the privacy readiness
token and SOCKS remote DNS route. No frontend networking or shell strings.

Acceptance (parent task is not complete until all are satisfied):
- Explicit typed runtime input emits --no-js-runtimes before exactly one
  --js-runtimes deno:<absolute-path> argument and --no-remote-components.
- No caller-controlled extra flags, runtime discovery or component downloads.
- Reject missing/relative/invalid runtime paths; spaces stay one argument.
- Metadata/media launch plans preserve ignore-config, Tor proxy and existing
  fail-closed contracts, with regression tests for the full argument vector.
- Paths are not falsely treated as hash-verified binaries: document the boundary
  to owned materialization/prelaunch verification and process environment.

Bounded phases:
A: additive typed runtime/path and runtime-only argument fragment with unit tests.
   Verified at 5e672012. Validates missing input and path syntax only; no
   existence, ownership, hash, executable-bit or environment attestation.
   B now consumes this policy; no runtime was automatically enabled.
B: integrate the representation into base/metadata/media plans and full argv
   tests, retaining the transport capability. Disable implicit discovery even
   when no bundled runtime is selected. Suppress plugins per ADR-0006.
   Implemented at 0fae2ad6; all 11 workflows green, Linux tests observed.
C: close coverage gap: execute ytdlp_ library tests in existing Linux/Windows
   metadata matrix. Existing rust-check only runs on Linux; example builds do
   not execute cfg(windows) library tests. Confirm named Windows tests in logs
   before completing parent acceptance. This is verification, no runtime change.

Scope: typed plan and unit tests, not automatic runtime enablement, shipping
materialization, process-environment enforcement or live YouTube test.
Do not introduce a default host-Deno fallback to preserve old behavior.
A syntactically valid path must still fail prelaunch if the file is absent,
not regular, not the verified candidate, or outside owned materialization.

Source check 2026-09-26: upstream EJS guide still describes default Deno discovery
and explicit deno:<path>. Pinned options.py confirms the reset-before-select order.
The mutable wiki minimum is not a reason to override ADR-0006's pinned version.
https://github.com/yt-dlp/yt-dlp/wiki/EJS
https://github.com/yt-dlp/yt-dlp/blob/3a08beaf031ab68f966401ead017ac81fe8486cf/yt_dlp/options.py

Local Rust tests not run: cargo is unavailable; container git clone failed DNS.
CI evidence and exact work head belong in the PR checkpoint; no success claim
until those exact-head checks are observed.
