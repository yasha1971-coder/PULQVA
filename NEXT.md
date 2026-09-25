# NEXT

## CURRENT: T061 compatibility assessment proposed

Base main b89990d34e227e19e5669b2ddfa900eb5e4237c1 verified: all 10 triggered
push workflows successful. PRs #62 and #63 merged. Desktop was path-filtered.

Branch: task/T061-js-runtime-assessment. Exact SHA/PR/check IDs are in the PR
checkpoint after publication. This change remains CI pending.

ADR-0006 and sidecars/deno/CANDIDATE.json propose a Deno 2.9.7 trial with
yt-dlp 2026.08.19/EJS 0.8.0. No runtime dependency was enabled.
Upstream hashes are not locally recomputed; executable EJS payload inspection
remains pending. T061 is ACTIVE, not DONE.

Both pinned yt-dlp executable payloads now match EJS 0.8.0 core/lib hashes;
see sidecars/deno/EJS_INSPECTION.json and scripts/inspect_ytdlp_ejs.py.
This supersedes the payload-inspection pending statement above. Deno archive
hashes remain upstream-only. Parent b52a820 passed all 10 triggered workflows.

One next action: review the updated PR's exact-head checks and assessment
before closing T061. Diagnose red checks first. Do not merge using
older checks or start launch wiring in this assessment.

Windows positive live media retrieval remains unproven; original Linux exit-1
root cause remains unknown. Deno execution, isolation, extracted size, Linux
minimum ABI and YouTube trial are untested.

Follow-up atomic tasks after assessment: verified packaging, typed launch boundary,
then bounded Tor-only trial. Full user journey remains the product goal.
