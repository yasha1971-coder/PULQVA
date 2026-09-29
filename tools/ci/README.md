# Discovery preflight and stage evidence

The existing commons-tor-discovery-check runs these steps on each native OS:
Python evidence tests -> pinned Rust -> existing locked core/discovery contracts
-> build live harness -> pinned Arti -> existing real search -> positive receipt.
Normal GitHub step failure stops the later product stages. No continue-on-error,
retry, permission expansion, dependency graph change or success downgrade is used.

The finalizer runs after success/failure when checkout and Python setup succeeded.
It records only declared step outcomes, source/checkout SHA, run/attempt and OS.
It does not serialize arbitrary context outputs or collect raw process logs,
URLs, queries, titles, response bodies or provider identifiers. Precise existing
safe failure categories remain in the existing live harness log; this receipt
identifies the failing workflow step, not the Tor protocol root cause.

Code evidence and live evidence have separate verdicts. A live failure cannot
erase passed code checks, nor can code success close live acceptance. Missing,
skipped, cancelled, invalid or wrong-SHA evidence is not PASS. Evaluation uses
outcome rather than conclusion so an allowed failure would not look successful.
Expected steps are declared in discovery_receipt.py, not inferred from results.

Scope is this OS's job up to the finalizer. It does NOT aggregate the sibling OS
or other workflows, prove artifact-upload success, cryptographically attest the
run, or verify a selected file. To report whole-PR status, inspect actual child
jobs for the exact source SHA and select the current attempt. The reducer alone
is not an automatic GitHub collector or an enforced merge/release gate.
Checkout/Python/finalizer failure or runner loss may prevent the receipt; absence
must stay incomplete, never pass. The original failing step still fails the job.

Run locally: python -m unittest discover -s tools/ci -p "test_*.py" -v

Primary-source review, 2026-09-27:
- https://docs.github.com/en/actions/reference/workflows-and-actions/expressions
- https://docs.github.com/en/actions/reference/workflows-and-actions/contexts
- https://doc.rust-lang.org/cargo/commands/cargo-test.html
- https://docs.github.com/en/actions/tutorials/store-and-share-data

This implements the narrow preflight/evidence recovery from PR73 checkpoint
5856546895. It does NOT fix UrlQueryOrFragment or Windows Readiness(Timeout).
