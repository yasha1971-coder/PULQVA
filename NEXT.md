# NEXT

## STOP / recovery checkpoint — 2026-09-20

The chat stream was interrupted. Do not resume from the old visible T004 message.
Live repository inspection shows T004 was merged as PR #3, T007 is recorded complete, and the outstanding failure is T008 / PR #7.

**T008 is BLOCKED, not DONE. Diagnosis is complete; repair and rebuild are not performed in this recovery phase.**

- Last observed main before this documentation checkpoint: `5d855e10ba7a83e80d8f0731096d8c002141ca9b` (T007).
- Existing work branch: `task/T008-arti-sidecar-build-proof`.
- PR: https://github.com/yasha1971-coder/PULQVA/pull/7 — open, unmerged at inspection.
- Failed PR head: `d08535319ce35d1166e5d17f3ff954873b02697b`.
- Run: https://github.com/yasha1971-coder/PULQVA/actions/runs/35516020963 .
- Linux job `106091982879`: completed successfully (build + version/proxy CLI checks).
- Windows job `106091982781`: failed at final linking, before CLI verification.
- Exact diagnostic: `LINK : fatal error LNK1181: cannot open input file 'sqlite3.lib'`.
- Cargo subsequently reported exit code 101. This is a Windows link/dependency failure, not evidence that the Rust compiler ran out of memory or that the PULQVA product design is impossible.
- The previous T004 locked test job `106089912014` passed all 9 tests (4 core, 5 JSON).
- `PROJECT_STATE.json` still describes the last completed T007 checkpoint. Its `ci: green` must not be interpreted as the state of PR #7.

### One next action

Prepare a Windows-only fix for the missing SQLite link dependency in the existing T008 work branch. First verify the pinned Arti feature/dependency options in the upstream source; do not assume a feature name. Keep Tor/privacy invariants and `--locked` intact. This is build setup, not a Tor runtime change.

Do not blindly rerun the failed configuration, repeat the successful Linux build merely to wait, merge PR #7, begin T009, or start a Tor connection in this phase.
For a rebuild, use a bounded CI job with a recorded run ID; limit chat status polling to two calls and return PENDING if unfinished. Recheck the current PR head before editing because another workstream may have advanced it.

### Session boundaries now apply

Read the bounded session policy in `AGENTS.md`: one task or recovery phase per response; an active orchestration target of 180 seconds; at most one new build attempt and two CI polls; checkpoint then final response and STOP. These are agent execution rules, not a claim that a ChatGPT platform watchdog has been installed.

## Current verified state

T007 is complete.

PULQVA is now pinned to Rust 1.91:

- workspace MSRV: `1.91`;
- repository toolchain: `1.91.0`;
- GitHub Actions verifies both `rustc` and `cargo` are exactly `1.91.0`;
- edition remains 2024;
- all existing workspace tests remain green at the recorded T007 checkpoint.

No runtime or product behavior changed in T007.

## Parent task (not yet complete)

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
