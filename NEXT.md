# NEXT

## Current verified state

T030 is complete.

PULQVA now proves one real bounded local FFmpeg stream-copy remux:

- exact pinned FFmpeg/ffprobe archives are verified before execution;
- the immutable local MP4 fixture is verified before use;
- input enters through the validated `CompletedDownloadResult` boundary;
- real execution starts only from `FfmpegRemuxPlan`;
- `-protocol_whitelist file` keeps FFmpeg input local;
- `-c copy` preserves stream-copy behavior with no re-encoding;
- non-media hint/data tracks are dropped explicitly;
- Windows and Linux complete the same real-remux proof;
- pinned ffprobe validates the resulting local Matroska file.

Verified PR head:
`378ef54cb5bf9cec834e40f826a89b5ffb0913fa`

## Active atomic task

**T031 — Bootstrap the Tauri 2 desktop shell**

T031 adds the first Windows/Linux desktop shell:

- Tauri is pinned to 2.11.6 and tauri-build to 2.6.3;
- the desktop crate is isolated from the root Rust workspace so existing locked core checks remain unchanged;
- the Rust backend links `pulqva-core` and exposes one typed `app_status` command;
- the frontend calls only Tauri IPC through `window.__TAURI__.core.invoke`;
- frontend assets are entirely local;
- CSP permits only local content plus the Tauri IPC endpoints;
- CI rejects fetch/XHR/WebSocket/EventSource/sendBeacon and remote HTTP(S) URLs in frontend code;
- Windows and Linux compile checks are defined.

## Queued next task

**T032 — Add the typed desktop intent-input boundary**

Add the first user text input to the local desktop UI and send it only through a typed Tauri command
that constructs `SearchIntent` in Rust. No AI provider, search backend, URL execution, or frontend
Internet access yet.

## Do not do yet

- no AI provider;
- no external search from the frontend;
- no arbitrary user URL execution;
- no packaging/release installers;
- no direct-network fallback.

## Success

PULQVA has a compiling Windows/Linux Tauri 2 shell whose frontend is local-only and can learn app
status solely through the typed Rust command boundary.
