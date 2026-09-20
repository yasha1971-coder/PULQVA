# T008 — Prove the pinned Arti sidecar builds on Windows and Linux

Parent: PRIVACY FOUNDATION  
Status: READY

## Goal

Prove that PULQVA can bundle the current official Arti SOCKS proxy as a zero-config sidecar on
the two kernel release targets before adding any Tor runtime behavior.

## Version snapshot

Checked 2026-09-20:

- official `arti` crate: 2.6.0;
- `rust-version = "1.91"`;
- official binary exposes the `proxy` subcommand;
- Arti can run as a SOCKS proxy.

## Acceptance criteria

- repository pins the sidecar version to exactly `2.6.0`;
- CI builds/installs that exact version using `cargo install arti --version 2.6.0 --locked`;
- CI matrix covers Windows and Linux;
- binary version output contains `2.6.0`;
- `arti help proxy` exits successfully;
- no Tor bootstrap is attempted;
- no product code opens a network connection;
- existing continuity and workspace Rust checks remain green.

## Out of scope

- launching Arti from PULQVA;
- Tor bootstrap/reachability;
- runtime configuration;
- reading dynamic SOCKS port data;
- yt-dlp;
- AI;
- UI.
