# T068 — First live Tor-backed request -> choice -> file

Status: ACTIVE. A correction verified; B0 native dependency/API trial pending.
Branch feat/T068-live-tor-e2e; PR #73.
Verified PR head b223f3a8825256c002b91bffd8508ecb9ba275be: 11 associated workflows
successful, including native discovery-contract-check 36300961027.

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
A: request plan/parser/CandidateSearch integration and query correction verified
with synthetic responses. Server filtering does not replace local validation.
B0: exact maintained HTTPS dependency graph plus native local SOCKS/API trial.
The generator modifies a COPY only. Export Cargo.lock, manifest, checksum receipt
and feature graph, compare both OSes, then commit an accepted real generated lock.
No shipping dependency change or production executor is included in B0.
B: implement endpoint-specific CommonsTransport with Tor HTTPS execution,
streaming/time/TLS/redirect controls, Tor liveness and negative tests. No host curl,
hand-written TLS, unreviewed graph or generic caller-overridable HTTP escape hatch.
C: same coordinator with real discovery and existing yt-dlp retrieval; explicit
selection -> validated file/receipt on Windows/Linux plus same-flow negative proof.
B and C NOT IMPLEMENTED. Do not mark the parent DONE or merge after local tests.

## Current-world review
See decisions/T068-review-2026-09-27.md, ADR-0007 and
 decisions/T068-B0-https-candidate-2026-09-27.md. Exact reqwest 0.13.5 source was
inspected rather than assuming old TLS features still exist. Candidate selection
is not adoption. Root-bundle maintenance and inherited runtime limitations remain.
T068 is one-provider full-text discovery, not arbitrary language understanding.

## Evidence boundaries
Current API response fixtures are synthetic. The new probe is a local SOCKS server
that refuses CONNECT before TLS; it does not access Commons, certify TLS behavior,
observe every OS connection or prove Tor/E2E. Its parser/body paths are compile-only
until B tests them. API SHA-1/size are server-declared metadata, not file receipts.
T066 cleanup/retention/race, descendant egress and SDK-content limitations remain.
Local Python syntax/YAML checks passed; Cargo absent and clone DNS failed again.
Rust compilation, candidate resolution and both native probe results await CI.

## Next
Observe B0 exact-head CI and retrieve/compare candidate lock artifacts. Diagnose
any failure without silently upgrading existing dependencies. Use the verified
lock for B, retire the one-off generator, then proceed directly toward C.
UI redesign, packaging, Autopilot, YouTube and new AI providers are out of scope.
