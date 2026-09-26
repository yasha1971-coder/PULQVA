# NEXT

## CURRENT: T066-B3 native Windows system-directory lookup; CI pending

PR71 branch feat/T066-deno-environment. Verified head
626225107073ecdf5ac0b7112166350f26cd1fdd: all 11 workflows green.
Desktop 36243575970: Windows 108408506722 passed both cleanup tests;
Linux 108408506759 also passed replacement-preservation test.

B3 removes the arbitrary system-root input from workspace apply. Uses bounded
GetSystemWindowsDirectoryW on Windows, no inherited env fallback; Linux emits
no Windows directory variables. Actual child fixture poisons WINDIR and checks
native lookup result. Local diff check passed; native execution pending CI.
Exact new head/run IDs are in the PR launch checkpoint.

ONE next action: inspect exact-head CI/native child proof; diagnose red.
Remaining T066: Windows ownership-loss coverage, process-lifetime coupling and
actual Deno cache trial. No activation, hostile-race or zero-retention claim.
