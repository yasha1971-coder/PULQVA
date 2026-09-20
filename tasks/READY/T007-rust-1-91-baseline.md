# T007 — Raise and pin the Rust baseline to 1.91

Parent: PRIVACY FOUNDATION  
Status: READY

## Goal

Prepare PULQVA to integrate the current supported Arti release without pinning the project to an
obsolete Tor implementation.

## Rationale snapshot

Checked 2026-09-20:

- Tor Project released Arti 2.6.0 on 2026-09-01;
- `arti` 2.6.0 declares `rust-version = "1.91"`;
- `arti-client` 0.46.0 declares `rust-version = "1.91"`.

Official/current references should be rechecked when Arti is actually added, because Arti evolves
rapidly.

## Acceptance criteria

- workspace `rust-version` is `1.91`;
- repository contains `rust-toolchain.toml` pinned to `1.91.0`;
- GitHub Actions installs/tests with Rust `1.91.0`;
- edition remains 2024;
- `cargo test --workspace --locked` passes;
- continuity guard remains green;
- no product/runtime behavior changes.

## Out of scope

- Arti dependency;
- Tor bootstrap;
- network traffic;
- yt-dlp;
- UI;
- AI.
