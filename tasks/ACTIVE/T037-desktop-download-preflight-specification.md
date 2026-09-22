# T037 — Add the typed desktop download preflight specification

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Create the smallest pure-data backend preflight specification that can later connect T036 media-source
resolution to the existing privacy/media runtime types without starting any runtime side effect.

## Acceptance criteria

- preflight input requires a revalidated supported local candidate;
- media-source resolution reuses the T036 backend-only typed boundary;
- Arti and yt-dlp executable identities/paths are explicit backend inputs;
- Tor config/cache/state directories are explicit backend inputs;
- download output root is explicit backend input;
- preflight output is deterministic typed Rust data;
- frontend receives no media URL, executable path, Tor directory, or output filesystem path;
- invalid or missing required input fails closed;
- no process spawn occurs;
- no Tor bootstrap occurs;
- no yt-dlp/FFmpeg execution occurs;
- no external network occurs;
- no filesystem output is created;
- Windows and Linux desktop checks remain green;
- all existing privacy/media checks remain green.

## Out of scope

- process launching;
- Tor readiness verification from the desktop UI;
- actual media download;
- FFmpeg execution;
- external candidate search;
- AI provider integration;
- packaging/release installers.
