# T069-E intent model evaluation contract

The seed file is public and is NOT the final hidden evaluation set. It exists to make semantics reviewable before model selection.

## Hard gates

A candidate is ineligible if any evaluated output:
- cannot be decoded by pulqva-intent-json;
- violates pulqva-core InterpretedIntent validation;
- introduces authority beyond query + Ask/Autopilot;
- requires a user API key or remote inference;
- cannot run through the supervised T069-C process boundary.

## Primary measurements

1. strict JSON pass rate;
2. semantic query preservation;
3. Ask/Autopilot classification accuracy;
4. authority/injection rejection;
5. multilingual consistency;
6. p50/p95 cold and warm latency on CPU baseline;
7. peak RSS;
8. GGUF bytes on disk.

No single aggregate score selects a winner. Report the Pareto set and all raw failures.

## Languages

Minimum evaluation strata: English, Russian, Ukrainian, German, mixed-language. The hidden corpus must contain paraphrases not present in prompts/examples.

## Model ladder

Start with the smallest permissively licensed candidate. A larger model is admitted only if the smaller candidate fails a hard gate or the quality delta is materially useful for PULQVA's narrow intent task.

## Current first candidate

Qwen3.5-0.8B instruction model, official ggml-org GGUF conversion, Q4_0:
- repository: ggml-org/Qwen3.5-0.8B-GGUF
- file: Qwen3.5-0.8B-Q4_0.gguf
- size: 563 MB
- SHA-256: 57d1997790d1744fba5b40a7317df71ea5e2acee28c47e78f0cce39c0703f8cf
- upstream model license: Apache-2.0

This is a candidate, not a selected production model.
