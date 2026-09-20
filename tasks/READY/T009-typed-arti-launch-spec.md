# T009 — Add a typed Arti sidecar launch specification

Parent: PRIVACY FOUNDATION  
Status: READY

## Goal

Create the smallest pure-data specification for launching the pinned Arti sidecar later, without
spawning a process or opening any network connection.

## Acceptance criteria

- a typed launch-spec type exists;
- executable path/identity is explicit input data;
- proxy subcommand arguments are deterministic;
- state/cache/config directory inputs are explicit data;
- the type does not spawn processes;
- the type does not open sockets or bootstrap Tor;
- tests cover deterministic argument construction;
- `cargo test --workspace --locked` passes;
- continuity guard remains green.

## Out of scope

- process spawning;
- runtime supervision;
- Tor bootstrap;
- SOCKS readiness probing;
- dynamic port discovery;
- yt-dlp/FFmpeg;
- AI;
- UI.
