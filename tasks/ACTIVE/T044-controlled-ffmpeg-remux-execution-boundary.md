# T044 — Add the backend-only controlled FFmpeg remux execution boundary

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Launch a verified T043 local remux plan through the existing controlled FFmpeg process boundary
without shell execution or any network-capable path.

## Acceptance criteria

- input is a verified `FfmpegRemuxPlan`;
- FFmpeg launch reuses the existing `launch_ffmpeg_remux` boundary;
- no shell command string is constructed;
- executable and argv come only from the typed plan;
- no URL, proxy, SOCKS endpoint, or external-network field is introduced;
- launch failure fails closed;
- the running FFmpeg child remains owned by the backend until explicit completion or cleanup;
- frontend receives no executable path, input/output filesystem path, argv, source URL, or process identifier;
- frontend behavior remains unchanged;
- no external network access occurs;
- Windows and Linux desktop checks remain green;
- all existing privacy/media checks remain green.

## Out of scope

- completed remux result surfacing to frontend;
- external candidate search;
- AI provider integration;
- packaging/release installers.
