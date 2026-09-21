# T032 — Add the typed desktop intent-input boundary

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-21

## Result

Added the first natural-language desktop input path.

The frontend sends raw text only through `submit_intent`; Rust constructs
`pulqva_core::SearchIntent`; the existing core invariant rejects blank input; successful responses
contain only deterministic validated query/stage fields. No network, provider, URL, shell, or media
side effect was introduced.

## Verification

PR #31 verified head:
`29193748fe3352684c30d52a94bc5b09ba421e11`

All existing checks passed, including `desktop-shell-check`.
