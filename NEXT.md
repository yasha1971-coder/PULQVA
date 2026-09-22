# NEXT

## Current verified state

T046 is complete.

PULQVA now has a sanitized desktop completed-file view boundary:

- input is a verified `CompletedFfmpegRemuxResult`;
- output contains only a display filename, byte size, and stable `completed-file-ready` stage;
- filename is derived only from the validated output file-name component;
- missing/empty filename fails closed;
- control characters and path separators are replaced in display text;
- no absolute or parent path is serialized;
- no executable path, argv, source URL, proxy/SOCKS data, process identifier, or raw backend result internals are serialized;
- frontend command wiring remains unchanged;
- no process or external network access is introduced.

Verified PR head:
`2812dec01ed08b94a35f0c7b42561f8dccf689ff`

All 11 required workflows passed for that exact head.

## Next atomic task

**T047 — Add a backend-only one-shot completed-file orchestration boundary**

Compose the already verified desktop/backend boundaries into one backend-only path from a validated
candidate through Tor download, completed download validation, local FFmpeg remux, completed remux
validation, and sanitized completed-file view.

Required boundary:

- input remains a validated local candidate plus explicit backend runtime inputs;
- reuse existing T037–T046 typed boundaries rather than duplicating process/network logic;
- Tor remains required before yt-dlp launch and there is no direct-network fallback;
- yt-dlp completion and Arti cleanup remain fail-closed;
- FFmpeg receives only the validated local artifact through the typed remux plan;
- FFmpeg completion returns the validated typed remux result;
- final output is only the T046 sanitized completed-file view;
- every failure path returns before any later phase starts;
- no backend path, executable path, argv, source URL, proxy/SOCKS data, process identifier, or raw
  completion internals cross the output boundary;
- no Tauri command/frontend wiring yet.

## Do not do yet

- no live desktop command wiring;
- no external search provider;
- no AI provider;
- no packaging/release installers;
- no direct-network fallback.

## Success

One backend-only function can drive the verified download-to-file pipeline to a sanitized completed
file value while preserving Tor-first, local-only FFmpeg, and fail-closed invariants.
