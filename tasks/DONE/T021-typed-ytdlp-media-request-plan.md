# T021 — Add a typed yt-dlp media request plan

Parent: MEDIA FOUNDATION  
Status: DONE  
Date: 2026-09-21

## Result

Added typed `YtDlpMediaSourceUrl` and `YtDlpMediaRequestPlan`.

The request layer accepts explicit HTTP(S) sources only, requires a non-empty output root, preserves
the Tor-only base argv from `YtDlpLaunchPlan`, and produces deterministic request arguments without
spawning a process.

## Verification

PR #20 verified head:
`369bb883554a1bc33e15e34b2afbbfa7956c758f`

All existing checks passed.
