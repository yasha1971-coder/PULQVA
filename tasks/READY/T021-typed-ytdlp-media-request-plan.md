# T021 — Add a typed yt-dlp media request plan

Parent: MEDIA FOUNDATION  
Status: READY

## Goal

Add pure-data source and output request data on top of YtDlpLaunchPlan without spawning yt-dlp.

## Acceptance criteria

- typed media source input exists;
- source scheme is explicitly constrained to supported network URL forms;
- output root is explicit;
- request construction requires YtDlpLaunchPlan;
- base Tor proxy arguments are preserved unchanged;
- request arguments are deterministic;
- no shell string is produced;
- no process is spawned;
- no direct/clearnet fallback exists;
- tests cover invalid source schemes and deterministic arguments;
- all existing checks remain green.

## Out of scope

- spawning yt-dlp;
- actual media download;
- FFmpeg;
- AI;
- UI.
