# T005 — Add the typed SearchCandidate boundary

Parent: FOUNDATION  
Status: READY

## Goal

Introduce the smallest provider-neutral search-result type in `pulqva-core` so future search
adapters can return choices without leaking backend-specific URL/command semantics into the core.

## Contract

`SearchCandidate` contains:

- `title`: human-visible display title;
- `locator`: opaque backend locator passed as data, not executable text.

## Acceptance criteria

- valid title + locator constructs a candidate;
- title and locator are readable;
- empty/whitespace-only title is rejected;
- empty/whitespace-only locator is rejected;
- no URL parsing semantics are introduced into core;
- no provider/network/runtime dependency is introduced;
- no shell/executable command generation exists;
- `cargo test --workspace --locked` passes;
- continuity guard remains green.

## Out of scope

- search providers;
- yt-dlp;
- URLs as a core concept;
- ranking;
- AI;
- Tor;
- UI;
- download execution.
