# NEXT

## CURRENT: B4 Windows rename-contract recovery; CI pending

PR71 branch feat/T066-deno-environment. Verified anchor remains
711b294c6c3c3d07ce1ec6528a717bd02e7565f8.
Head0d0c489 desktop36248605915 Windows108422309634 failed at root/parent
rename with code5 PermissionDenied before any substitution. Actual cache/home/tmp
replacement tests passed; Linux108422309479 passed all tests.

Only test expectations changed: Windows code5 requires identity intact and
normal apply/cleanup success; completed replacement requires fail-closed refusal
and foreign-data preservation. Other errors fail. No production handle changes.
Native prevention is not post-replacement detection. Local diff check passed;
no local Rust. New head/run IDs saved in PR launch checkpoint.

ONE next action: inspect exact-head native test results; diagnose red.
T066 remains ACTIVE: process lifetime and actual Deno cache trial still required.
Historical SOCKS cause remains open; no runtime activation or sandbox claim.
