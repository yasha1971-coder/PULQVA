# T017 — Add a readiness-gated Tor transport capability

Parent: PRIVACY FOUNDATION  
Status: DONE  
Date: 2026-09-21

## Result

Added public `ReadyTorTransport` with no public constructor.

Raw `TorSocksEndpoint` remains internal runtime data and no longer exposes a public proxy URL.
Only privacy-crate internal code can mint the ready capability.

## Verification

PR #16 verified head:
`7009058db21c86a442ce466bc200f6b035e5919a`

All existing privacy checks passed.
