# T061 — Assess bundled JavaScript runtime compatibility

Status: DONE — assessment only; closeout commit CI/merge pending
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

Acceptance evidence:
- ADR-0006 records a trial candidate, alternatives, permissions, OS/size limits and
  bounded Tor-only test plan. Shipping adoption remains deferred.
- sidecars/deno/CANDIDATE.json pins exact official candidate assets and source.
- EJS_INSPECTION.json records both hash-authenticated yt-dlp payloads matching
  EJS 0.8.0 core/lib. Inspection repeated successfully on 2026-09-25.
- ec01fa0a4f665da7f221d2abbe2c1e89f0c4ee3f: all 10 triggered workflows successful.
  This does not mean CI itself ran the local payload inspector.
- Deno archive hashes are upstream metadata, not locally recomputed. Actual
  Deno launch, Linux minimum ABI, permissions and YouTube trial remain untested.

Next atomic task: T062 verifies Deno artifact materialization in CI, not app wiring.
