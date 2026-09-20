# PULQVA Privacy Invariants v1

These invariants are architectural constraints, not marketing claims.

## P-001 — External network is privacy-gated
No product subsystem may intentionally open an external network connection outside the approved privacy transport boundary.

## P-002 — Tor failure is fail-closed
If Tor cannot bootstrap, loses required connectivity, or the privacy transport cannot verify the configured route, external network operations fail. PULQVA must not silently retry over clearnet.

## P-003 — Frontend does not own Internet access
The UI communicates with the trusted application core through typed IPC/commands. External network access belongs to the trusted backend/privacy layer.

## P-004 — Remote DNS stays inside the privacy path
Where a SOCKS transport is used, remote name resolution must be configured so DNS does not silently escape through the host resolver.

## P-005 — AI output is data, never code
Remote or local AI output must be parsed as a typed structure and validated. It must never be concatenated into an executable shell command.

## P-006 — Least persistent metadata
The default privacy path should retain only the local state required for user-visible function. Sensitive request/network logs are opt-in or minimized and must have explicit retention rules.

## P-007 — No false anonymity claim
PULQVA may claim specific verifiable properties such as "Tor-routed by default" and "no intentional direct-network fallback." It must not claim mathematically absolute anonymity.

## P-008 — Privacy regression is a release blocker
A test or implementation that requires disabling an invariant above is considered a defect, not a workaround.
