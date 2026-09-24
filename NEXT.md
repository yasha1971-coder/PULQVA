# NEXT

## Current verified state

T058 is complete and merged on main at
`0ca4264e47cb9d8c5a464cc93e2b43561e602710`.

Verified T058 closeout head:
`d02793b5d9d2e11fb9c627b88f69dacdce4b85f5`.

All 11 required workflows passed for that exact closeout head.

## Active atomic task

**T059 — Wire verified Arti materialization into backend prelaunch preparation**

Phase: IMPLEMENT. T059 is NOT DONE.

The real completed-file backend now preserves the existing T056 yt-dlp materialization step and adds
T058 Arti materialization before the existing pipeline can prepare or launch Tor.

Implemented ordering:

1. frontend query/locator are validated into backend-owned candidate/pipeline inputs;
2. Tauri AppHandle resolves the package resource plan;
3. blocking backend validates packaged sidecar sources;
4. existing T056 prelaunch materializes yt-dlp and binds its path to the runtime layout and backend
   completed-file input;
5. exactly one verified Arti artifact is selected;
6. app-owned runtime directories are prepared/reused;
7. T058 atomically materializes Arti;
8. the materialized Arti path must equal both `runtime/bin/arti(.exe)` and the backend-derived
   completed-file Arti input path;
9. only then is the existing completed-file pipeline invoked; Tor runtime preparation/launch stays
   downstream of this boundary.

The orchestration helper is covered without launching processes. Tests prove:

- yt-dlp materialization happens before Arti materialization;
- Arti materialization happens before the pipeline;
- Arti materialization failure prevents the pipeline closure from running;
- missing/duplicate verified Arti artifacts fail before the Arti materializer callback;
- runtime directories exist before the Arti materializer callback;
- a non-layout Arti materialized path fails closed.

No frontend filesystem path is accepted. No FFmpeg extraction/materialization, new network operation,
shell command, direct-network fallback, bundle activation, installer, external provider, or AI
provider is added.

Branch:
`task/T059-wire-arti-materialization-into-backend-prelaunch`

## Immediate next action

Inspect the exact T059 implementation head and all triggered CI. If any check fails, diagnose only
that concrete failure. If all required Windows/Linux/privacy checks are green, close T059 in a later
bounded phase.

Do not start T060.


## Recovery after first T059 implementation CI

Implementation head `f7dad20ceaff5100e2ce2858d1e161099ed17a70` produced 10/11 green workflows.
Only `desktop-shell-check` failed, and both Windows/Linux jobs failed at test-module compilation
before desktop boundary tests.

Exact error: `DownloadActionError` was referenced by the new Arti-failure test closure but omitted
from the test module import list. Production behavior was not implicated.

This recovery changes only that test import. No production path, ordering, Tor behavior, privacy
boundary, or materialization logic is changed.
