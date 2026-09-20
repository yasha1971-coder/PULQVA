# T002 — Create the minimal Rust workspace

Parent: FOUNDATION  
Status: READY

## Goal

Create the smallest Rust workspace that establishes PULQVA's code foundation without choosing
or implementing UI, networking, Tor, AI, search, or download subsystems.

## Acceptance criteria

- root `Cargo.toml` defines a workspace;
- `crates/pulqva-core` exists as a library crate;
- the crate contains a minimal non-network smoke test;
- `cargo test --workspace` passes;
- continuity guard remains green;
- no external network/runtime dependency is introduced.

## Out of scope

- Tauri;
- frontend framework/bundler;
- Arti/Tor;
- HTTP clients;
- AI providers;
- yt-dlp/FFmpeg;
- result ranking;
- packaging.
