# ADR-0003 — Atomic work and green checkpoints
Status: Accepted  
Date: 2026-09-20

## Decision
All implementation work is decomposed into atomic, independently verifiable tasks.

The project advances only by green checkpoints:
READ -> CHANGE -> VERIFY -> GREEN -> COMMIT -> STATE UPDATE -> NEXT

Large tasks are split before implementation if they contain multiple independent outcomes.

## Reason
This makes interruption cheap and prevents a long AI session from becoming a hidden dependency.
