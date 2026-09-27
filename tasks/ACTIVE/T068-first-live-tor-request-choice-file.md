# T068 — First live Tor-backed request -> choice -> file

Status: ACTIVE. A verified; B0 Windows local SOCKS recovery pending native CI.
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

## B0 recovery evidence — 2026-09-27
At 3c84e61182e8c2b7a700f843e32ce9aac085c034, 11/12 workflows succeeded.
Candidate run 36302674079: Linux 108573162130 passed; Windows 108573162014
compiled and verified pins/lock but read_exact on an accepted socket returned
Winsock 10035/WouldBlock. Poisoned-environment test failed through the same helper.
The nonblocking listener's accepted stream had not been explicitly reset before
blocking Read/Write. Normalize accepted-stream mode without changing the listener,
finite timeouts, SOCKS domain assertion or refusal outcome. Add one native regression
forcing the inherited state on both OSes. Repaired native results PENDING.

Primary sources checked (entry/exit):
- https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-accept
- https://doc.rust-lang.org/std/net/struct.TcpStream.html#method.set_nonblocking
These support an OS fixture correction, not a dependency or transport-policy change.

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
No local rustc/Cargo available; repaired Rust tests have not been executed locally.

## Next
Observe repair-head CI, diagnose red first, then retrieve/compare candidate artifacts.
Use the verified lock for B, retire the one-off generator, and proceed toward C.
UI redesign, packaging, Autopilot, YouTube and new AI providers are out of scope.
