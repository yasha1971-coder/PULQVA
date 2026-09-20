# T013 — Add atomic Arti config materialization

Parent: PRIVACY FOUNDATION  
Status: READY

## Goal

Introduce the first controlled filesystem side effect by materializing the deterministic Arti
configuration from `ArtiRuntimePlan` to its explicit config path safely.

## Acceptance criteria

- config bytes come only from `ArtiRuntimePlan::render_config`;
- only the config parent directory is created as needed;
- write occurs through a temporary sibling file;
- temporary contents are flushed/synced before final replacement;
- final config replacement is atomic where supported by the platform;
- stale temporary files are cleaned up on handled failures;
- rendered bytes are preserved exactly;
- Windows and Linux tests cover materialization behavior;
- no Arti process spawn;
- no sockets or Tor bootstrap;
- existing Rust, continuity, Arti config and sidecar checks remain green.

## Out of scope

- Arti runtime process launch/supervision;
- Tor bootstrap/reachability;
- SOCKS readiness probing;
- dynamic port discovery;
- yt-dlp/FFmpeg;
- AI;
- UI.
