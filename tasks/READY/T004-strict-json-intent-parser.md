# T004 — Add strict JSON -> SearchIntent parsing

Parent: FOUNDATION  
Status: READY

## Goal

Create the smallest strict JSON decoding boundary for intent data without selecting or calling an
AI provider.

## Acceptance criteria

- a valid object of the form `{"query":"..."}` produces a validated `SearchIntent`;
- malformed JSON is rejected;
- missing `query` is rejected;
- unknown fields are rejected;
- empty and whitespace-only `query` values are rejected through the existing core validation;
- no network/runtime client is introduced;
- no provider-specific code is introduced;
- no shell/executable command generation exists;
- `cargo test --workspace --locked` passes;
- continuity guard remains green.

## Out of scope

- Duck.ai or any AI provider;
- HTTP;
- Tor/Arti;
- yt-dlp/FFmpeg;
- search execution;
- ranking;
- UI;
- download behavior.
