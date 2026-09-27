# T068 — First live Tor-backed request -> choice -> file

Status: READY.

## Atomic outcome
Through the existing T067 coordinator, prove one bounded real external path:
natural-language request -> at least two real candidates -> explicit selection ->
real downloaded file -> typed receipt, with all external traffic privacy-gated.

## Fixture constraints
- Stable public source, small payload, no account/API key.
- Avoid YouTube/JS challenge for this generic first-live proof.
- No frontend network and no new generic HTTP escape hatch.
- Remote DNS remains in the Tor path.
- Failure to establish ReadyTorTransport produces no external request.
- Download is bounded by time and output size and isolated to a test-owned directory.
- Receipt/file content must be objectively verifiable.
- Evidence must distinguish Linux/Windows positive retrieval from fail-closed-only results.

## Acceptance
- Same coordinator/interfaces verified by T067; no parallel demo-only orchestration.
- >=2 real candidates are obtained externally through the privacy boundary.
- Explicit candidate selection is validated.
- Selected content is downloaded through the existing Tor-gated retrieval adapter.
- File exists, is nonempty and matches fixture-specific validation.
- Tor-unavailable negative case fails closed.
- Exact-head CI records platform-specific evidence.

## Out of scope
YouTube EJS/POT/SABR reliability, UI, Autopilot, packaging, provider expansion,
absolute anonymity/sandbox claims.
