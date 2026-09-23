# T056 — Wire verified yt-dlp materialization into backend prelaunch preparation

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-23

## Result

The real completed-file backend now invokes the verified T055 yt-dlp materializer before any yt-dlp
process can launch.

Verified ordering:

1. backend validates query/locator and derives app-owned runtime paths;
2. Tauri AppHandle resolves the package resource plan;
3. packaged sidecar sources are validated;
4. exactly one verified yt-dlp artifact is selected;
5. app-owned runtime directories are prepared;
6. T055 materializes the verified yt-dlp direct binary;
7. the materialized path must equal both the runtime layout destination and the pre-derived
   completed-file yt-dlp input;
8. only then may the existing completed-file pipeline proceed toward process launch.

Missing or duplicate verified yt-dlp artifacts fail before runtime-directory mutation and before
materializer invocation. A mismatched materialized path fails before pipeline launch.

No frontend filesystem authority, shell command, new network operation, direct-network fallback,
Arti executable materialization, FFmpeg extraction, bundle activation, installer, external provider,
or AI provider was introduced.

## Verification

Verified implementation head:
`c06539668b8d9dc94ea8e61138b5ab144a56623f`

All 11 required workflows passed for that exact head, including Windows and Linux desktop tests.
