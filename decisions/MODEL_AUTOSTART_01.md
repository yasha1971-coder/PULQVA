# MODEL_AUTOSTART_01 — one-click local model, minimal authority

Date: 2026-11-04. Architecture/acceptance checkpoint, not an implemented feature.
Owner's strategic veto is reserved; this record authorizes no merge or release.

## Observed baseline (exact source c01dbe99a311fe9f7804d0663b7d81bf432f72de)

- Windows owner tested AppContainer pipe PING/PONG on f389cfd: 1 passed / exit 0, SHA-verified ZIP and EXE. This verifies bounded child IPC, NOT model execution or network isolation.
- `crates/pulqva-intent-server/src/windows_isolation.rs` offers an AppContainer child with pipes, no declared network capabilities and owned Job Object; c01dbe9 exposes the module in production Rust. No Qwen/GUI wiring.
- `apps/pulqva-desktop/src-tauri/src/interpret_local.rs` invokes a pre-existing HTTP model at fixed 127.0.0.1:18081. It does not start or own the model.
- `crates/pulqva-intent-server/src/lib.rs` has a separate `std::process::Command` llama-server lifecycle with `--host 127.0.0.1`, readiness and Drop cleanup; using it alone is NOT proof of outbound containment.
- No llama runtime or GGUF file is committed in `sidecars/llama.cpp/` (only policy/schema/contract). Existing real Windows journey still depends on a manually started model.

## Narrow product decision

The first owner-visible improvement is removal of the manual Qwen process, **not** a new inference framework.

1. Reuse the existing strict intent schema, typed Rust parser, singleflight guard and UI command; do not duplicate discovery or Tor.
2. Prefer an **owned, bounded, no-network child process communicating by stdio/AppContainer IPC**, if a verified pinned llama.cpp executable can take a fixed prompt and produce exactly one bounded structured response without an HTTP listener. This CLI compatibility and resource access MUST first be demonstrated; it is NOT assumed.
3. Do **not** launch an unrestricted llama-server just to satisfy zero-configuration. A loopback-only bind does not prove absence of external egress. Do not give AppContainer broad network capabilities to preserve HTTP.
4. Use an explicit packaged runtime/model manifest with exact byte sizes and SHA-256, checked before execution. The ~3 GB observed GGUF is not itself included in the repository or any current release.
5. Keep Windows and Linux platform adapters separate. Preserve the currently accepted owner journey while the new path is qualified.
6. Existing Tauri IPC and user input must never become shell commands or executable paths. No new account, key, direct-network fallback or manual installer.

## One next atomic implementation task — MODEL-IPC-01

Build a fixed-input, **non-networked Windows model-CLI witness** using the existing AppContainer launcher, without modifying the desktop entrypoint:
- Pin exact llama.cpp Windows CLI executable and GGUF identities/flags in a test-only receipt (no unverified names or digests).
- Confirm AppContainer process launch, read-only model access, one fixed-public intent, strict JSON decoding and owned cleanup using real Windows.
- Capture `PASS / FAIL / NOT TESTED` individually for process, model loading, bounded output, typed decoding, PID cleanup and direct-network egress. A PING/PONG result may not substitute for any of them.
- On failed launch, preserve exact Windows error without changing security policy or running an unconfined fallback.
- Acceptance requires actual Windows output and immutable source/runtime/model hashes, not solely CI build success.

Only after MODEL-IPC-01 PASS: wire the owned model into `interpret_local_request` with a bounded lifecycle and an explicit missing/bad-assets failure, then independently accept GUI one-click request-to-file. Do not bundle an unqualified binary into a consumer package.

## Status

ADR: RECORDED. Windows child pipe: PASS (owner). Qwen-in-AppContainer: NOT TESTED.
Automatic GUI Qwen startup, bundled model, no-egress and clean-machine: PENDING.
