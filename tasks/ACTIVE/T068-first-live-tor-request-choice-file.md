# T068 — First live Tor-backed request -> choice -> file

Status: ACTIVE. T068-A request/parser adapter implemented; native CI PENDING.
Branch: feat/T068-live-tor-e2e. Verified base: 3f85e10c621380195a90ea4864df4acf88c7cad4.

## Atomic parent outcome (NOT complete)
Through the existing T067 coordinator, prove one bounded real external path:
natural-language request -> at least two real candidates -> explicit selection ->
real downloaded file -> typed receipt, with all external traffic privacy-gated.

## Acceptance retained
- Stable public source, small payload, no account/API key; avoid YouTube challenges.
- Same T067 coordinator, not parallel demo orchestration.
- At least two real externally obtained candidates through the privacy boundary.
- Explicit selection validated against that presented set.
- Existing Tor-gated retrieval downloads the selected content.
- Bounded time/output size, test-owned output, objective actual-file validation.
- No frontend networking, remote DNS through Tor, no clearnet fallback.
- Tor-unavailable negative case and platform-specific native CI evidence.
- Positive live Windows/Linux results distinguished from fail-closed-only results.

## Bounded implementation phases
A: CommonsSearchPlan, bounded typed response parser, returned media metadata and
CandidateSearch adapter for request_choices. Native local tests and compile-fail
raw-port capability test. No network executor, no real choices or download proof.
B: endpoint-specific Tor HTTPS execution enforcing streaming/time/TLS/redirect
policies; no host curl dependency or generic HTTP escape hatch. NOT STARTED.
C: same coordinator with real discovery and existing yt-dlp retrieval; explicit
selection -> validated real file/receipt; Windows/Linux positive + negative proof.
NOT STARTED. Do not mark the parent DONE after A's local contract tests.

## Current evidence and limitations
ADR-0007 records today's official MediaWiki/yt-dlp source review. Actual Commons
query availability and >=2 usable small WebM candidates are unverified. All A
response fixtures are SYNTHETIC. Public adapter construction requires verified Tor
capability, but a transport trait/plan alone does not prove DNS/egress enforcement.
No local Rust toolchain; container clone failed DNS. Compilation/testing pending CI.
Existing T066 safety/retention limitations are not erased by this phase.

## Next
Observe exact-head native discovery-contract-check and existing CI; diagnose red.
Only after A is verified, implement the real executor in a later bounded phase.
Out of scope: UI redesign, packaging, Autopilot, YouTube compatibility, new AI
provider or claims of absolute anonymity. Do not merge this partial parent task.
