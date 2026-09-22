# NEXT

## Current verified state

T044 is complete.

PULQVA now has a backend-only controlled FFmpeg remux execution boundary:

- input is a verified `FfmpegRemuxPlan`;
- launch reuses the existing typed `launch_ffmpeg_remux` boundary;
- executable and argv come only from the typed plan;
- no shell command string is constructed;
- no URL, proxy, SOCKS endpoint, or network-capable field is introduced;
- launch failure maps to a typed fail-closed backend error;
- the private `RunningFfmpegRemuxRuntime` owns the live `RunningFfmpeg` child;
- explicit backend cleanup reuses `RunningFfmpeg::stop_and_wait`;
- frontend IPC/output remains unchanged and exposes no executable path, input/output filesystem path,
  argv, source URL, or process identifier;
- no external network access occurs.

Verified PR head:
`a1031df085949eef0df246a70dd2fecd9c434a0e`

All 11 required workflows passed for that exact head.

## Next atomic task

**T045 — Add the backend-only completed FFmpeg remux result boundary**

Advance one running T044 FFmpeg remux runtime through deterministic process completion and validate
the local remux artifact before it can become a backend result.

Required boundary:

- input is a T044 `RunningFfmpegRemuxRuntime`;
- FFmpeg completion waits for the owned child and fails closed on non-zero exit;
- the expected output path comes only from the verified T043 remux plan, never from frontend input;
- successful completion validates that the output exists, is a regular non-symlink file, is non-empty,
  and is distinct from the validated input artifact;
- completion returns a typed backend-only remux result;
- no shell, URL, proxy, SOCKS endpoint, or network field is introduced;
- frontend receives no executable path, input/output filesystem path, argv, source URL, process
  identifier, or raw completion internals;
- no external network access occurs.

## Do not do yet

- no completed remux result surfacing to frontend;
- no external search provider;
- no AI provider;
- no packaging/release installers;
- no direct-network fallback.

## Success

A running local FFmpeg remux can complete into a validated typed backend result while preserving
local-file-only execution and fail-closed artifact validation.
