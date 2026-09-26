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

## T065-C launch
B verified at f2a47abe0fa631e6a33c271ba641eb6f655a6df7, all 11 workflows green.
Desktop run 36224411245: Windows job 108355535489 (10 Deno tests),
Linux job 108355535542 (12 Deno tests), all passed.
C adds independent Python ZIP reference and explicit ignored Rust integration test
on both native runners: real pinned source, ZIP adapter, owned stage, disk hash,
receipt, Unix permissions, cleanup and unrelated-file preservation. Missing source,
directory source and wrong-platform archive must fail before output is created.
C is pending CI; no real-artifact success claimed until exact-head logs confirm it.
