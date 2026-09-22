# T044 — Add the backend-only controlled FFmpeg remux execution boundary

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-22

## Result

Added a backend-only controlled FFmpeg remux execution boundary.

A verified `FfmpegRemuxPlan` can now be launched through the existing typed
`launch_ffmpeg_remux` boundary. Executable and argv come only from the typed plan, with no shell
command string, URL, proxy, SOCKS endpoint, or network-capable field introduced.

The resulting private `RunningFfmpegRemuxRuntime` owns the live `RunningFfmpeg` child until
explicit backend cleanup. Launch and cleanup failures map to typed fail-closed backend errors.

Frontend IPC/output is unchanged and exposes no executable path, input/output filesystem path, argv,
source URL, or process identifier. No external network access is introduced.

## Verification

PR #44 verified head:
`a1031df085949eef0df246a70dd2fecd9c434a0e`

All 11 required workflows passed:

- continuity-guard;
- rust-check;
- desktop-shell-check;
- arti-sidecar-check;
- arti-materialization-check;
- arti-config-contract;
- arti-lifecycle-check;
- tor-readiness-check;
- ytdlp-sidecar-check;
- ytdlp-tor-metadata-check;
- ytdlp-tor-media-check.
