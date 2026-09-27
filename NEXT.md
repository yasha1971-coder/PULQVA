# NEXT

## CURRENT: T068 coherent readiness migration recovery

PR73 / feat/T068-live-tor-e2e. Parent 04ac183089b7ba1e54877d66be869055bf1b933e.
Main unchanged. Global verified anchor remains fe1fb03dd88e9dea01a4d70dd69082cb1ba8b4c0.
Separate Linux C1 anchor: 8934c3e81c51ac6c24e3cbb1099d4d321629981c,
live run36327480415/job108642912589, ten real choices. Not file-download proof.

Parent run set completed: 10/13 workflow successes. rust-check36343498991
job108688018537 showed E0308: https/tests.rs still used the old unit Timeout as
a value. Live run36343498990 stopped at contracts on both OSes; Arti/live steps
were skipped, not Windows network evidence. Do not download these logs again.

ONE recovery outcome: finish the staged-timeout integration without changing the
pre-diagnostics address-attempt policy. Expand the existing HTTPS mapping test to
construct all three timeout stages. Restore IPv6 attempts after EVERY retryable
IPv4 failure, not only Listener. Preserve maximum observed stage across both
addresses. This repairs an introduced control-flow regression; it does not claim
to explain the original Windows timeout. The socket exchange, destination, TLS,
90-second caller budget, per-attempt values and outer retry loop are unchanged.

Four private Rust regression tests cover all 25 pairs of success/retry-stage/
protocol outcomes against the prior coarse policy, all nine retry-stage pairs,
IPv6 success after each IPv4 retry stage, and fixed timeout formatting. Tests use
a private injected probe for action traces; they are not public Tor or real-socket
stage-failure evidence. The prior successful Linux Commons normalization is untouched.

Existing native Commons workflow now prints the exact checkout's TorReadiness
source inventory and requires --workspace --all-targets check and no-run builds
with --keep-going, then privacy library tests and the existing core/discovery
contracts before Arti/live. Explicit Bash/set -euo pipefail prevents intermediate
command failures being masked on Windows. These are the four root workspace
members, not every separate workspace or every optional feature. Other workflows
remain independently triggered: no global DAG/merge-gate change is claimed.

Local verification: exact baseline Git hashes checked; only asserted edits;
25 Python tests passed, including the strengthened workflow contract. The new
contract rejects the old narrow workflow. YAML and Bash syntax checked; five
actual Bash executions with stub Cargo verify early-failure propagation. None of
that is Rust compilation. Local Rust is absent and public archive retrieval failed
DNS. Full local repository inventory was NOT completed. Native all-target checking
is PENDING and is the explicit completeness gate, not indexed GitHub code search.

ONE NEXT ACTION: inspect this recovery's exact-head native compilation, all four
new arti_readiness tests and all three HTTPS timeout mappings. If code passes,
inspect the same live run and stage reports; do not blindly rerun or interpret a
skipped live stage as Windows failure. Record the observed stage, not an invented
bootstrap cause. Do not start C2 in the same observation. Publication SHA/run IDs
are in the PR checkpoint. No verified-anchor advancement or main merge.

Windows C1, selected-file C2/real bytes/digest, actual in-flight Tor loss and
positive Windows/Linux full E2E remain open. Existing deadline overshoot, T066
cleanup/races, confinement, SDK-content, YouTube/AI/UI and packaging remain open.
Source review 2026-09-27: Cargo check/test docs (--all-targets, --keep-going,
--no-run) and GitHub workflow syntax (Bash early exits versus PowerShell status).
https://doc.rust-lang.org/cargo/commands/cargo-check.html
https://doc.rust-lang.org/cargo/commands/cargo-test.html
https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax
