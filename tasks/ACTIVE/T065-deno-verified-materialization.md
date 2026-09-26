# T065 — Verified Deno materialization boundary

Status: ACTIVE — T065-B implementation pending CI.
Outcome: owned, hash-verified Deno executable receipt usable by later launch
integration; syntax-only BundledDenoPath is not sufficient authorization.

Inspect existing Arti/yt-dlp/FFmpeg materialization patterns and pinned Deno
CANDIDATE.json / ARTIFACT_VERIFICATION.json before choosing the smallest adapter.
Split implementation/real-artifact validation into bounded phases if needed.

Acceptance:
- Exact platform/version/archive and extracted executable hashes/sizes from
  existing verified candidate; no new version or host runtime discovery.
- Reject missing, corrupt, nonregular and unsafe archive/path inputs; bounded
  extraction into owned staging, no overwrite of unrelated user files.
- Cleanup partial staging on error; typed receipt records actual verified file.
- Tests cover negative identity/path cases and exact successful receipt;
  verify real pinned artifacts on Windows/Linux before completion.
- State precisely what replacement-race/prelaunch revalidation remains required.

No default runtime activation, live YouTube, network fallback or user installation.
Environment allowlist/owned cache and process network confinement are separate
required integration gates; do not infer them from successful materialization.

## Bounded phases
A: authenticated immutable ZIP adapter and pinned identities; verified on Windows/Linux at a99eb1b023f04aa5fe011651b36f6866746ad526.
B: regular-file source snapshot and owned staging receipt; implemented, CI pending.
C: real pinned-artifact adapter/staging validation on Windows/Linux; not started.
Do not mark DONE or merge A as completed T065. Production launch stays disabled.
