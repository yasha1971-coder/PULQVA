# NEXT

## CURRENT: T066-B5 test-only process/workspace lifetime; CI pending

PR71 branch feat/T066-deno-environment. Verified head
1379f1167990126c62c0d75b41e15d18883348ec: all11 workflows green.
Desktop36249829776 Windows108425656837 / Linux108425656974: all five cache
ownership/cleanup tests passed, with native prevention distinguished from
successful replacement detection.

B5 introduces test-only single-child execution retaining workspace until observed
termination, bounded timeout kill/reap grace, and retention on unknown termination.
Two native tests invoke success/nonzero/timeout fixtures plus spawn-failure check.
Local diff check passed; no local Rust. Exact launch head/run IDs in PR comment.

ONE next action: inspect exact-head CI and B5 native test evidence; diagnose red.
After green, run actual pinned Deno cache trial with this test boundary and no
permission to spawn descendants. Production tree lifetime/egress remain separate
activation gates. No app launcher change or runtime activation; historical Tor
cause and positive Windows live retrieval remain unresolved.
