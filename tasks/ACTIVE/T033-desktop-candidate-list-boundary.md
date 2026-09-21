# T033 — Add the typed desktop candidate-list boundary

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Add the first provider-neutral candidate response path to the desktop backend and UI using the
existing `pulqva_core::SearchCandidate` type.

## Acceptance criteria

- typed desktop candidate DTO exists;
- candidate title/locator values originate only from validated `SearchCandidate`;
- Rust command validates the incoming intent with `SearchIntent`;
- desktop Rust command returns a deterministic ordered candidate list;
- this task uses only a local/test candidate source;
- frontend renders candidate titles and opaque locators only as inert text;
- frontend does not interpret, navigate, select, or execute candidate locators;
- no search provider is contacted;
- no AI provider is contacted;
- no URL/shell/media execution is introduced;
- frontend retains no external Internet API;
- Windows and Linux desktop checks remain green;
- all existing privacy/media checks remain green.

## Out of scope

- external candidate search;
- AI provider integration;
- candidate selection/download execution;
- packaging/release installers.
