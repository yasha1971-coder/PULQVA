# T033 — Add the typed desktop candidate-list boundary

Parent: DESKTOP FOUNDATION  
Status: READY

## Goal

Add the first provider-neutral candidate response path to the desktop backend and UI using the
existing `pulqva_core::SearchCandidate` type.

## Acceptance criteria

- typed desktop candidate DTO exists;
- candidate title/locator values originate from validated `SearchCandidate`;
- desktop Rust command returns a deterministic candidate list for a validated intent;
- this task uses only a local/test candidate source;
- frontend renders candidate titles and opaque locators without interpreting or executing locators;
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
