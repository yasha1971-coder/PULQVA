# NEXT

## Current verified state

T040 is complete.

Verified T040 implementation head:
`021994420b8e3ee393785e6dc4ef3685150efdd6`

Merged T040 main:
`5aed554436378b874cd62550b002a15efc03f10c`

## Active atomic task

**T041 — Add the backend-only controlled yt-dlp media execution boundary**

T041 advances a verified T040 Tor-gated media request into a controlled running media state:

- input is a `TorGatedMediaRequestRuntime`;
- yt-dlp launch reuses the existing `launch_ytdlp_request` typed process boundary;
- the resulting backend state owns both the live Arti child and the running yt-dlp child;
- yt-dlp launch failure explicitly stops and waits for Arti before returning;
- cleanup failure is surfaced separately and fail-closed;
- coordinated shutdown stops both yt-dlp and Arti and reports either or both cleanup failures;
- no shell command string is constructed;
- frontend IPC/output remains unchanged and exposes no executable path, filesystem path, SOCKS/proxy information, media URL, argv, or process identifiers;
- no direct-network fallback exists;
- FFmpeg and completion/result handling remain out of scope.

Branch:
`task/T041-controlled-ytdlp-media-execution-boundary`

## Next task

T041 remains the next task until its exact head is verified green and closed.

## Do not do yet

- no external search provider;
- no AI provider;
- no FFmpeg execution from the desktop UI;
- no completion receipt/result surfacing;
- no packaging/release installers;
- no direct-network fallback.

## Success

Windows/Linux desktop checks and all existing privacy/media checks are green for the exact T041 head.
