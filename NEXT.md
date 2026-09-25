# NEXT

## CURRENT: T063M metadata failure diagnostic launch

Base main cdd2299e450161ed2842f654878125e5ce125c51: 9/10 checks green.
Arti compiler recovery passed. Linux metadata workflow 36177458014,
job 108211426176 exited 1 without retained stderr. Cause unknown.
PR #67 checkpoint 5839441794 records diagnosis.

Branch recovery/T063M-metadata-diagnostics adds opt-in bounded failure-only
stderr to the fixed public metadata example and enables it in CI. Production
launcher, fixture URL, Tor gate, timeout and success criteria remain unchanged.
Rust execution pending in CI; local Cargo unavailable.

ONE next action: observe this branch's exact-head checks and inspect metadata
failure receipt if red. If successful, record not-reproduced status without
claiming root cause fixed, then close the diagnostic task on exact-head evidence.
Head/PR/run IDs recorded in PR launch comment. T064 READY but not started.
