# T034 — Add the typed desktop candidate-selection boundary

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Allow one already-returned local candidate to be selected through a typed Rust command while keeping
its opaque locator inert.

## Acceptance criteria

- frontend exposes one explicit Select action per displayed candidate;
- selection sends only validated intent text plus opaque candidate locator through Tauri IPC;
- Rust reconstructs the same deterministic local candidate set;
- requested locator must exactly match one validated `SearchCandidate`;
- unknown locators fail closed;
- typed selection response contains deterministic intent/title/locator/stage fields;
- frontend renders selection result as inert text;
- no locator interpretation or execution exists;
- no search provider, AI provider, shell, URL, filesystem, or media process is invoked;
- frontend retains no external Internet API;
- Windows and Linux desktop checks remain green;
- all existing privacy/media checks remain green.

## Out of scope

- external candidate search;
- download execution;
- Autopilot selection logic;
- packaging/release installers.
