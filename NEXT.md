# NEXT

## Current verified state

T037 is complete.

Verified T037 implementation head:
`a82f326d82f117a6053f86bbde5d1eb7d23f29a2`

Merged T037 main:
`9cac8b97ec0ee8990853a49c48cc232aa6aa03fc`

## Active atomic task

**T038 — Add the backend-only prepared download runtime boundary**

T038 converts the typed T037 preflight into a prepared backend runtime state:

- input is a verified `DownloadPreflightSpec`;
- Arti configuration materialization reuses `prepare_arti_runtime`;
- prepared state retains `PreparedArtiRuntime`, explicit yt-dlp executable path, typed `YtDlpMediaSourceUrl`, and output root;
- preparation errors fail closed and return no prepared runtime;
- only the Arti configuration is materialized; no sidecar executable, cache/state directory, media output, or download output is created by this boundary;
- frontend behavior and IPC output remain unchanged, so no backend paths or media URL are exposed;
- no Arti/yt-dlp/FFmpeg process is spawned;
- no Tor bootstrap/readiness probe or external network access occurs.

Branch:
`task/T038-prepared-download-runtime-boundary`

## Next task

T038 remains the next task until its exact head is verified green and closed.

## Do not do yet

- no external search provider;
- no AI provider;
- no actual media download from the desktop UI;
- no Arti process launch from the desktop UI;
- no yt-dlp/FFmpeg execution from the desktop UI;
- no packaging/release installers;
- no direct-network fallback.

## Success

Windows/Linux desktop checks and all existing privacy/media checks are green for the exact T038 head.
