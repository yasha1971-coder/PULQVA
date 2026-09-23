# NEXT

## Current verified state

T047 is complete.

PULQVA now has a backend-only one-shot completed-file orchestration boundary:

- input is a validated `SearchCandidate` plus explicit backend runtime inputs;
- preflight, Arti preparation/readiness, Tor-gated yt-dlp request planning/launch, completed download,
  FFmpeg remux planning/launch/completion, and sanitized completed-file conversion all reuse the
  previously verified T037–T046 boundaries;
- Tor remains mandatory before yt-dlp launch;
- there is no direct-network fallback;
- yt-dlp completion and Arti cleanup remain fail-closed;
- FFmpeg receives only the validated local artifact and remains local-file-only;
- every failed phase returns before any later phase starts;
- final output is only the sanitized `CompletedFileView`;
- no Tauri command/frontend wiring exists yet;
- no backend filesystem path, executable path, argv, source URL, proxy/SOCKS data, process identifier,
  or raw completion internals cross the output boundary.

Verified PR head:
`7c56f08b62fc6110c2167398fbf9f012e46856c7`

All 11 required workflows passed for that exact head.

## Next atomic task

**T048 — Add async desktop completed-file command wiring**

Expose the verified T047 pipeline to the desktop shell without accepting backend runtime paths from
frontend code and without blocking the UI thread.

Required boundary:

- frontend input remains only the natural-language query plus validated candidate locator;
- query and locator are revalidated before any runtime work starts;
- backend constructs all runtime paths/settings itself;
- no executable path, filesystem root, proxy/SOCKS data, media URL, argv, or process identifier is
  accepted from frontend input;
- execution runs off the UI thread through a bounded backend blocking task;
- backend task calls the existing T047 one-shot pipeline rather than duplicating network/process logic;
- successful command output is only the sanitized T046 `CompletedFileView`;
- task-join failure is surfaced as a typed fail-closed desktop error;
- no direct-network fallback exists.

## Do not do yet

- no external search provider;
- no AI provider;
- no packaging/release installers;
- no direct-network fallback.

## Success

A desktop command can trigger the verified Tor-first download-to-file pipeline from query + candidate
locator and return only a sanitized completed-file value without blocking the UI thread or exposing
backend runtime inputs.
