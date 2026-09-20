# NEXT

## Current verified state

T007 is complete.

PULQVA is now pinned to Rust 1.91:

- workspace MSRV: `1.91`;
- repository toolchain: `1.91.0`;
- GitHub Actions verifies both `rustc` and `cargo` are exactly `1.91.0`;
- edition remains 2024;
- all existing workspace tests remain green.

No runtime or product behavior changed.

## Next atomic task

**T008 — Prove the pinned Arti sidecar builds on Windows and Linux**

Use the official `arti` binary package at exactly version `2.6.0`.

The task is build/CLI proof only:

- build/install `arti 2.6.0` from crates.io with its lockfile;
- verify the binary reports version 2.6.0;
- verify the `proxy` command is present;
- run the check on GitHub-hosted Windows and Linux;
- do not bootstrap Tor and do not open Tor network connections.

This validates the zero-config sidecar direction before PULQVA writes launcher/runtime code.

## Why sidecar first

The official `arti` binary already implements a SOCKS proxy, while future PULQVA consumers such
as yt-dlp require a SOCKS endpoint. Keeping Arti as a separately pinned sidecar also lets Tor be
upgraded independently from the PULQVA core.

This is not yet a permanent runtime lock-in; the decision becomes durable only after the proof is
green and recorded as an ADR.

## Do not do yet

- no Tor bootstrap/network;
- no process launcher in product code;
- no yt-dlp;
- no HTTP;
- no AI provider;
- no UI.

## Success

Arti 2.6.0 builds and its proxy CLI is verified on Windows and Linux in CI, while the existing
continuity and Rust workspace checks remain green.
