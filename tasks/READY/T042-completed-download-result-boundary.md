# T042 — Add the backend-only completed download result boundary

Parent: DESKTOP FOUNDATION  
Status: READY

## Goal

Complete a T041 running media runtime through the existing typed yt-dlp completion and artifact
validation path while deterministically cleaning up the live Arti child.

## Acceptance criteria

- input is a T041 `RunningMediaDownloadRuntime`;
- yt-dlp completion reuses `RunningYtDlp::complete_download`;
- successful yt-dlp completion yields the existing `CompletedDownloadResult`;
- Arti is stopped and waited after yt-dlp completion on success and failure paths;
- yt-dlp completion failure fails closed;
- Arti cleanup failure fails closed;
- combined completion and cleanup failure preserves both failure causes;
- frontend receives no executable path, filesystem path, SOCKS endpoint, proxy URL, media URL, argv, process identifier, or raw completion internals;
- frontend behavior remains unchanged;
- no FFmpeg process is started;
- no direct-network fallback exists;
- Windows and Linux desktop checks remain green;
- all existing privacy/media checks remain green.

## Out of scope

- frontend completed-result surfacing;
- FFmpeg execution;
- external candidate search;
- AI provider integration;
- packaging/release installers.
