# T007 — Raise and pin the Rust baseline to 1.91

Parent: PRIVACY FOUNDATION  
Status: DONE  
Date: 2026-09-20

## Result

Raised the PULQVA Rust baseline without changing product behavior.

- workspace `rust-version = "1.91"`;
- `rust-toolchain.toml` pins `1.91.0`;
- GitHub Actions installs and verifies exact `rustc/cargo 1.91.0`;
- edition remains 2024.

## Verification

PR #6 implementation head:
`579fd2728bf6061250119290629f9082df666877`

GitHub Actions:

- continuity-guard: success;
- rust-check: success;
- exact Rust 1.91.0 verification: success;
- `cargo test --workspace --locked`: success.
