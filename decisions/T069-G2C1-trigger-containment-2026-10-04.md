# T069-G2C1 — separate automatic contracts from unadmitted live jobs

Date: 2026-10-04. Source base: 54136c31554af0192b1c10a78045009d610b70d0.
Owner goal: reach a verified intent-to-file product without repeated blind live runs.
Prior handoff: PR80/5980560445. Predeclared phase: PR80/5980678069.

## Current primary sources and decision

Reviewed released GitHub mechanisms, not experimental third-party tooling:
- https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow
- https://docs.github.com/en/actions/concepts/workflows-and-actions/concurrency
- https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency

A workflow trigger is not evidence of runtime/capture/isolation readiness. Concurrency
only schedules jobs and cannot authenticate an evidence generation. Path filtering
alone does not implement this contract; skipped jobs may appear successful in GitHub.
Decision: keep automatic deterministic contracts and explicitly distinguish their
success from NOT_TESTED live behavior. No new dependency, launcher or transport policy.

## One correction, deliberately staged

In the four existing readiness/media/metadata/Commons workflows, the original live
job now depends on contracts plus explicit admission. Ordinary PR/push events run
native contracts on both OSes, not Arti preparation or a public-network test. All
old live steps and success predicates are retained byte-for-byte, checked against
pre-change step SHA256 values. The original Commons full evidence/unit preflight stays
available automatically; its per-live-job structural tests still verify the original
live block. No continue-on-error, pin relaxation or false live PASS is added.

The supported legacy_live_guard produces NOT_TESTED with zero live-launch budget.
A dispatch alone returns BLOCKED/nonzero: a compliant live collector is absent in these
old paths. This is temporary fail-closed containment, NOT completed positive admission.
There is deliberately no magic enable flag or fabricated collector registration.
This is not blanket disabling of deterministic tests or closure of old live failures.
A positive live path must be added next in one existing workflow with actual capture,
exact identities/prerequisites and bounded launch; do not leave this as the final design.
Other independent expensive build/materialization jobs were not changed. Main untouched.

## Verification and evidence generations

First local preparation failed manifest validation BEFORE any probe: flags identity
was missing. Preserve its invalid manifest, raw preflight-error record and prepared
source patch. No passing matrix is invented for it. A separate correction adds only
the flags identity to the test collector. The new frozen generation ran 11/11 synthetic
contracts PASS. Three related Commons workflow structural tests also passed. YAML was
parsed locally as an additional syntax check, not proof of GitHub scheduler behavior.
No local rustc/cargo was available; no native pass or live transport execution claimed.

Each probe has a predeclared ID/hypothesis/invariants/expected result, actual outcome
and monotonic duration. The existing G2A codec records the matrix. Component hashes
are checked around the matrix; fresh fixtures own temporary output; no hot fixes.
G2B1's accepted native archive is retained byte-for-byte without a new native rerun.

## Post-test comparison and pending hosted acceptance

Local contracts confirm source gating, exact original live-step identity and actual
CLI NOT_TESTED/BLOCKED behavior. This matches the selected trigger/needs/outputs model.
GitHub-hosted execution of this change remains pending: inspect child jobs and uploaded
status artifacts, not workflow badge alone. No full industrial-pipeline claim.
Before enabling ONE live generation, implement/verify the complete positive-admission
and collector path. Full request->choice->verified-file, UI/packages and clean-machine
acceptance remain open. Global accepted anchor is unchanged.
