# PULQVA — continuation entry

This page routes development; it does not merge unfinished product code.

**Active branch:** `feat/T069-intent-contract` — **PR #80**.
Read [PROJECT_ENTRY.json](PROJECT_ENTRY.json), resolve the live PR HEAD, and then
open [START_HERE on that branch](https://github.com/yasha1971-coder/PULQVA/blob/feat/T069-intent-contract/START_HERE.md).
Read Kernel, State, NEXT and recovery/INDEX at that SAME commit. main State/NEXT
remain the merged-baseline snapshot; do not restart T068 from them.

Recovery floor: `65f5f2d708c058d2bb98c612cc994f2d1f6afc9d`. It contains the sealed
continuation record, retained model evidence, ADR index and recovery validator.
Use this immutable floor only when live work is unavailable, never to overwrite
newer commits. Conflicts or missing evidence require reconciliation, not guessing.

G1 is accepted only for deterministic adapter tests; G2 is the next product task.
Full AI/Tor/choice/file acceptance and Windows/Linux packaging remain open.
No autonomous scheduler, merge or release is authorized by this pointer.

An independently retained Git bundle with a successful restore receipt recovers
source/history and committed evidence without chat or Actions. It does not include
all GitHub metadata or external binaries. See ADR-0010 on the active branch.
