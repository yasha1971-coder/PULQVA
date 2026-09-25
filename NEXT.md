# NEXT

## CURRENT: T060 extraction complete; closeout documentation CI/merge pending

Verified head: `24a7a066f938bc7d6336deb39cc56c84a132b891`, all 11 workflows green.
PR #62 branch: task/T060-verified-ffmpeg-archive-extraction.
Desktop run 36095498238 verifies Windows/Linux extraction; media run 36095497965
verifies actual Linux retrieval and Windows fail-closed timeout only.
Earlier Linux failure at c2161c8 remains unexplained, not reproduced; do not claim fixed.

T060 moved to DONE with acceptance evidence and scope limitations.
This documentation-only closeout has not yet passed its own CI. Do not merge based
on parent checks. ONE next action: observe current PR #62 head checks, diagnose any
failure, otherwise mark ready and merge without another documentation-only checkpoint.
No new implementation task until merge succeeds.

After merge: T061 (READY) assesses pinned Deno/EJS compatibility; PR #63 separately
contains requested sponsorship/license documentation. Preserve that PR and verify its
own head before merge. Runtime FFmpeg publication/prelaunch remains future work.

Deferred research: compare compact local intent parsing; source selection that minimizes
query recipients; resumable/cancellable downloads; per-binary source/license inventory.
These are proposals, not implemented features or accepted dependency changes.
