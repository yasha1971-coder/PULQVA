# T061 — Assess bundled JavaScript runtime compatibility

Status: ACTIVE — assessment drafted; review/CI and executable payload inspection pending
Outcome: a reviewed compatibility decision and exact candidate manifest for the
pinned yt-dlp/EJS + JavaScript runtime on Windows/Linux.

Verify against current primary sources: which EJS scripts the pinned executables
contain, supported Deno version and distribution/license/source identity, explicit
runtime path, restricted execution permissions, and disabled remote component downloads.
Compare QuickJS-NG only where size/performance warrants it. No automatic latest pin.

Acceptance:
- Record exact official candidate versions, assets, hashes, licenses and supported OSes.
- Define a bounded real Tor-only YouTube test with no user account/key/runtime installation.
- Identify package-size and permission implications; do not claim runtime isolation without tests.
- Record adopt/defer choice as ADR, using the next free ADR number (PR #63 proposes 0005).

Scope: assessment/manifest only; split actual packaging, launch wiring and live test
into follow-up atomic tasks. Preserve kernel and fail-closed transport.
