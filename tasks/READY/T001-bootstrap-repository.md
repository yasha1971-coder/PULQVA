# T001 — Establish repository and green bootstrap checkpoint
Parent: BOOTSTRAP  
Status: READY

## Goal
Make the continuity/kernel scaffold the first verified source of truth in `yasha1971-coder/PULQVA`.

## Acceptance criteria
- repository exists;
- scaffold is committed;
- `python scripts/continuity_guard.py` passes;
- GitHub Actions continuity guard is green;
- `PROJECT_STATE.json` contains the verified commit;
- repository-missing blocker is removed;
- `kernel-v1.0.0` tag marks the verified kernel checkpoint;
- next atomic implementation task is written before new product code starts.

## Out of scope
- Tauri application;
- UI;
- Tor/Arti;
- AI integration;
- yt-dlp;
- release packaging.
