# T069-G2A — finite evidence codec before another expensive generation

Date: 2026-10-03. Scope: existing CI evidence tools, not product architecture.
User outcome: avoid accepting or repeating inconclusive model/Tor tests while closing
the original local-intent -> choices -> selected verified file journey.

## Decision and current primary sources

Python time documentation describes monotonic clocks and integer nanosecond counters:
https://docs.python.org/3/library/time.html#time.monotonic_ns
Rust test documentation warns about shared process state and parallel interference:
https://doc.rust-lang.org/book/ch11-02-running-tests.html
GitHub documents immutable v4 artifacts and artifact SHA-256 validation:
https://docs.github.com/en/actions/tutorials/store-and-share-data

These are released platform facilities, not experimental product dependencies. Retain
pinned Rust/model/Tor components. Use Python standard library already present in CI,
no new test service, agent scheduler or downloader. Current Rust Instant/ErrorKind and
SOCKS5 references were also inspected for the queued readiness seam, but no runtime
change or transport diagnosis is claimed here.

## Implementation and acceptance

A pre-run manifest declares bounded IDs, hypotheses, expected outcomes, invariants,
order/dependencies, generation, source/checkout/run identity, component hashes and
isolation. The recorder copies it, accepts each result once, preserves all unstarted
probes as SKIP, stops after integrity failure and closes once. An independent verifier
requires the separately retained manifest and recomputes acceptance. Missing timing
cannot be PASS; unavailable component identity and unverified isolation prevent PASS.
No code execution/retry/repair API is added. Exclusive writes refuse replacement of
an existing generation. The verifier is not a signature, malicious-collector defense,
filesystem sandbox, or a check that referenced evidence files actually exist.

## Evidence generations and post-test review

Generation 1:16 local deterministic probes passed. Static review AFTER closure found
that the self-test report held unittest TestCase objects on failures, which JSON cannot
serialize. Candidate source is preserved at6cc878f69802520025c2b91b89980d86a551993c.
Generation 2:one reporter correction converts cases to string IDs; its explicit
synthetic failure/error serialization test was added. All17 probes passed. No repair
was performed inside either run; both manifests, receipts and raw results are retained
unchanged in the supplied local conversation archive (not mirrored in Git). The codec itself did not
change between these two generations. Tests are local-overlay Python evidence, with
actual evaluator/fixture/runtime digests, NOT new Rust or public-network acceptance.

Observed outcomes match the source requirements: monotonic timing, fresh per-test
fixtures, separate hypotheses/expectations, complete non-green early exits and immutable
generation output. Stdlib implementations were sufficient; no dependency upgrade was
justified. CI now invokes this matrix through the EXISTING continuity workflow and
retains its nested artifact. Remote CI remains pending until its exact checkout and
receipt are observed. No full G2 acceptance or progress-percentage increase.

## Limits and next

The failed readiness path needs its previously scoped passive diagnostic seam; this
codec is a prerequisite for collecting/validating that generation, not a fix for Tor.
Four known red jobs remain open. Runtime isolation, actual per-request collection,
legacy automatic live triggers and complete model/Tor/file integration remain open.
Only next action: observe T069-G2A-CI, then the prepared readiness-diagnostic phase.

The wider local `unittest discover` regression ran54 cases and reported1 import error
in the unchanged test_tagged_runner happy-case assertion. All17 new codec tests passed.
This existing-path error is not attributed to the codec or environment without evidence,
not repaired in this generation, and not hidden as a successful full-suite run.
`legacy-regression.log` is retained in that local evidence ZIP. Parent next_subtask remains
T069-G2, with next_phase=G2A-CI; existing recovery fixture acceptance remains unchanged.

Milestone loop remains: user-path goal -> current primary-source review -> decision
-> one implementation -> tests -> external post-test comparison -> one next action.
The next runtime must reuse the codec through its existing harness, not grow a second
framework. Legacy live CI triggers were not changed; this is not repository-wide
prevention of repeated expensive tests yet.
