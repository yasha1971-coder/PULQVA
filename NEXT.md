# NEXT

## CURRENT: T068-A Commons discovery contract implemented; exact-head CI pending

Branch: feat/T068-live-tor-e2e.
Verified base: main@3f85e10c621380195a90ea4864df4acf88c7cad4 (T067 merged).
The implementation SHA, draft PR and registered run IDs belong in the PR checkpoint.

Implemented: narrow CommonsSearchPlan requiring ReadyTorTransport, UTF-8 query
encoding, bounded typed API parser, small WebM source validation/provenance, and
CandidateSearch adapter feeding the existing request_choices coordinator. Native
Windows/Linux local contract CI added. No new upstream dependency versions.

Today's official-source review is in ADR-0007. Existing yt-dlp supplies retrieval,
not Commons search; the former assertion that no discovery executor was needed
is corrected. This phase does NOT perform HTTP, prove live Tor/DNS or download
anything. Synthetic responses must not be presented as real external choices.

ONE next action: inspect exact implementation-head discovery-contract-check and
other triggered CI. Diagnose red before further work. Then a later bounded T068-B
phase implements endpoint-specific Tor HTTPS execution with streaming byte/time
limits, TLS validation and no redirects/ambient proxy/direct DNS fallback. Reuse
the existing verified media retrieval for C, not a second demo downloader.

T068 is ACTIVE, not DONE. No automatic merge, no live-E2E percentage claim. Local
Cargo absent and clone failed DNS: Rust compilation/tests were not run locally.
Kernel/privacy/UX and existing media/Tor launch behavior are unchanged.
