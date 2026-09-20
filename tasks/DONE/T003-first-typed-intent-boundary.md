# T003 — Add the first typed intent boundary

Parent: FOUNDATION  
Status: DONE  
Date: 2026-09-20

## Result

Added a dependency-free `SearchIntent` type in `pulqva-core`.

Behavior:

- accepts non-empty natural-language input;
- rejects empty input;
- rejects whitespace-only input;
- preserves the original non-blank query;
- exposes no provider, network, shell, or command-execution behavior.

## Verification

PR #2 head commit:
`7e6483c544b5df421fb0e9c0f57afe076adea0b9`

GitHub Actions:

- continuity-guard: success;
- rust-check / `cargo test --workspace --locked`: success.
