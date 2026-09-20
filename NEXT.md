# NEXT

## Current verified state

T002 is complete.

The repository now has a minimal Rust workspace with `crates/pulqva-core`.
GitHub Actions verified both:

- continuity guard: green;
- `cargo test --workspace --locked`: green on commit
  `08ed4f4d1f3f0d8cd7d2efe7f33e402e93f84f45`.

No UI, network, Tor, AI, yt-dlp, ranking, or packaging code exists yet.

## Next atomic task

**T003 — Add the first typed intent boundary**

Add only a minimal `SearchIntent` type to `pulqva-core` with explicit validation that rejects
an empty/whitespace-only query.

This establishes the first safe data boundary without introducing any provider, parser, network
client, or executable command generation.

## Do not do yet

- no Tauri;
- no frontend framework/bundler;
- no Arti/Tor;
- no AI provider;
- no yt-dlp/FFmpeg;
- no shell command generation;
- no search execution.

## Success

`SearchIntent` is typed, blank queries are rejected by tests, non-empty queries are accepted,
`cargo test --workspace --locked` passes, and continuity remains green.
