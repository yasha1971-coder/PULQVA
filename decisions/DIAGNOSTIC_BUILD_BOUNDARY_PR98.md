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
