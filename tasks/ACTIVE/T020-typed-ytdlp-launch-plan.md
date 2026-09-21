# T020 — Add a typed yt-dlp launch plan gated by ReadyTorTransport

Parent: MEDIA FOUNDATION  
Status: ACTIVE

## Goal

Create a pure-data yt-dlp launch plan whose network route can only come from a verified
ReadyTorTransport.

## Acceptance criteria

- typed YtDlpLaunchPlan API exists;
- construction requires ReadyTorTransport;
- generated proxy argument comes only from ReadyTorTransport::proxy_url();
- no direct/clearnet route variant exists;
- executable path is explicit;
- argument ordering is deterministic;
- --ignore-config prevents ambient yt-dlp config from overriding routing;
- no shell command string is produced;
- no process is spawned;
- no media URL is requested;
- unit tests prove IPv4/IPv6 Tor-only proxy rendering;
- all existing checks remain green.

## Out of scope

- spawning yt-dlp;
- downloading media;
- FFmpeg;
- AI;
- UI.
