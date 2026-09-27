# T068 — First live Tor-backed request -> choice -> file

Status: ACTIVE. A/B0 and controlled B library tests verified; C1 live discovery LAUNCH.
PR73 / feat/T068-live-tor-e2e. Verified parent fe1fb03: 11 associated workflows green.

## Parent outcome — NOT complete
Natural-language request -> at least two real externally obtained candidates ->
explicit selection -> existing Tor-gated retrieval -> actual verified local file.
Use the same T067 coordinator, not a parallel demonstration or canned locator.

## Acceptance retained
- No account/API key or frontend external network. Tor default, remote DNS, fail closed.
- Bounded total time/output size, owned output, objectively verified file bytes/digest.
- Explicit selection belongs to the presented response; errors never yield a success receipt.
- Positive native Windows/Linux results distinguished from negative fail-closed evidence.
- Stable small public fixture first; YouTube/AI/UI/packaging remain separate gates.

## Verified B scope
Exact Cargo-generated graph adopted; existing parser preserved in commons.rs.
CommonsHttpsTransport borrows/verifies an owned Arti child and feeds the same search
coordinator. Native run 36314712742 passed controlled SOCKS/TLS/response tests on
Windows/Linux. Its cancellation/liveness unit test uses a pending future, not actual
Arti termination during network I/O. Bootstrap has a separate 90-second gate.
Copy-generators and duplicate candidate workflows are retired and must stay retired.

## C1 atomic external-discovery gate
The new native real_commons_discovery harness calls the existing library and
request_choices with fixed public full-text `countdown`. No canned response/URL.
It requires >=2 real validated choices, selects index 1 from that response, persists
bounded JSON evidence and verifies sticky cancellation clears stale search results.
Both native OSes require positive live discovery; no timeout-as-success exception.
Compile actual code, use existing pinned Arti, stop/reap on returned errors, clean
owned state, then publish only public metadata evidence. First results are PENDING.

## Remaining acceptance
C2 must retrieve the actually selected discovered locator via the existing typed
media boundary and same coordinator, verify file bytes/size/digest and ownership,
and demonstrate same-flow Tor-unavailable failure. Actual in-flight Tor loss and
cancellation evidence remain required, not replaced by C1 pre-cancellation testing.

## Evidence boundaries
No local Rust/Cargo or live request execution. C1 is an executable acceptance harness,
not a shipped UI, arbitrary natural-language interpretation or completed E2E.
Declared API SHA-1/size are server metadata, not authenticated downloaded-file receipts.
T066 race/retention, descendant confinement, SDK-content and packaging remain open.

## ONE next action
Observe exact-head commons-tor-discovery-check and existing CI. Diagnose red first;
otherwise inspect the externally obtained choice evidence and implement C2 directly.
Primary sources checked on 2026-09-27: MediaWiki API:Etiquette, API:Search and
API:Imageinfo (full URLs in NEXT.md). No dependency/provider/architecture change.
