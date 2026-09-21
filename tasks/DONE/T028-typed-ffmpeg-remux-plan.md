# T028 — Add a typed local FFmpeg remux plan

Parent: MEDIA FOUNDATION  
Status: DONE  
Date: 2026-09-21

## Result

Added `FfmpegRemuxPlan` with typed MP4/Matroska containers.

The plan accepts input only from `CompletedDownloadResult`, requires explicit executable/output
paths, rejects output/input equality and parent traversal, and produces deterministic local-only
stream-copy argv. No process, shell, URL, proxy, or network route is introduced.

## Verification

PR #27 verified head:
`33c5ef60429f88842967a1677256e2f44d5aaf96`

All existing checks passed.
