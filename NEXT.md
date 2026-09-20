# NEXT

## Current verified state

T003 is complete.

The core now exposes a dependency-free `SearchIntent` boundary:

- non-empty natural-language queries are accepted;
- empty and whitespace-only queries are rejected;
- the type is data-only;
- no network/runtime dependency exists;
- no shell or executable command generation exists.

GitHub Actions verified continuity and `cargo test --workspace --locked` on commit
`7e6483c544b5df421fb0e9c0f57afe076adea0b9`.

## Next atomic task

**T004 — Add strict JSON -> SearchIntent parsing**

Add a minimal JSON decoding boundary that converts a strict JSON object into `SearchIntent`
without choosing or calling any AI provider.

The JSON contract must reject unknown fields and still enforce the existing non-blank query
validation.

## Do not do yet

- no AI provider;
- no HTTP client;
- no Tor/Arti;
- no yt-dlp/FFmpeg;
- no search execution;
- no shell command generation;
- no UI;
- no ranking/download logic.

## Success

A valid JSON object produces `SearchIntent`; blank queries, malformed JSON, and unknown fields
are rejected by tests; `cargo test --workspace --locked` and continuity guard remain green.
