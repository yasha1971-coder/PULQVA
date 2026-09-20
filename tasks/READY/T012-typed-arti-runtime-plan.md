# T012 — Add a typed Arti runtime plan

Parent: PRIVACY FOUNDATION  
Status: READY

## Goal

Compose the verified launch and configuration specs into one pure-data runtime plan before any
side effects exist.

## Acceptance criteria

- `ArtiRuntimePlan` exists in `pulqva-privacy`;
- executable path is explicit;
- config-file path is explicit;
- one typed `TorSocksEndpoint` is shared by the plan and config renderer;
- cache/state paths are supplied once and cannot drift between launch metadata and config;
- launch arguments are deterministic;
- rendered configuration is deterministic;
- canonical inputs still match the verified fixture;
- no file I/O;
- no process spawn;
- no sockets or Tor bootstrap;
- `cargo test --workspace --locked` passes;
- continuity and Arti contract checks remain green.

## Out of scope

- writing configuration to disk;
- runtime process launch/supervision;
- Tor bootstrap/reachability;
- readiness probing;
- dynamic port discovery;
- yt-dlp/FFmpeg;
- AI;
- UI.
