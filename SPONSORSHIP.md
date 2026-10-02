# Support PULQVA

Help turn local intent parsing and Tor-routed media retrieval into a tool people
can use without assembling a collection of command-line programs.

**Pre-alpha. This is a development proposal, not a released privacy product.**

[Support the maintainer on GitHub Sponsors](https://github.com/sponsors/yasha1971-coder)

This is the maintainer's existing, shared sponsorship profile. It also covers
ACEAPEX and GLYPH; a donation is not automatically earmarked for PULQVA.
Supporting development does not buy a released application. This page creates no
new payment account, sponsorship tier, delivery contract or anonymous payment claim.

## Three useful ways to help

- **Review a boundary:** inspect the [threat model](THREAT_MODEL.md), identify a
  concrete failure mode, and help define a reproducible test.
- **Reproduce the work:** start with the [recorded backend evidence](README.md#verified-result),
  not an assumption that the complete AI-driven app already works.
- **Fund a scoped milestone:** agree on deliverables, costs and acceptance evidence
  before treating a contribution as project-specific funding.

## Proposed first milestone: a reviewable request-to-file pilot

**Goal:** a bounded path from a plain-language media request to real choices,
explicit selection, and a locally saved file with a verification receipt.

Start with one declared Linux environment. Native Windows parity and clean-machine
packages are follow-on milestones, not hidden requirements inside the first budget.
Windows and Linux remain the product's release targets.

| Deliverable | Acceptance evidence required |
| --- | --- |
| Local intent-to-file pilot | Pinned source, runtime and model; documented invocation; public test inputs; real candidates; selected file; receipt with size and SHA-256 |
| Failure-path tests | Documented results for unavailable Tor, mid-flight Tor loss, rejected model output, cancellation and incomplete files; no false success receipt or intentional direct-network fallback |
| Reproduction by another developer | A replay on a separately identified environment with the source, procedure, outcome and limitations recorded; not just another badge from the author |
| Reusable test material | Published fixtures, failure scenarios and instructions explaining which parts another desktop application can reuse |

These are proposed deliverables. Their inclusion here does **not** mean they are
implemented, verified across the whole product, or accepted by an independent reviewer.
Reuse existing verified evidence where it applies; do not repeat completed measurements
just to create a new funding report. A hash identifies file bytes; it does not establish
that a downloaded file is safe.

## Follow-on work

| Priority | Evidence of completion |
| --- | --- |
| Native Windows parity | The same declared pilot and negative cases on native Windows, separately from WSL |
| Self-contained Windows/Linux packages | Clean-machine startup, supported OS versions, pinned component provenance and distribution-license checks |
| Independent privacy and supply-chain review | Agreed scope, reviewer engagement, public findings where safe, and tracked remediation |

A contributor's replay is not an independent security audit. Both are useful; they
must be reported separately.

## Before a funding agreement

A scoped proposal must name the responsible people, estimated work, rates or
supplier quotes, budget, payment milestones, acceptance tests and maintenance plan.
Budget and dates are not set on this page. No grant, sponsor or reviewer is confirmed
for this proposed milestone.
The use of AI-assisted development must be disclosed accurately where relevant.

Funding does not buy access to user queries, influence over search results, or
exceptions to the [privacy invariants](kernel/PRIVACY_INVARIANTS.md). Payment and
support remain separate from the intended no-account application workflow.

PULQVA is not affiliated with, endorsed by, or funded by Zcash, the Tor Project or
any other organization merely because it uses related software or seeks feedback.
A generic privacy tool is not automatically eligible for an ecosystem-specific grant.

## Discuss a contribution

Use [Issues](https://github.com/yasha1971-coder/PULQVA/issues) for a public,
non-sensitive pilot use case, reproduction result, or initial milestone discussion.
Do not post private queries, credentials, personal documents, donor payment details,
or exploitable security details in a public issue.

Engineering progress is recorded in commits, pull requests, CI evidence and the
latest checkpoint of the relevant branch. See [Contributing](CONTRIBUTING.md).
