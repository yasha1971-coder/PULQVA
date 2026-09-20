# NEXT

## Current verified state

T001 is complete.

The continuity/kernel scaffold is published in `main`. The continuity guard passed in GitHub
Actions for commit `ba39fc6e2757120c5daf38edccbfaee5a5de3f6e`.

Kernel v1 is the durable product contract. The completion workflow creates `kernel-v1.0.0`
once T001 is recorded as complete; later CI rejects ordinary changes to `kernel/`.

## Next atomic task

**T002 — Create the minimal Rust workspace**

Create only the smallest compiling Rust foundation:

- root Cargo workspace;
- `crates/pulqva-core`;
- one smoke test proving the workspace builds and tests.

## Do not do yet

- no Tauri;
- no React/Vite/Farm;
- no Arti/Tor implementation;
- no AI provider;
- no yt-dlp;
- no network dependency.

## Success

`cargo test --workspace` passes and the repository remains continuity-green.

After T002, update state and create the next atomic task before adding another subsystem.
