# NEXT

## Current verified state

T006 is complete.

`pulqva-privacy` now exposes a dependency-free `TorSocksEndpoint` capability:

- fixed loopback host `127.0.0.1`;
- non-zero port required;
- proxy representation uses `socks5h://` so DNS stays on the proxy side;
- there is no Direct/Clearnet route variant;
- the crate opens no socket and performs no network request.

GitHub Actions verified continuity and `cargo test --workspace --locked` on commit
`61fe501443c4883ea6b87a0760f2d72cf4836aa3`.

## Next atomic task

**T007 — Raise and pin the Rust baseline to 1.91**

Reason: current Arti 2.6.0 declares `rust-version = "1.91"`. PULQVA is still pinned to Rust 1.85,
so the toolchain must be upgraded before integrating the current Tor implementation.

At T007 creation time (2026-09-20):

- latest Tor Project Arti release: 2.6.0;
- `arti` 2.6.0 rust-version: 1.91;
- `arti-client` 0.46.0 rust-version: 1.91.

T007 changes only the build baseline. It does not add Arti yet.

## Do not do yet

- no Arti dependency or runtime;
- no Tor bootstrap/network;
- no yt-dlp;
- no HTTP;
- no AI provider;
- no UI.

## Success

Workspace MSRV and CI are pinned to Rust 1.91.0, all current tests remain green, continuity remains
green, and there is no product behavior change.
