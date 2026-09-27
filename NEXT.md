# NEXT

## CURRENT: T068-B0 native HTTPS dependency/API trial pending

Branch feat/T068-live-tor-e2e; PR #73 remains draft. Main is unchanged.
Verified PR head b223f3a8825256c002b91bffd8508ecb9ba275be: all 11 associated
workflows succeeded, including discovery-contract-check 36300961027. A's query
correction is no longer pending. Parent T068 has not delivered live E2E.

B0 is the required dependency step for the real HTTPS executor, not a new provider
or alternative demo coordinator. A new native Windows/Linux candidate workflow
copies the checkout, adds four exact HTTP/TLS/runtime pins to the COPY, asks Cargo
to resolve them while retaining existing registry identities, and tests API
compatibility plus observed SOCKS domain forwarding and refusal under poisoned
proxy environment. Shipping manifests and Cargo.lock are untouched. Local Python
syntax/YAML checks passed; no local Rust execution (Cargo absent, clone DNS failed).

ONE next action: inspect this exact head's discovery-https-candidate-check. If red,
recover its exact failing log. If green, retrieve BOTH candidate artifacts, verify
receipt hashes and compare the actual generated locks/features before accepting
one as the committed workspace graph. No manually invented checksums or full-graph
blind upgrade. Then implement the endpoint-private CommonsTransport executor using
the accepted graph and existing CommonsSearchPlan; remove the one-off generator
once its lock is imported. No further optional parser work.

B0 local refusal is NOT TLS success, live Tor, real Commons discovery, OS egress
confinement or completed B. B still needs controlled HTTPS positive/negative tests,
streaming limits, one deadline, Tor liveness and cancellation. C connects real
external choices and existing retrieval through the SAME coordinator, validates
actual file bytes and records positive Windows/Linux results separately from
fail-closed evidence. Kernel/UX/privacy, T066 limitations and packaging gates remain.

Milestone entry/exit primary-source review: decisions/T068-B0-https-candidate-2026-09-27.md.
Exact new commit and native run IDs belong in PR #73's launch checkpoint.
