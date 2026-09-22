# NEXT

## Current verified state

T044 is complete.

Merged T044 main:
`99a95d73886b0057e80fecacc78c507f37ddea43`

## Active atomic task

**T045 — Add the backend-only completed FFmpeg remux result boundary**

T045 completes the backend-owned FFmpeg process into a validated typed local result:

- `RunningFfmpeg` retains the verified plan input/output paths at launch;
- `RunningFfmpeg::complete_remux` waits for process completion and fails closed on non-zero exit;
- successful process completion validates the expected output through a dedicated local artifact boundary;
- output must exist, be a regular non-symlink file, be non-empty, and remain distinct from the validated input;
- completion returns `CompletedFfmpegRemuxResult`;
- process exit, artifact validation, and wait I/O failures remain typed and distinguishable;
- the T044 `RunningFfmpegRemuxRuntime` now has a backend-only completion boundary returning that typed result;
- the pinned real-remux proof exercises this completed-result path;
- frontend IPC/output remains unchanged and exposes no executable path, input/output filesystem path,
  argv, source URL, process identifier, or raw completion internals;
- no shell or external network path is introduced.

Branch:
`task/T045-completed-ffmpeg-remux-result-boundary`

## Next task

T045 remains next until its exact head is verified green and closed.

## Do not do yet

- no completed remux result surfacing to frontend;
- no external search provider;
- no AI provider;
- no packaging/release installers;
- no direct-network fallback.

## Success

The exact T045 head passes desktop/privacy checks and the dedicated real FFmpeg remux proof on
Windows and Linux.
