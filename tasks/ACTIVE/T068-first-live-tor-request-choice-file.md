# T068 — First live Tor-backed request -> choice -> file

Status: ACTIVE. A baseline verified; A query-envelope correction pending native CI.
Branch: feat/T068-live-tor-e2e. PR #73.
Verified PR head: c0b68dfbec23967c0f12009716f1a51286a2762c (11 associated workflows).
Native contract run 36299874386 passed on Windows/Linux; actual test checkout was
GitHub integration merge c6338de1dabe93897e5cf3860031953c98c8c534.

## Atomic parent outcome (NOT complete)
Through the existing T067 coordinator, prove one bounded real external path:
natural-language request -> at least two real candidates -> explicit selection ->
real downloaded file -> typed receipt, with all external traffic privacy-gated.

## Acceptance retained
- Stable public source, small payload, no account/API key; avoid YouTube challenges.
- Same T067 coordinator, not parallel demo orchestration.
- At least two real externally obtained candidates through the privacy boundary.
- Explicit selection validated against that presented set, not a canned download URL.
- Existing Tor-gated retrieval downloads the selected content.
- Bounded total time/output size, owned output, objective actual-file validation.
- No frontend networking, remote DNS through Tor, no clearnet fallback.
- Tor-unavailable negative case and platform-specific native CI evidence.
- Positive live Windows/Linux results distinguished from fail-closed-only results.

## Bounded implementation phases
A: request plan/parser/CandidateSearch integration verified with synthetic responses.
A review correction: provider filters now match WebM <=8 MiB local acceptance and
request one image revision. Existing client checks remain mandatory. Added full-query
and exact-size-boundary tests; their CI is pending. No real HTTPS request performed.
B: implement endpoint-specific Tor HTTPS execution, streaming/time/TLS/redirect
controls and poisoned-environment/negative tests. No host curl, hand-written TLS,
unreviewed dependency graph or generic caller-overridable HTTP escape hatch.
C: same coordinator with real discovery and existing yt-dlp retrieval; explicit
selection -> validated file/receipt on Windows/Linux plus same-flow negative proof.
B and C NOT IMPLEMENTED. Do not mark the parent DONE or merge after local tests.

## Current-world review
See decisions/T068-review-2026-09-27.md and ADR-0007. Official CirrusSearch supports
filemime exact matching and filesize in 1024-byte units. This closes an availability
mismatch between broad server results and our narrow client acceptance; no live
rate/success/latency improvement was measured. T068 is one-provider full-text discovery,
not yet arbitrary natural-language understanding or general Internet search.

## Evidence boundaries
Current API response fixtures are SYNTHETIC. Tor capability construction and typed
arguments are not transport execution, TLS, DNS observation, ongoing Tor liveness,
process confinement or verified downloaded bytes. API SHA-1/size are server-declared
metadata. T066 cleanup/retention/race and SDK-content limitations remain open.
Local Cargo unavailable; container DNS still fails. New tests await native CI.

## Next
Observe this correction's CI, then implement B directly without optional parser work.
Do not add a documentation-only closeout or silently replace the fixture/backend.
UI redesign, packaging, Autopilot, YouTube and new AI providers remain out of scope.
