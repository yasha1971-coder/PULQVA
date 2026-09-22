# T045 — Add the backend-only completed FFmpeg remux result boundary

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-23

## Result

Added a backend-only completed FFmpeg remux result boundary.

`RunningFfmpeg` now retains the verified input/output paths from the typed remux plan and exposes
`complete_remux`, which waits for the owned FFmpeg child and fails closed on non-zero exit.

Successful process completion validates the expected output through a dedicated local artifact
boundary. The output must exist, be a regular non-symlink file, be non-empty, and remain distinct
from the validated input. Successful completion returns `CompletedFfmpegRemuxResult`.

Process exit failure, artifact validation failure, and wait I/O failure remain typed and
distinguishable. The desktop backend T044 runtime now completes through this result boundary.

Frontend IPC/output remains unchanged and exposes no executable path, input/output filesystem path,
argv, source URL, process identifier, or raw completion internals. No shell or external network path
is introduced.

## Verification

PR #45 verified head:
`e6d17692de4a164c3e202bd307a4afcdf1502216`

All 12 triggered workflows passed, including the dedicated real FFmpeg remux proof.
