# T068 — First live Tor-backed request -> choice -> file

Status: ACTIVE. A and B0 verified; B integration remains incomplete.
Branch feat/T068-live-tor-e2e; PR #73 draft. Last recorded all-green implementation
anchor cd785a38eb66b11721a4d65788dd9ba332169719 (12 associated workflows).
Current recovery: Windows LF/CRLF regression portability; native CI pending.

## Parent outcome — NOT complete
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
B0: native dependency/API/SOCKS trial passed Windows/Linux at cd785a3. The previous
accepted-stream mode problem is resolved within that fixture. Both generated B0
lock files are 33037 bytes and SHA256
ea282fedb7128d918b428cb30e5563b44075c770cb6672bffa682fe10091f5e5.
They are already mounted and readable; adoption must use exact verified bytes.
B: substantive endpoint-private CommonsTransport with maintained HTTPS/TLS,
streaming/time/redirect controls, Tor liveness, cancellation and negative tests.
No host curl, handwritten TLS, unreviewed graph or caller-overridable HTTP escape.
C: same coordinator with real discovery and existing yt-dlp retrieval; explicit
selection -> validated file/digest receipt on Windows/Linux plus fail-closed proof.
Production B/C are not implemented. Do not merge this parent after local tests.

## Current recovery evidence — 2026-09-27
95664ce, run 36311576973: Linux 108598413781 passed all stages. Windows
108598413887 passed source compilation and actual clean PREPARE but two test
assertions expected only LF; count=0. The previous literal-newline syntax error
is gone. All other 12 workflows are successful; no pending jobs in that snapshot.
Exact baseline blobs reproduce the two failures locally under CRLF, not LF.
The corrected test accepts actual LF/CRLF while still rejecting literal backslash+n.
Five tests pass locally for each checkout spelling; native repaired CI pending.
Raw byte/hash round-trip checks remain exact. No shipping file normalization,
Windows skip, changed dependency pin, new generator, Tor/kernel or permissions.

Primary-source review:
- https://docs.python.org/3.12/reference/lexical_analysis.html#physical-lines
- https://git-scm.com/docs/gitattributes
Prior provider/HTTPS reviews remain in ADR-0007, decisions/T068-review-2026-09-27.md
and decisions/T068-B0-https-candidate-2026-09-27.md.

## Evidence boundaries and next action
Local Python execution is not native Windows, Cargo, successful TLS, real Commons
availability or live E2E. API size/SHA-1 are declared metadata, not file receipts.
T066 cleanup/race/retention, descendant egress and SDK-content limitations remain.
ONE next action: inspect repaired native candidate preflight/Cargo; diagnose red,
otherwise adopt the already verified B0 graph with the real executor. No additional
generator or optional parser task. UI, packaging, Autopilot, YouTube and new AI
providers remain out of scope for this parent.
