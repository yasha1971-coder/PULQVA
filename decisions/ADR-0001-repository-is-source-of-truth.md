# ADR-0001 — Repository is the source of truth
Status: Accepted  
Date: 2026-09-20

## Context
Long-running AI-assisted projects fail when critical state exists only in chat history. Dialogues can be truncated, interrupted, or resumed by a different agent.

## Decision
PULQVA stores durable project truth in the repository:
- kernel = enduring product identity and invariants;
- `PROJECT_STATE.json` = current machine-readable state;
- `NEXT.md` = immediate human-readable continuation point;
- task files = atomic units of work;
- ADRs = durable reasons for architectural choices;
- Git history = execution history.

Chat history is context, never authoritative state.
