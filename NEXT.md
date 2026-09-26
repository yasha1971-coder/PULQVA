# NEXT

## CURRENT: T067 PREPARE — first request -> choice -> file vertical slice

Verified base: main@46776b8c1f274113319edcdc8d7bedb15d0ebfb9, post-merge
20/20 checks success. T066 is complete within its documented scope.

Anti-drift target is the kernel primary journey:
natural-language request -> privacy-gated interpretation/search -> real candidate
choices -> explicit selection -> one-click retrieval -> predictable local file.
No account/API key, no frontend Internet, Tor by default, fail closed.

T067 is intentionally orchestration, not another infrastructure subsystem.
First acceptance is deterministic/local so orchestration failures are separable from
network variance. It must use typed existing boundaries and must never be described
as proof of live external search/download. The next gate after T067 is bounded live
Tor-backed evidence through the same coordinator.

ONE next action: inspect the existing typed intent, candidate/search and media
retrieval interfaces, then implement the smallest coordinator and one integration
acceptance test. Stop after launching exact-head CI.
