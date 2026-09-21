# NEXT

## Current verified state

T032 is complete.

PULQVA now accepts a natural-language request in the local desktop shell:

- frontend sends raw text only through the typed `submit_intent` Tauri command;
- Rust constructs the existing `pulqva_core::SearchIntent`;
- blank/whitespace-only input is rejected by the core invariant;
- successful submission returns deterministic typed query/stage data;
- no AI provider, search provider, URL execution, shell execution, or frontend Internet access exists.

Verified PR head:
`29193748fe3352684c30d52a94bc5b09ba421e11`

## Active atomic task

**T033 — Add the typed desktop candidate-list boundary**

T033 adds the first provider-neutral candidate response path:

- Rust validates the request as `SearchIntent` before producing candidates;
- local proof candidates are constructed only through `pulqva_core::SearchCandidate`;
- a typed desktop DTO copies only validated title + opaque locator fields;
- candidate order and values are deterministic;
- the source is local/test-only and contacts no provider;
- frontend renders title/locator fields only as inert text;
- opaque locators are not interpreted, navigated, selected, or executed.

## Queued next task

**T034 — Add the typed desktop candidate-selection boundary**

Allow the user to select one already-returned candidate through a typed local command. Validate the
selection against the same local candidate set and return only deterministic selection data. Do not
execute the locator or start any media/network action yet.

## Do not do yet

- no external search provider;
- no AI provider;
- no locator execution;
- no media download from UI;
- no packaging/release installers;
- no direct-network fallback.

## Success

A validated intent can produce a deterministic provider-neutral candidate list whose opaque locators
remain inert data all the way through the desktop UI.
