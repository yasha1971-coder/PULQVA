# T011 — Add a typed deterministic Arti configuration renderer

Parent: PRIVACY FOUNDATION  
Status: READY

## Goal

Turn the source-verified Arti 2.6.0 configuration contract into pure typed PULQVA data that can be
rendered deterministically before any runtime file-writing or process-launch code exists.

## Acceptance criteria

- `ArtiConfigSpec` exists in `pulqva-privacy`;
- SOCKS port comes from typed non-zero loopback endpoint data;
- cache directory is explicit input data;
- state directory is explicit input data;
- rendered config always sets `application.defer_bootstrap = true`;
- rendered config disables configuration watching;
- rendered config disables DNS listening;
- canonical test inputs render byte-for-byte equal to `sidecars/arti/pulqva.toml`;
- no file I/O;
- no process spawn;
- no sockets or Tor bootstrap;
- `cargo test --workspace --locked` passes;
- continuity and Arti contract checks remain green.

## Out of scope

- writing the config to disk;
- runtime process launch/supervision;
- Tor bootstrap/reachability;
- SOCKS readiness probing;
- dynamic port discovery;
- yt-dlp/FFmpeg;
- AI;
- UI.
