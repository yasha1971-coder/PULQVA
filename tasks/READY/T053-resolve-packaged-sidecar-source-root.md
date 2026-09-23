# T053 — Resolve packaged sidecar source root from the Tauri resource directory

Parent: DESKTOP FOUNDATION  
Status: READY

## Goal

Resolve the filesystem source paths for the T052 logical packaged sidecar resources from the
desktop application's own Tauri resource directory, without copying executable bytes.

## Acceptance criteria

- Tauri `AppHandle` is the only authority for the package resource root;
- frontend input supplies no resource/filesystem path;
- T052 logical resource identifiers are joined beneath the resolved resource root;
- resource paths containing parent traversal or escaping the resource root fail closed;
- plan still covers exactly Arti, yt-dlp, and FFmpeg;
- pinned version/hash identities are preserved unchanged;
- runtime destinations remain under the verified `runtime/bin/*`;
- output is typed source filesystem paths plus the existing destination/identity data;
- no file is copied, written, downloaded, chmodded, or launched;
- no external network access occurs;
- Windows and Linux desktop tests remain green;
- existing privacy/media/remux checks remain green.

## Out of scope

- actual sidecar binary materialization;
- installer/release packaging;
- external candidate search;
- AI provider integration.
