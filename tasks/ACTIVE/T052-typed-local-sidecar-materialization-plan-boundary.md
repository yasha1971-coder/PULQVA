# T052 — Add a typed local sidecar materialization plan boundary

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Define one typed backend-owned plan that maps verified packaged sidecar sources to the app-owned
`runtime/bin/*` destinations without copying any binary yet.

## Acceptance criteria

- plan covers exactly Arti, yt-dlp, and FFmpeg;
- destination paths come only from the verified `AppRuntimeLayout`;
- all destinations remain beneath the runtime `bin` child;
- source resource paths are backend-owned constants/metadata and are not accepted from frontend input;
- source and destination paths must differ;
- no shell command string is constructed;
- no process is launched;
- no external network access occurs;
- no binary is copied, downloaded, or modified;
- Windows and Linux desktop checks remain green;
- existing privacy/media/remux checks remain green.

## Out of scope

- actual sidecar binary materialization;
- active Tauri bundle resources;
- packaging/release installers;
- external candidate search;
- AI provider integration.
