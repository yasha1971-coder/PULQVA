# NEXT

## CURRENT: T063 Linux external-network gate CI pending

PR #66 branch task/T063-deno-restricted-execution.
Verified head d68f4fac8e76f7c833bcf4397c3414d89040eed2: all 11 workflows green.
Run 36138486228 jobs 108082230510 (Windows) and 108082230733 (Linux) both
passed computation, five explicit permission denials, remote import rejection,
and zero loopback connections. Receipts preserved in RESTRICTED_FIXTURE_EVIDENCE.json.

This phase adds a fresh Linux network namespace with only loopback, verifies
distinct namespace and no external IPv4 route, drops UID/GID before Deno, and
runs the same fixtures inside it. CI gate NOT yet observed on this new head.
No host network namespace is changed; namespace is released when processes exit.
Reference: https://man7.org/linux/man-pages/man7/network_namespaces.7.html

ONE next action: observe exact-head CI, inspect the Linux network_gate receipt
and diagnose failures. Windows external-egress gate remains pending; T063 ACTIVE.
No OS sandbox, full privacy, YouTube, minimum-OS or application integration claim.
