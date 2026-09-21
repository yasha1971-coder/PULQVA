# T032 — Add the typed desktop intent-input boundary

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Add the first natural-language input path to the desktop shell while keeping all interpretation and
validation on the Rust side.

## Acceptance criteria

- desktop UI contains one natural-language intent input and submit action;
- frontend sends raw text only through the typed `submit_intent` Tauri command;
- Rust command constructs `pulqva_core::SearchIntent`;
- empty/whitespace-only input is rejected by the existing core invariant;
- successful response returns deterministic typed fields: validated query and stage;
- error response is typed and deterministic;
- no AI provider is contacted;
- no search provider is contacted;
- no URL or shell execution is introduced;
- frontend retains no external Internet API;
- Windows and Linux desktop checks remain green;
- all existing privacy/media checks remain green.

## Out of scope

- AI intent parsing/provider integration;
- candidate search;
- media download from UI;
- packaging/release installers.
