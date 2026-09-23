# T049 — Add an app-owned desktop runtime layout boundary

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Centralize every backend runtime filesystem path behind one typed application-owned layout so the
T048 command no longer depends on duplicated relative path literals.

## Acceptance criteria

- one typed layout owns the runtime root;
- Arti executable, yt-dlp executable, FFmpeg executable, Tor config, Tor cache, Tor state, and download output paths are all derived from that root;
- frontend cannot provide or override the runtime root or any derived path;
- the root must not be empty;
- derived paths contain no parent-directory traversal and remain rooted beneath the application-owned root;
- T048 command preparation consumes the typed layout;
- existing Tor-first and local-only FFmpeg behavior is unchanged;
- no process is launched by the layout itself;
- no external network access is introduced;
- no installer or Tauri bundle activation is added;
- Windows and Linux desktop checks remain green;
- existing privacy/media/remux checks remain green.

## Out of scope

- runtime binary materialization;
- packaging/release installers;
- Tauri bundle resources;
- external candidate search;
- AI provider integration.
