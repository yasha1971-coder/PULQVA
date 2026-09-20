# T003 — Add the first typed intent boundary

Parent: FOUNDATION  
Status: READY

## Goal

Introduce the smallest typed intent boundary in `pulqva-core` so future AI or rule-based parsers
produce validated data rather than executable commands.

## Acceptance criteria

- `SearchIntent` exists in `pulqva-core`;
- construction accepts a non-empty query;
- construction rejects an empty or whitespace-only query;
- tests cover accepted and rejected inputs;
- no network/runtime dependency is introduced;
- no shell/executable command generation exists;
- `cargo test --workspace --locked` passes;
- continuity guard remains green.

## Out of scope

- AI providers;
- JSON parsing;
- search execution;
- yt-dlp;
- Tor;
- UI;
- ranking;
- download behavior.
