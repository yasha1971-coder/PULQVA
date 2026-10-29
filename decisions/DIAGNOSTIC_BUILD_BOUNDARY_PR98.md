# Diagnostic build evidence boundary — PR98

Frozen application source: b217e0816f217ca1e65b63283dcb4cd0ab0ac79a.
This workflow-only branch is the build driver; its SHA is separate from source SHA.
The new job checks out exact source, rejects source edits, builds only the app
with Rust1.91.0 and --locked, and records binary and input-manifest hashes.
It never builds Arti, runs PULQVA/Tor, changes pins or claims live acceptance.
Existing checks remain enabled. Artifacts are app-only, not complete packages.

## Evidence-derived lessons

1. A commit SHA is insufficient build provenance when a packaging workflow
   mutates manifests before compiling. Record the effective manifest hashes and
   reject or explicitly bind an overlay. Historical Linux package at0262e9a
   rewrote root Arti SHA256SUMS. Its Arti2.7.0 binary5f579c25... therefore must not
   be silently paired with a clean PR98 app embedding root digestd6bef8db....
2. Local Application Control blocked an ICU build-script with OS4551; use trusted
   remote compilation rather than disabling protection. This does not establish
   the cause of runtime Tor readiness failure.
3. Cleanup ownership must span the dynamic process tree/session: Arti is restarted
   during bootstrap activation. Capturing one initial child PID is insufficient.

These are observations of this project, not a claim of worldwide novelty.
An experience report can cite raw logs, exact inputs, counterexamples and limits;
it must not present an untested hypothesis or an unfinished experiment as a result.

## Next bounded phases

Verify CI run and artifact-v4 provenance, receipt, source/workflow SHA, binary hash
and manifests. Stage a new Windows package with previously verified sidecars.
Before Linux runtime resolve the committed/effective Arti digest/version split
through a separate evidence-supported correction, without editing existing USB.
Then run the predeclared GUI/readiness/countdown matrix sequentially per OS using
owner's existing bounded-launch authorization. Never click Download. Reserve time
for graceful close, child reaping, PID/session and TCP/UDP checks and checkpoint.

## Artifact-export correction

Run40878061471 successfully built both app binaries, but blanket post-build
status cleanliness rejected untracked outputs before upload. Pinned tauri-build
2.6.3 and tauri-utils2.9.3 explicitly write gen/schemas JSON in the source tree.
This is generated output, not a license to permit changed source inputs.

The build driver and immutable source now have separate checkout roots. Before
build, all tracked source bytes are snapshotted; after build every tracked path
and raw SHA256 must still match. Only four exact platform schema JSON paths are
accepted as untracked outputs, with symlinks, invalid JSON, oversized files and
unknown names rejected. Generated hashes and guard/snapshot identity enter the
receipt. Six local probes passed; the symlink probe was skipped by Windows
privilege constraints and remains required on Linux CI. No app-source, pin or
lock change is made. Lost runner-local binaries require a new build wave; the
old successful compile is preserved as evidence, not relabelled as an artifact.
