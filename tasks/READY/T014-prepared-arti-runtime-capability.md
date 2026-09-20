# T014 — Add a prepared Arti runtime capability

Parent: PRIVACY FOUNDATION  
Status: READY

## Goal

Make successful configuration materialization a type-level prerequisite for any future Arti
process launcher.

## Acceptance criteria

- public `PreparedArtiRuntime` type exists;
- callers cannot directly construct it from raw fields;
- successful preparation materializes config first and only then returns the capability;
- the capability owns or carries the immutable `ArtiRuntimePlan`;
- it exposes the typed `TorSocksEndpoint`;
- it exposes deterministic launch arguments;
- failed materialization returns no prepared capability;
- tests prove preparation writes exact config bytes;
- no process spawn;
- no sockets or Tor bootstrap;
- Windows and Linux materialization checks remain green;
- all existing checks remain green.

## Out of scope

- Arti process launch/supervision;
- Tor bootstrap/reachability;
- SOCKS readiness probing;
- dynamic port discovery;
- yt-dlp/FFmpeg;
- AI;
- UI.
