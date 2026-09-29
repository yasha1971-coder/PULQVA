"""Conservative reducer for a collector-normalized, exact-source-SHA job snapshot.

This module does not fetch GitHub, configure required checks, or verify E2E files.
Call separately for code checks and live acceptance: neither substitutes for the
other. source_sha MUST come from a trusted collector associating the job with its
PR head; record the actual checkout/merge SHA separately for reproducibility.
The expected job set MUST be declared in advance, not inferred from returned jobs.
"""
from __future__ import annotations

import re
from collections.abc import Iterable, Mapping
from dataclasses import dataclass


@dataclass(frozen=True)
class Verdict:
    state: str
    failed: tuple[str, ...] = ()
    pending: tuple[str, ...] = ()
    missing: tuple[str, ...] = ()
    problems: tuple[str, ...] = ()


def evaluate(source_sha: str, expected_jobs: Iterable[str],
             jobs: Iterable[Mapping[str, object]]) -> Verdict:
    """Never report PASS with a missing, stale, skipped, or failed expected job."""
    if re.fullmatch(r"[0-9a-f]{40}", source_sha) is None:
        raise ValueError("expected a full lowercase source SHA")
    expected_list = list(expected_jobs)
    if (not expected_list or any(not isinstance(x, str) or not x for x in expected_list)
            or len(set(expected_list)) != len(expected_list)):
        raise ValueError("declare a nonempty, unique expected job set")
    expected = set(expected_list)
    seen: set[str] = set()
    failed: set[str] = set()
    pending: set[str] = set()
    problems: set[str] = set()
    for job in jobs:
        name = job.get("name")
        if not isinstance(name, str) or name not in expected:
            problems.add("unexpected_or_invalid_job")
            continue
        if job.get("source_sha") != source_sha:
            problems.add("stale_or_unattributed_job")
            continue
        if name in seen:
            problems.add("duplicate_job_requires_attempt_selection")
        seen.add(name)
        status, conclusion = job.get("status"), job.get("conclusion")
        # Inspect child jobs, never assume an in-progress parent has no failures.
        if status == "completed":
            if conclusion in ("failure", "timed_out", "action_required", "startup_failure"):
                failed.add(name)
            elif conclusion != "success":
                problems.add("required_job_not_successfully_executed")
        elif status in ("queued", "in_progress", "waiting", "requested", "pending"):
            pending.add(name)
            if conclusion is not None:
                problems.add("inconsistent_job_status")
        else:
            problems.add("unknown_job_status")
    missing = expected - seen
    state = ("FAIL" if failed else "INCOMPLETE" if problems or missing
             else "PENDING" if pending else "PASS")
    return Verdict(state, tuple(sorted(failed)), tuple(sorted(pending)),
                   tuple(sorted(missing)), tuple(sorted(problems)))
