# T029 — Add a controlled FFmpeg child-process launcher

Parent: MEDIA FOUNDATION  
Status: DONE  
Date: 2026-09-21

## Result

Added `RunningFfmpeg` and `launch_ffmpeg_remux`.

The launcher accepts only `FfmpegRemuxPlan`, directly spawns the explicit executable with the
plan's exact argv, uses no shell, and provides deterministic process status and stop/wait cleanup.
Windows/Linux tests use a network-free fixture that records argv and creates no media output.

## Verification

PR #28 verified head:
`0e11a4612a334b777ee7161db4bea010f043185f`

All existing checks passed.
