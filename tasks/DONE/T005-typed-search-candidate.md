# T005 — Add the typed SearchCandidate boundary

Parent: FOUNDATION  
Status: DONE  
Date: 2026-09-20

## Result

Added provider-neutral `SearchCandidate` to `pulqva-core`.

The type contains:

- a human-visible title;
- an opaque locator with no URL/command semantics in core.

Both fields reject empty and whitespace-only input.

No provider, network, runtime, URL parser, shell, ranking, or download behavior was added.

## Verification

PR #4 verified head commit:
`e14ee06114e98a14cfe6d60b48c0c040847faa5b`

GitHub Actions:

- continuity-guard: success;
- rust-check / `cargo test --workspace --locked`: success.
