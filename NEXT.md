# NEXT

## Current verified state

T004 is complete.

The project now has a strict provider-neutral JSON intent adapter in
`crates/pulqva-intent-json`.

Verified behavior:

- `{"query":"..."}` decodes into validated `SearchIntent`;
- malformed JSON is rejected;
- missing `query` is rejected;
- unknown fields are rejected;
- blank query values are rejected by the existing core validation;
- no AI provider, network client, Tor, yt-dlp, UI, or shell execution exists.

The generated dependency lockfile is committed and the normal locked CI path is restored.
GitHub Actions verified continuity and `cargo test --workspace --locked` on commit
`b875c99ddf97804155b084224854be3bf6a17cd1`.

## Next atomic task

**T005 — Add the typed SearchCandidate boundary**

Add a provider-neutral candidate type to `pulqva-core` containing:

- a human-visible title;
- an opaque locator understood by a future search/download adapter.

Both fields must reject empty/whitespace-only values.

## Do not do yet

- no search provider;
- no yt-dlp;
- no URL semantics in core;
- no AI provider;
- no HTTP;
- no Tor/Arti;
- no UI;
- no ranking or download execution.

## Success

A valid candidate can be constructed and read; blank title/locator values are rejected by tests;
`cargo test --workspace --locked` and continuity remain green.
