# T041 — Add the backend-only controlled yt-dlp media execution boundary

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Launch a T040 Tor-gated media request through the existing controlled yt-dlp process boundary while
retaining backend ownership of the running Arti child.

## Acceptance criteria

- input is a T040 `TorGatedMediaRequestRuntime`;
- yt-dlp launch reuses the existing `launch_ytdlp_request` boundary;
- no shell command string is constructed;
- the running Arti child remains owned by the resulting backend state;
- yt-dlp launch failure stops and waits for Arti before returning failure;
- cleanup failure is surfaced separately and fail-closed;
- frontend receives no executable path, filesystem path, SOCKS endpoint, proxy URL, media URL, argv, or process identifier;
- frontend may receive only inert running/stage data;
- no direct-network fallback exists;
- FFmpeg is not started;
- completion/result handling is not added;
- Windows and Linux desktop checks remain green;
- all existing privacy/media checks remain green.

## Out of scope

- yt-dlp completion/result receipt;
- FFmpeg execution;
- external candidate search;
- AI provider integration;
- packaging/release installers.
