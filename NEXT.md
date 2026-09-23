# NEXT

## Current verified state

T055 is complete and merged on main at
`2d99206da86ab64412516d26b762bc01cc9a6771`.

Verified T055 closeout head:
`c40ad5d6feb52d308c5d9c7b3febb642c5b3cdeb`.

All 11 required workflows passed for that exact closeout head.

## Active atomic task

**T056 — Wire verified yt-dlp materialization into backend prelaunch preparation**

T056 connects the T055 direct-binary materializer to the real completed-file backend flow without
expanding scope to Arti or FFmpeg materialization.

Implemented ordering:

1. frontend query/locator are validated into backend-owned candidate/pipeline inputs;
2. Tauri AppHandle resolves the package resource plan;
3. blocking backend validates all packaged sidecar sources;
4. exactly one verified yt-dlp artifact is selected;
5. app-owned runtime directories are prepared;
6. T055 atomically materializes that verified yt-dlp artifact;
7. the returned materialized path must equal both the runtime layout yt-dlp destination and the
   already backend-derived completed-file input path;
8. only then may the existing completed-file pipeline proceed toward Arti/Tor/yt-dlp launch.

Missing or duplicate verified yt-dlp artifacts fail before runtime-directory mutation and before the
materializer callback. A wrong materialized path fails before pipeline launch.

No frontend filesystem path is accepted. No shell command, new network operation, direct-network
fallback, Arti executable materialization, FFmpeg archive extraction, Tauri bundle activation,
installer, external provider, or AI provider is added.

Tests use an internal materializer callback only to prove ordering/fail-closed behavior without
pretending fixture bytes are the repository-pinned yt-dlp binary. The production path calls the real
T055 materializer with backend-pinned identity metadata.

Branch:
`task/T056-wire-ytdlp-materialization-into-backend-prelaunch`

## Next action

Inspect the exact T056 implementation head and all triggered CI. Repair only a concrete failure, or
close T056 if all required checks are green.

Do not start another task in the same response.
