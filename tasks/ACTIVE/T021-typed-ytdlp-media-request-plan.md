# T021 — Add a typed yt-dlp media request plan

Parent: MEDIA FOUNDATION  
Status: ACTIVE

## Goal

Add pure-data source and output request data on top of `YtDlpLaunchPlan` without spawning yt-dlp.

## Acceptance criteria

- typed `YtDlpMediaSourceUrl` exists;
- source is constrained to explicit HTTP(S) network URL forms;
- local/file and unsupported schemes are rejected;
- non-empty authority is required;
- whitespace/control characters are rejected;
- output root is explicit and non-empty;
- request construction requires `YtDlpLaunchPlan`;
- base Tor proxy arguments are preserved unchanged;
- request arguments are deterministic;
- no shell string is produced;
- no process is spawned;
- no direct/clearnet fallback exists;
- tests cover invalid source forms and deterministic arguments;
- all existing checks remain green.

## Out of scope

- spawning yt-dlp;
- actual media download;
- FFmpeg;
- AI;
- UI.
