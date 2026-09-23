# NEXT

## Current verified state

T048 is complete.

PULQVA now has async desktop completed-file command wiring:

- frontend input is only natural-language query text plus validated candidate locator;
- query is revalidated through `SearchIntent`;
- locator is revalidated against the local candidate set;
- unsupported candidates fail before any blocking/runtime work starts;
- backend constructs Arti, yt-dlp, FFmpeg, Tor config/cache/state, output-root, SOCKS port, and Tor
  readiness timeout inputs itself;
- no executable path, filesystem root, proxy/SOCKS value, media URL, argv, or process identifier is
  accepted from frontend input;
- `download_completed_file` runs the existing T047 pipeline through
  `tauri::async_runtime::spawn_blocking`;
- task-join failure maps to typed `completed-file-task-join-failed`;
- successful output is only the sanitized `CompletedFileView`;
- no duplicate network/process implementation or direct-network fallback is introduced.

Verified PR head:
`12f4306a027de390dd136c13e302be2ada6c7d1e`

All 11 required workflows passed for that exact head.

## Next atomic task

**T049 — Add an app-owned desktop runtime layout boundary**

Replace ad-hoc relative runtime path literals with one typed backend-owned layout rooted in an
application-owned runtime directory.

Required boundary:

- one typed layout owns Arti, yt-dlp, FFmpeg, Tor config/cache/state, and download output paths;
- all paths are derived from one backend-owned root;
- frontend provides no filesystem path or runtime location;
- derived paths reject parent traversal and do not escape the root;
- T048 command preparation consumes this typed layout rather than duplicating path literals;
- no process launch, network access, installer, or bundle activation is added;
- Windows/Linux path behavior remains deterministic.

## Do not do yet

- no packaging/release installers;
- no active Tauri bundle resources;
- no external search provider;
- no AI provider;
- no direct-network fallback.

## Success

The async desktop command gets every runtime path from one validated backend-owned layout, ready for a
future materialization/bundling phase without exposing filesystem control to the frontend.
