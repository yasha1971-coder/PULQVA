# T052 — Add typed bundled-sidecar materialization specification

Parent: DESKTOP FOUNDATION  
Status: READY

## Goal

Define the backend-only pure-data contract that maps the already pinned Arti, yt-dlp, and FFmpeg
sidecars from future bundle resources to the verified app-owned `runtime/bin/*` destinations.

## Acceptance criteria

- input requires a prepared app-runtime-directory capability;
- Arti, yt-dlp, and FFmpeg each receive a typed destination beneath `runtime/bin/*`;
- platform-specific executable filenames are explicit for Windows and Linux;
- pinned version/hash identity is carried as typed data from repository sidecar manifests;
- resource/source identifiers are backend-owned and never accepted from frontend input;
- destinations remain beneath the verified runtime root;
- no file is copied, written, downloaded, chmodded, or launched;
- no Tauri bundle resource is activated yet;
- no external network access occurs;
- Windows and Linux desktop checks remain green;
- existing privacy/media/remux checks remain green.

## Out of scope

- copying/materializing executable bytes;
- Tauri bundle-resource activation;
- process launch;
- packaging/release installers;
- external candidate search;
- AI provider integration.
