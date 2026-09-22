# NEXT

## Current verified state

T036 is complete.

Verified T036 implementation head:
`510c737f4845bd4c40c1b2a1571b116bf59bcd66`

Merged T036 main:
`dbffadb5367b190ddd04c124bfc196754fb4bd04`

## Active atomic task

**T037 — Add the typed desktop download preflight specification**

T037 is a pure-data backend boundary:

- input begins with a revalidated supported `SearchCandidate`;
- media-source resolution reuses the T036 backend-only `YtDlpMediaSourceUrl` mapping;
- Arti executable, yt-dlp executable, Tor config/cache/state directories, download output root, and SOCKS port are explicit backend inputs;
- the resulting typed Rust spec contains an `ArtiRuntimePlan`, yt-dlp executable path, typed media source, and output root;
- empty required paths and invalid SOCKS ports fail closed;
- the existing frontend response remains inert and receives no media URL, executable path, Tor directory, or output path;
- no process spawn, Tor bootstrap, yt-dlp/FFmpeg execution, external network, or filesystem output occurs.

Branch:
`task/T037-desktop-download-preflight-specification`

## Next task

T037 remains the next task until its exact head is verified green and closed.

## Do not do yet

- no external search provider;
- no AI provider;
- no real Download execution from the desktop UI;
- no Tor bootstrap from the desktop UI;
- no yt-dlp/FFmpeg process from the desktop UI;
- no packaging/release installers;
- no direct-network fallback.

## Success

Windows/Linux desktop checks and all existing privacy/media checks are green for the exact T037 head.
