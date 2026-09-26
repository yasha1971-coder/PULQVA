# NEXT

## CURRENT: T066-B4 native directory replacement tests; CI pending

PR71 branch feat/T066-deno-environment. Verified head
711b294c6c3c3d07ce1ec6528a717bd02e7565f8: all11 workflows green.
Media36247336054 Linux108418847826 actual success attempt1; Windows108418847924
fail-closed only. All five example tests passed both OSes. Historical SOCKS cause
remains open; live final-failure diagnostic branch was not exercised.

B4 enables root-directory replacement test on Windows and adds parent plus each
cache/home/tmp replacement. Real renames, no skips: apply/cleanup must reject,
Drop must preserve foreign and displaced original data. Local diff check passed;
no local Rust. Exact head/run IDs saved in PR launch comment.

ONE next action: inspect exact-head CI and native replacement tests; diagnose red.
T066 remains ACTIVE. Process-lifetime coupling and actual Deno cache trial remain.
No runtime activation, hostile-race, Windows ACL or zero-retention claim.
