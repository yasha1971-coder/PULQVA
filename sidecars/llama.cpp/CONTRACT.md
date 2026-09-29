# PULQVA llama.cpp sidecar contract

Status: contract only; no binary or model is vendored by this commit.

## Upstream selection

- Project: ggml-org/llama.cpp
- Stable release selected for evaluation: v0.4.1
- Release commit: b29c606
- Rationale: stable tagged release; upstream release notes explicitly include JSON-schema handling improvements and publish CPU assets for Windows x64 and Ubuntu x64.
- Nightly builds are not the default production pin.

## Required capabilities

The eventual materialized sidecar MUST:
1. be a pinned upstream build whose archive bytes are verified against an immutable SHA-256 recorded in this directory before execution;
2. run locally as a child process through pulqva-intent-process, never through a shell;
3. receive the PULQVA intent JSON schema explicitly;
4. emit at most one bounded JSON document to stdout;
5. operate with the cleared environment provided by pulqva-intent-process;
6. have no authority to choose URL, proxy, Tor route, filesystem destination, downloader, command, or provider;
7. remain untrusted: output MUST still pass pulqva-intent-json and pulqva-core validation;
8. fail closed on timeout, crash, malformed/oversized/non-UTF8 output;
9. not require a user API key or cloud account for the production path.

## Schema

The normative generation shape is `intent.schema.json`.

Grammar-constrained generation is a reliability layer, not a security boundary. The Rust decoder/validator remains authoritative.

## Acceptance before a real model is admitted

- immutable upstream archive URL/tag/commit recorded;
- archive SHA-256 recorded and independently checked;
- expected executable identity recorded per platform;
- explicit argv reviewed;
- Windows x64 CPU and Linux x64 CPU smoke tests;
- process failure matrix remains green;
- model file has its own immutable digest and license/provenance record;
- multilingual hidden corpus evaluation is recorded separately from product code.

No inference-quality claim is made by this manifest.
