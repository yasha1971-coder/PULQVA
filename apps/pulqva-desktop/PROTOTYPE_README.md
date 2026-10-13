# PULQVA — experimental request / choice / file source candidate

**NOT a release ZIP, NOT a packaged executable, NOT privacy-certified, NOT v0.1 acceptance.**

Base: `54aa65874a5cc8a47777b8c1c15a22b53d42b98c`.
The immutable PULQVA Kernel, existing local proof UI, historical evidence,
Recovery validators, and pinned sidecar versions are unchanged.

This candidate connects a separately labelled experimental window to actual Rust
commands: `prototype_search`, `prototype_select`, `prototype_download`,
`prototype_clear`. It does not fabricate search results. The existing local proof
commands are not used by this window.

Supported experimental scope: Wikimedia Commons keyword search, 2–10 returned
video/WebM candidates, each at most 8 MiB. AI interpretation, Autopilot, additional
providers and formats remain requirements of the full product, not implemented
features of this slice. No public anonymity guarantee is made.

The backend verifies bundled Arti/yt-dlp/FFmpeg resource identities through the
existing materialization boundary, verifies the Tor SOCKS route before search,
retains actual search metadata, binds selection to a CSPRNG session nonce, allows
only one backend operation at a time, and accepts file completion only after the
existing Commons verifier and saved-file SHA-256 readback succeed. The frontend
supplies only the session nonce and candidate index for selection/download, never
a URL, proxy, executable, command line, or output path. Files are not auto-opened.

Normal completion stops Arti, removes the private runtime and retains only the
selected file under the OS Downloads/PULQVA directory. Observed Tor child loss,
nonzero downloader exit, timeout, oversized staging, unexpected output entries,
stale session or mismatched readback return errors. The window refuses normal
close while a backend operation is running. These mechanisms still require native
and adversarial E2E validation; process-tree containment, forced OS termination,
WebView2/OS egress, port substitution, filesystem races and disk quota enforcement
are NOT established by source inspection or UI unit tests. Polling is not a hard
filesystem quota. HTTPS delegates to the existing CommonsHttpsTransport, which
supervises its owned Arti child. The surrounding IPC lifecycle and shutdown path
still require native acceptance before public release.

## Validation actually performed locally

- `node --check frontend/prototype.js`: exit 0.
- `node --test frontend/test_prototype.mjs`: 8 passed, 0 failed, 0 skipped.
- Original `test_async_generation.mjs`: 3 passed, 0 failed, 0 skipped.
- The tests execute the actual JS with an in-memory DOM and controlled IPC; they
  do NOT execute Rust, Tauri, Tor, real search or file transfer.
- Raw TAP outputs are in the companion evidence folder. They are ordinary unit
  results, not a project-codec live admission matrix.
- `git diff --check` and clean-base patch application checked separately.
- Rust compiler and Cargo were unavailable locally; native compilation and
  `cargo --locked` acceptance have NOT run. No PASS is claimed for them.

## Dependency graph

The proposed desktop Cargo.lock imports the transitive package graph from the
already committed root and desktop locks. Existing package checksums/versions
are preserved and duplicate dependency names are version-qualified. No registry
resolution occurred. Cargo must accept the resulting graph with `--locked` before
this candidate is integrated. A lockfile rejection is a failed preflight, not
permission to regenerate with unpinned upgrades.

## Native admission commands

From the repository root, with the project's pinned Rust toolchain installed:

```sh
node --test apps/pulqva-desktop/frontend/test_async_generation.mjs apps/pulqva-desktop/frontend/test_prototype.mjs
cargo +1.91.0 check --locked --manifest-path apps/pulqva-desktop/src-tauri/Cargo.toml --all-targets
cargo +1.91.0 test --locked --manifest-path apps/pulqva-desktop/src-tauri/Cargo.toml --all-targets
```

The existing desktop CI workflow is extended to include the new JS tests. This
archive does not contain the native sidecars, a bundled WebView2 runtime, the
compiled application or a validated installer. With missing package resources,
search fails closed; the application must not download dependencies at runtime.

## Release gate

The candidate requires native Windows/Linux checks, real supported-query -> user
selection -> saved file tests, Tor failure/DNS/child-egress tests, complete runtime
bundling and clean-machine validation. Only then may a separately labelled
experimental binary be distributed. A full PULQVA v1 release must also meet the
unchanged natural-language understanding, Autopilot and both-platform contract.

Method references, reviewed 2026-10-10:
- https://v2.tauri.app/develop/calling-rust/
- https://v2.tauri.app/distribute/windows-installer/
- https://v2.tauri.app/reference/config/
