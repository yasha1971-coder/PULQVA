# NEXT

## CURRENT: T065M bounded Tor media recovery pending CI

PR70 branch feat/T065-deno-materialization. Base d5ade9b91d7dbeb6559a8556b7ba3a53186a5f33
failed Linux media run 36232028938 job 108376857791 with generic SOCKS REP 1 before
download. Other 11 workflows, including Deno/firewall, passed. Last all-green head
remains 810abe3e9778105ad1952833c67ad7b9eff7339e.

Fix: narrowly classified pre-download SOCKS failure allows at most 3 attempts with
2/4-second backoff inside the SAME 180-second budget. Same typed Tor route and URL.
No partial-output retry, no timeout reset or direct fallback. Attempt logs retained.
Unit tests cover budget/count/classification; actual native tests pending CI.
Research and precise limitations: tasks/ACTIVE/T065M-bounded-media-recovery.md.

ONE next action: inspect exact-head Rust tests and real media result/attempt count.
If red diagnose exact logs; if green record whether retry branch was exercised,
then complete recovery. Do not represent successful first-attempt CI as proof an
external network outage is fixed. T066 remains unstarted, PR70 not merge-ready.
