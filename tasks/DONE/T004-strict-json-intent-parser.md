# T004 — Add strict JSON -> SearchIntent parsing

Parent: FOUNDATION  
Status: DONE  
Date: 2026-09-20

## Result

Created `crates/pulqva-intent-json`, a narrow provider-neutral adapter.

The adapter uses a strict JSON wire shape with unknown-field rejection and delegates semantic
query validation to `pulqva-core::SearchIntent`.

No network/provider/execution behavior was added.

## Verification

PR #3 verified head commit:
`b875c99ddf97804155b084224854be3bf6a17cd1`

GitHub Actions:

- continuity-guard: success;
- rust-check / `cargo test --workspace --locked`: success;
- 4 core tests + 5 JSON adapter tests passed.

`Cargo.lock` was generated on the pinned Rust CI runner, committed, and the temporary diagnostic
workflow step was removed before merge.
