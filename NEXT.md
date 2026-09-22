# NEXT

## Current verified state

T041 is complete.

PULQVA now has a backend-only controlled yt-dlp media execution boundary:

- input is a verified T040 `TorGatedMediaRequestRuntime`;
- yt-dlp launch reuses the existing typed `launch_ytdlp_request` boundary;
- the resulting backend state owns both the live Arti child and the running yt-dlp child;
- yt-dlp launch failure explicitly stops and waits for Arti before returning;
- coordinated shutdown stops both yt-dlp and Arti and surfaces cleanup failures fail-closed;
- no shell execution or direct-network fallback exists;
- frontend IPC/output remains unchanged and exposes no executable path, filesystem path, SOCKS/proxy information, media URL, argv, or process identifiers;
- FFmpeg and completion/result handling remain out of scope.

Verified PR head:
`e9aa2b87dc6b8e88807a35ab464628ef5f5b2487`

All 11 required workflows passed for that exact head.

## Next atomic task

**T042 — Add the backend-only completed download result boundary**

Advance one running T041 media runtime through the existing yt-dlp completion and artifact-validation path while retaining deterministic cleanup of Arti.

Required boundary:

- input is a T041 `RunningMediaDownloadRuntime`;
- yt-dlp completion reuses the existing typed `complete_download` boundary;
- successful completion returns the existing `CompletedDownloadResult`;
- Arti is stopped and waited after yt-dlp completion, on both success and failure paths;
- yt-dlp completion failure and Arti cleanup failure are surfaced distinctly and fail closed;
- frontend receives no backend path, proxy/SOCKS data, media URL, argv, process identifier, or raw completion internals;
- FFmpeg remains out of scope;
- no direct-network fallback exists.

## Do not do yet

- no external search provider;
- no AI provider;
- no FFmpeg execution from the desktop UI;
- no completed-result surfacing to frontend yet;
- no packaging/release installers;
- no direct-network fallback.

## Success

A running media download can complete through the existing typed artifact-validation path and produce a
backend-only `CompletedDownloadResult` while Arti cleanup remains deterministic and fail-closed.
