# T022 — Add a controlled yt-dlp child-process launcher

Parent: MEDIA FOUNDATION  
Status: DONE  
Date: 2026-09-21

## Result

Added `RunningYtDlp` and `launch_ytdlp_request`.

The launcher accepts only `YtDlpMediaRequestPlan`, spawns the explicit executable directly with
the plan's deterministic argv, uses no shell, and provides deterministic stop/wait cleanup.

Windows/Linux tests use only local fixtures. A local fake Arti/SOCKS fixture certifies the transport
without external network, and a yt-dlp fixture records the exact argv without contacting the URL.

## Verification

PR #21 verified head:
`be07ecbaba2dd3027b3592ba0a4c191739e9c0e1`

All existing checks passed.
