# T020 — Add a typed yt-dlp launch plan gated by ReadyTorTransport

Parent: MEDIA FOUNDATION  
Status: READY

## Goal

Create a pure-data yt-dlp launch plan whose network route can only come from a verified
`ReadyTorTransport`.

## Acceptance criteria

- typed launch-plan API exists;
- construction requires `ReadyTorTransport`;
- generated proxy argument comes only from `ReadyTorTransport::proxy_url()`;
- no direct/clearnet route variant exists;
- executable path is explicit;
- argument ordering is deterministic;
- no shell command string is produced;
- no process is spawned;
- no media URL is requested;
- unit tests prove the Tor-only proxy invariant;
- all existing checks remain green.

## Out of scope

- spawning yt-dlp;
- downloading media;
- FFmpeg;
- AI;
- UI.
