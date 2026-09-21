# T034 — Add the typed desktop candidate-selection boundary

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-22

## Result

Added explicit candidate selection to the local desktop shell.

Selection sends only validated intent text plus an opaque locator. Rust reconstructs the same
deterministic candidate set and requires an exact validated locator match. Unknown locators fail
closed. The returned selection remains inert data and triggers no network, provider, shell,
filesystem, URL, or media execution.

## Verification

PR #33 verified head:
`adad7423dab41387ca70d483bbedc466b50f1bf5`

All existing checks passed after pinning the metadata proof to the immutable T024 media object.
