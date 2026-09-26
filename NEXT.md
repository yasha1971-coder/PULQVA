# NEXT

## CURRENT: T066-C real pinned Deno runtime workspace trial; CI pending

PR71 branch feat/T066-deno-environment. Verified head
b6c713089fa8bf6c6ee63dad36c7a74ca12ecaf2: all11 workflows green.
Desktop36256139529 Linux108443175567 / Windows108443175715 passed B5 lifetime tests.

C adds native ignored test using existing authenticated Deno2.9.7 fixture and
materialize_deno. Fixed no-import script with no permission grants must reach
exit42/43; post-exit observer records cache entry count then verifies workspace
cleanup and executable/script preservation. CI step explicitly invokes both OSes.
Local diff check passed; no local Rust. New head/run IDs saved in PR checkpoint.

ONE next action: inspect exact-head CI and PULQVA_DENO_RUNTIME_WORKSPACE_OK;
diagnose red. Zero cache entries are allowed and must not imply observed analysis
cache writes. No live EJS, OS egress or descendant confinement claim; no activation.
