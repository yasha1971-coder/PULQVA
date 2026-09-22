# NEXT

## Current verified state

T043 is complete.

Verified T043 recovery head:
`3c67ce313e29a291138f97a75c9bbae85eb9d207`

Merged T043 main:
`677a1125a709d15c0ffdd71e6a962980e340b4c4`

## Active atomic task

**T044 — Add the backend-only controlled FFmpeg remux execution boundary**

T044 advances a verified local remux plan into a controlled backend-owned FFmpeg child:

- input is a verified `FfmpegRemuxPlan`;
- launch reuses the existing typed `launch_ffmpeg_remux` boundary;
- executable and argv come only from the typed plan;
- no shell command string is constructed;
- no URL, proxy, SOCKS endpoint, or network-capable field is introduced;
- launch failure is mapped to a typed fail-closed backend error;
- the resulting private `RunningFfmpegRemuxRuntime` owns the live `RunningFfmpeg` child;
- explicit backend cleanup reuses `RunningFfmpeg::stop_and_wait`;
- frontend IPC/output remains unchanged and exposes no executable path, input/output filesystem path,
  argv, source URL, or process identifier;
- no external network access occurs.

Branch:
`task/T044-controlled-ffmpeg-remux-execution-boundary`

## Next task

T044 remains next until its exact head is verified green and closed.

## Do not do yet

- no completed remux result surfacing to frontend;
- no external search provider;
- no AI provider;
- no packaging/release installers;
- no direct-network fallback.

## Success

Windows/Linux desktop checks and all existing privacy/media checks are green for the exact T044 head.
