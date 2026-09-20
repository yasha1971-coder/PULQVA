# T002 — Create the minimal Rust workspace

Parent: FOUNDATION  
Status: DONE  
Date: 2026-09-20

## Result

Created:

- root Cargo workspace;
- `crates/pulqva-core` library crate;
- pinned Rust 1.85 / edition 2024 baseline;
- dependency-free smoke test;
- GitHub Actions Rust check.

## Verification

PR head commit:
`08ed4f4d1f3f0d8cd7d2efe7f33e402e93f84f45`

Verified in GitHub Actions:

- continuity guard: success;
- `cargo test --workspace --locked`: success.

No runtime/network dependency was introduced.
