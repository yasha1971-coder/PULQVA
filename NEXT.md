# NEXT

## Current verified state

T045 is complete.

PULQVA now has a backend-only completed FFmpeg remux result boundary:

- `RunningFfmpeg` retains the verified remux input/output paths from the typed plan;
- `RunningFfmpeg::complete_remux` waits for process completion and fails closed on non-zero exit;
- successful process completion validates the expected local output;
- output must exist, be a regular non-symlink file, be non-empty, and remain distinct from the validated input;
- successful completion returns typed `CompletedFfmpegRemuxResult`;
- process exit, artifact validation, and wait I/O failures remain distinguishable;
- the desktop backend completion boundary consumes the T044 runtime and returns the typed remux result;
- the dedicated real FFmpeg remux proof exercises this completed-result path;
- frontend IPC/output remains unchanged and exposes no executable path, input/output filesystem path,
  argv, source URL, process identifier, or raw completion internals;
- no shell or external network path is introduced.

Verified PR head:
`e6d17692de4a164c3e202bd307a4afcdf1502216`

All 12 triggered workflows passed for that exact head, including `ffmpeg-real-remux-check`.

## Next atomic task

**T046 — Add a sanitized desktop completed-file view boundary**

Convert a verified T045 `CompletedFfmpegRemuxResult` into a data-only desktop success value that can
later be surfaced to the UI without exposing backend filesystem/runtime internals.

Required boundary:

- input is a verified `CompletedFfmpegRemuxResult`;
- output contains only a safe display filename, byte size, and stable completion stage;
- filename is derived only from the validated completed output path;
- no absolute or parent filesystem path is serialized;
- no executable path, argv, source URL, proxy/SOCKS data, process identifier, or raw result internals
  are serialized;
- frontend command wiring remains out of scope;
- no new process or external network access occurs.

## Do not do yet

- no live desktop command wiring for completed results;
- no external search provider;
- no AI provider;
- no packaging/release installers;
- no direct-network fallback.

## Success

A verified completed remux can be converted into a deterministic data-only desktop success value
without exposing backend paths or runtime internals.
