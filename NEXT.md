# NEXT

## CURRENT: T066-B2 owned workspace cleanup; CI pending

PR71 branch feat/T066-deno-environment. Verified head
8bc24d62b2d0a5c2dfcf9552b18a7465c8b71766: all 11 workflows green.
Desktop run 36242334885, Windows 108405062986 / Linux 108405063013:
actual child environment probe passed with PULQVA_DENO_CHILD_ENVIRONMENT_OK.

B2 adds owned home/tmp/cache workspace, Unix 0700 directories, bounded cleanup
with identity checks and rejection of links/special entries. Tests cover cache
cleanup, error-return cleanup and Unix replacement preservation. Local diff check
passed; no local Rust compiler. Exact head/run IDs saved in PR checkpoint.

ONE next action: inspect exact-head CI and cleanup test evidence; diagnose red.
T066 remains ACTIVE. Windows replacement coverage, native system-directory
lookup, process-lifetime coupling and actual Deno cache trial remain. No default
activation or race-free sandbox claim. Cleanup failures may retain local data.
