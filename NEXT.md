# NEXT

## Current verified state
The PULQVA continuity/kernel bootstrap scaffold has been generated.

The product kernel is defined in `kernel/` and is intentionally independent of specific UI, bundler, AI provider, or retrieval backend choices.

## Next atomic task
**T001 — Establish the repository and green bootstrap checkpoint**

After `yasha1971-coder/PULQVA` exists:
1. Publish this scaffold to the default branch.
2. Run the continuity guard.
3. Fix bootstrap-only issues if any.
4. Set `last_verified_commit` in `PROJECT_STATE.json`.
5. Remove the repository-missing blocker.
6. Tag the verified kernel checkpoint `kernel-v1.0.0`.
7. Move `T001` to DONE.
8. Create the next small implementation task; do not jump directly to Tor + UI + AI together.

## Do not do yet
- Do not add product code before the bootstrap checkpoint is green.
- Do not select a remote AI provider as an irreversible dependency.
- Do not implement direct-network fallback.
- Do not redesign the kernel while implementing tooling.

## Success
A fresh agent can open the repository, read the required files, run the guard, and identify the next task without reading any prior chat.
