# T033 — Add the typed desktop candidate-list boundary

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-22

## Result

Added the first provider-neutral candidate-list path to the desktop shell.

Rust validates the intent, constructs deterministic proof candidates only through
`pulqva_core::SearchCandidate`, and returns title + opaque locator data. The frontend renders those
fields as inert text and performs no provider, URL, shell, media, or network action.

## Verification

PR #32 verified head:
`9b09501d6e8332437ec5583c5899961f41e08be5`

All existing checks passed, including `desktop-shell-check`.
