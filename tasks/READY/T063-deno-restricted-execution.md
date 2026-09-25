# T063 — Verify restricted local Deno execution contract

Status: READY — start after PR #65 merges
Outcome: executable evidence on Windows/Linux that the selected Deno invocation
can process stdin JavaScript while denying the capabilities challenge code
must not have. This advances ADR-0006 before application launch wiring.

Reuse verified candidate artifact materialization; do not install or discover
another runtime. Match the pinned yt-dlp Deno flags, use owned cache/work paths
and an environment allowlist, disable prompts, imports/downloads and updates.

Acceptance:
- A deterministic local stdin computation succeeds without external services.
- Negative fixtures explicitly identify permission denial for network, file
  read/write, environment access and child execution, not generic errors.
- Fixtures attempting network use only an owned loopback sentinel; no accidental
  Internet requests. Remote-module rejection is checked without relying on a
  connection failure as proof; monitor or block external access during fixtures.
- Bounded execution and cleanup; record exact executable identity/args/evidence.
- Both OS jobs distinguish successful computation from correct negative denials.

Scope: runtime contract validation, not OS sandbox certification, real YouTube,
application integration or full package readiness. No privacy relaxation.
