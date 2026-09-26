# NEXT

## CURRENT: T065-C real Deno archive proof pending CI

PR70 branch feat/T065-deno-materialization. Verified parent
f2a47abe0fa631e6a33c271ba641eb6f655a6df7: all 11 workflows green.
Desktop 36224411245: Windows 108355535489, Linux 108355535542; native
source/stage/ZIP tests passed. A and B verified; T065 remains ACTIVE.

C adds CI-only download of existing pinned 2.9.7 artifacts, independent Python ZIP
reference hashes and a Rust integration test invoking materialize_deno through
source/ZIP/stage. It verifies actual disk bytes, receipt, native name, Unix mode,
cleanup, unrelated-file preservation and missing/directory/wrong-target rejection.
No binary execution. Exact head and run IDs saved in PR70 launch checkpoint.

ONE next action: inspect exact-head desktop-shell-check logs on BOTH platforms for
PULQVA_DENO_COMPAT and independent receipt agreement. Diagnose red before other work.
Only after all checks pass record real-artifact evidence and prepare T065 closeout.
Local Python syntax/YAML/continuity/diff checked; Rust unavailable locally.

No runtime activation or permanent publication. Environment/cache, process egress,
Windows ACL isolation and prelaunch replacement-race controls remain separate gates.
Historical Linux failures, SDK content pinning and positive Windows live media
limitations remain unchanged.
