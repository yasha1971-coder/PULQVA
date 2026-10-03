# NEXT

## T069 — local intent -> Tor choices -> explicit selection -> verified file

Read live PR80 / feat/T069-intent-contract HEAD, State and recovery/INDEX together.
No autonomous mode, merge or release. Frozen kernel and global accepted anchor remain.
G1 acceptance is retained in State; this change does not advance it.

## ONE NEXT ACTION: T069-G2 / phase G2A-CI

Observe the new current-head continuity job and its nested g2a/matrix_receipt.json,
manifest.json and test-results.json. Check actual checkout, component identities and
all planned probes before accepting the codec substep. Do not repeat model attempt9.

Implemented in tools/ci/matrix_receipt.py: finite recorder + independent-manifest
verifier. Local generation1 passed16 probes. Static review then found the self-test
reporter could not serialize unittest failure objects. That candidate is preserved at
6cc878f69802520025c2b91b89980d86a551993c. One separate correction stringifies test IDs
and adds its negative regression; generation2 passed17 probes. The codec did not change.
New remote CI is PENDING; no runtime capture or Tor fix is implied.

The broader local Python regression ran54 cases and had1 import error in the unchanged
happy-case assertion of test_tagged_runner.py. All17 new codec cases passed. Cause not
established, no speculative correction or baseline rerun, no all-green claim.
Local observation summary: recovery/evidence/g2a-observation.json. Raw local evidence
was provided as a conversation archive, not mirrored in Git; preserve remote evidence
when closing this gate. No fabricated current-commit or whole-runtime identity.

Then resume the bounded passive readiness diagnostic from checkpoint5972671898,
reusing this codec before any expensive evidence generation. Preserve SOCKS action
order, destinations, retry/time budgets and Tor-only routing; test trace equivalence.
Do not treat retries on shared Tor state as independent probes. Arti Windows compiler
pin mismatch and Commons Linux ConnectOrTls are separate open failures. No blind reruns.

Full G2 remains OPEN: wire actual model/readiness capture and independently establish
context/cache/filesystem/transport isolation in the EXISTING harnesses. The codec does
not launch tests, fetch evidence references or sandbox runtimes. Its own PASS is not
permission to call old live jobs compliant matrices. Legacy automatic live triggers
remain unchanged and are not a source of new acceptance by themselves.

Decision/pre/post-test primary review: decisions/T069-G2A-evidence-boundary-2026-10-03.md.
Historical full G1 handoff:65f5f2d:NEXT.md and State verification_context. The original
user path, Windows/Linux UI/packages/clean machines and privacy blockers stay OPEN.
