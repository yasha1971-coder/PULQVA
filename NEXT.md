# NEXT

## Current verified state

T031 is complete.

PULQVA now has a compiling Windows/Linux Tauri 2 desktop shell:

- Tauri 2.11.6 and tauri-build 2.6.3 are pinned;
- the Rust backend links `pulqva-core`;
- the frontend loads only local assets;
- frontend status access crosses only the typed Tauri command boundary;
- CSP grants local content plus required Tauri IPC endpoints only;
- CI rejects frontend fetch/XHR/WebSocket/EventSource/sendBeacon and remote HTTP(S) URLs;
- Windows and Linux desktop compile checks are green.

Verified PR head:
`6faff7d194d87e714ae3b50d34834223a34309b1`

## Active atomic task

**T032 — Add the typed desktop intent-input boundary**

The desktop shell now accepts one natural-language request:

- the frontend submits only the raw text through `submit_intent`;
- Rust constructs the existing `pulqva_core::SearchIntent`;
- blank/whitespace-only requests are rejected by the core invariant;
- successful submissions return deterministic typed data: validated query + stage;
- the UI shows only the local validation result;
- no AI provider, search provider, URL execution, shell execution, or frontend Internet access is introduced.

## Queued next task

**T033 — Add the typed desktop candidate-list boundary**

Add a provider-neutral candidate response contract to the desktop backend and UI using
`pulqva_core::SearchCandidate`. Keep the source local/test-only in this task; no search provider or
external request yet.

## Do not do yet

- no AI provider;
- no external search provider;
- no media download from UI;
- no arbitrary user URL execution;
- no packaging/release installers;
- no direct-network fallback.

## Success

A user's raw text can cross the local desktop boundary, be validated by the core `SearchIntent`
invariant, and return deterministic typed data without any network side effect.
