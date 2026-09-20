# T010 — Verify the pinned Arti configuration contract

Parent: PRIVACY FOUNDATION  
Status: READY

## Goal

Confirm and prove the exact Arti 2.6.0 configuration contract PULQVA will use before any product
runtime code writes or launches that configuration.

## Acceptance criteria

- exact Arti 2.6.0 keys are verified from upstream source/documentation;
- a deterministic config fixture exists;
- cache directory is explicit;
- state directory is explicit;
- SOCKS listener is loopback-only;
- pinned Arti accepts/parses the fixture on Windows and Linux;
- validation does not start the proxy or bootstrap Tor;
- no product process-spawn code is introduced;
- existing Rust and continuity checks remain green.

## Out of scope

- runtime process launch;
- Tor bootstrap/reachability;
- SOCKS readiness probing;
- dynamic port discovery;
- yt-dlp/FFmpeg;
- AI;
- UI.
