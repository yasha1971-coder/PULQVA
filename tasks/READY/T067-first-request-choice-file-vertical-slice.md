# T067 — First request -> choice -> file vertical slice

Status: ACTIVE (PREPARE).

## Atomic outcome
Given one natural-language request and deterministic backend fixtures, produce typed
candidate choices, accept one explicit selection, execute the existing retrieval
boundary, and return a predictable saved-file receipt.

## Acceptance
- Existing typed intent boundary receives the request; no shell text.
- At least two typed candidates exercise RESULT CHOICE.
- Explicit selection is validated against the presented set.
- Retrieval crosses the existing privacy-gated media abstraction; no direct fallback.
- Fixture writes one bounded known payload only inside an owned temporary output root.
- Receipt identifies selected candidate and resulting path/size.
- Invalid selection and retrieval failure produce no success receipt.
- Integration test demonstrates request -> choices -> selection -> file/receipt.
- Fixture evidence is deterministic/local, never a live Tor/Internet claim.
- Existing Windows/Linux checks remain green.

## Out of scope
UI redesign, Autopilot, provider expansion, packaging, new network clients,
Tor bypass, live-site reliability tuning, sandbox/anonymity claims.

## Next gate
Bounded live Tor-backed search/retrieval through this same coordinator.
