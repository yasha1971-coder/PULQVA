# NEXT

## CURRENT: T063 Windows external-network rule CI pending

PR #66 branch task/T063-deno-restricted-execution.
Verified head a7d1d319d2d82e1801b639bc50de31b200dc10d6: all 11 workflows green.
Linux run 36139924719 job 108086958317 confirms fresh net namespace, lo only,
ENETUNREACH external IPv4 route and runtime UID 1001. Receipt committed.

This phase adds a temporary exact-program Windows outbound block. Requires
already-enabled effective profiles and running firewall service; checks ActiveStore
rule, program/address/protocol before and after fixtures, removes only its unique
owned rule in finally. Does not change global profile settings.
Reference: Microsoft Learn New-NetFirewallRule (NetSecurity).

ONE next action: observe exact-head CI and inspect Windows network_gate evidence.
Windows gate has NOT run yet; do not claim it passed. Diagnose red first.
T063 remains ACTIVE. Effective rule configuration is not packet capture or
DNS-service/descendant isolation; those broader claims remain unproven.
No app wiring or YouTube test. Linux result is a CI fixture boundary only.
