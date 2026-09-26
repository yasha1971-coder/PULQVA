# NEXT

## CURRENT: T065-A archive adapter pending CI

Base main 66f2b27f356c3e2b26dbfe96f70190d9b21b746d verified: all 11 applicable
workflows green, including Arti run 36219805072. PR69 merged; T064 DONE.
Branch feat/T065-deno-materialization. Exact new head/PR/run IDs recorded in PR.

T065-A adds private pinned Deno identities and authenticated immutable ZIP extraction
with archive and executable size/SHA checks, strict single-member policy and bounded
writer. Tests registered in the separate desktop workspace, whose desktop-shell-check
runs native Windows/Linux tests. Local Rust unavailable; compilation/tests pending CI.

ONE next action: inspect exact-head CI; diagnose red first. After A passes, implement
T065-B source snapshot and owned staging in a later phase. T065-C must test actual
pinned archives through the Rust adapters on both OSes before T065 is DONE.

No filesystem authority or launch authorization follows from the byte receipt.
Caller must discard partial writer output on error. No default Deno activation,
remote fetch, environment/cache/egress confinement or YouTube proof. Prelaunch
replacement-race revalidation remains required. Historical Linux failures, unpinned
SDK content and unproven positive Windows live media remain unresolved.
