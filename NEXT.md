# NEXT

## Current verified state

T047 is complete.

Merged T047 main:
`3e3773183f0f14c24f0714815e0541f5ab4e49ae`

## Active atomic task

**T048 — Add async desktop completed-file command wiring**

T048 wires the verified T047 pipeline into the desktop shell while preserving backend ownership:

- frontend command input is only natural-language query text plus candidate locator;
- query is revalidated through `SearchIntent`;
- locator is revalidated against the local candidate set;
- unsupported candidates fail before a blocking/runtime task starts;
- backend constructs Arti, yt-dlp, FFmpeg, Tor config/cache/state, output-root, SOCKS port, and Tor
  readiness timeout inputs itself;
- no executable path, filesystem root, proxy/SOCKS value, media URL, argv, or process identifier is
  accepted from frontend input;
- `download_completed_file` is async and moves the existing T047 synchronous pipeline into
  `tauri::async_runtime::spawn_blocking`;
- task-join failure maps to typed `completed-file-task-join-failed`;
- success returns only the sanitized `CompletedFileView`;
- no duplicate network/process implementation or direct-network fallback is introduced.

Branch:
`task/T048-async-desktop-completed-file-command-wiring`

## Next task

T048 remains next until its exact head is verified green and closed.

## Do not do yet

- no external search provider;
- no AI provider;
- no packaging/release installers;
- no direct-network fallback.

## Success

Desktop/privacy/media/remux checks are green for the exact T048 head and the async command exposes only
query + candidate locator on input and sanitized completed-file data on output.
