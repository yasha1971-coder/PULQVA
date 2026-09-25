# T063 — Verify restricted local Deno execution contract

Status: DONE within local fixture scope; closeout CI/merge pending
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

Evidence: c8900e98199def05a5dc2d029b0c8c025a6a0a7e passed all 11 workflows.
Run 36142395574 jobs 108095142055 (Linux), 108095142491 (Windows) both
passed deterministic computation, explicit NotCapable denials for net/read/write/
env/run and --no-remote rejection. Zero owned loopback connections.
Exact flags, executable identities and results:
sidecars/deno/RESTRICTED_EXECUTION_VERIFICATION.json.

Linux: fresh network namespace, only lo, external IPv4 ENETUNREACH, UID 1001.
Windows: effective ActiveStore outbound Block for exact executable, enabled
profiles, Any protocol/address; verified before/after; owned rule removal confirmed.
Windows evidence is effective policy, not packet capture or DNS-service/descendant
isolation. Linux namespace is CI-only. Neither is an application sandbox.
Normal fixture cleanup observed; abrupt runner cancellation cleanup not proven.
No real EJS challenge, yt-dlp-to-Deno integration or YouTube retrieval tested.
