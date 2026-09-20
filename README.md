# PULQVA

**Private intent-to-file. One package. Zero configuration. Tor by default.**

PULQVA is being built around a deliberately small, durable product kernel:

`natural-language intent -> private search -> real choices -> one click / Autopilot -> file`

The implementation stack is allowed to evolve. The product/privacy contract is not.

## Development continuity
This repository is designed for long-running AI-assisted development without depending on chat history.

Start here, in order:
1. `AGENTS.md`
2. `kernel/`
3. `PROJECT_STATE.json`
4. `NEXT.md`

Then run:
```bash
python3 scripts/continuity_guard.py
```

## Current phase
Bootstrap only. Product implementation begins after T001 creates the first verified green checkpoint and freezes tag `kernel-v1.0.0`.

See `NEXT.md`.
