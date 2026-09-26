# NEXT

## UPDATE: T064-B integrated, exact-head CI pending

A verified at 5e67201226056d5a3a6164c33d508a7a30e86563 (11 checks green).
B consumes runtime policy in YtDlpLaunchPlan: disabled by default, explicit
bundled selection via typed builder, plugin discovery suppressed for every plan.
Full media/metadata vectors and process fixtures updated. Local Cargo unavailable;
continuity/diff checks only. No live bundled execution or file/hash gates claimed.
ONE next action: inspect PR #69 exact-head CI, diagnose red before closeout.
Do not merge until parent acceptance reviewed. Older A instructions below superseded.

## CURRENT: T064-A typed runtime representation — exact-head CI pending

Branch: feat/T064A-typed-runtime. Base: main@2b33eccc0b64b71bc133033e2b21f5562125683c.
PR #68 was already merged; all 18 post-merge checks passed as recorded in its
recovery checkpoint. Do not repeat that merge or its documentation closeout.
The new commit SHA, PR and CI run IDs are recorded in the T064-A PR checkpoint.

Implemented: additive BundledDenoPath and YtDlpJsRuntime types, deterministic
runtime-only argv, platform path rejection tests, explicit no-verification docs.
No production launch arguments changed; the new type is NOT yet consumed by
YtDlpLaunchPlan. Do not claim host discovery is disabled in the current launcher.
No kernel, dependency pin, privacy route or automatic runtime enablement change.

Local git clone failed (container DNS); cargo unavailable. Rust compilation and
unit tests NOT RUN locally. Existing CI must verify the exact new commit.
Historical Linux failures and missing positive Windows retrieval remain open.

ONE next action: observe exact-head CI of the T064-A draft PR. Diagnose red;
if all relevant checks are green, close the A phase with a PR comment. Then a
later bounded phase may implement T064-B in the same feature branch: integrate
the typed runtime into base/metadata/media plans, clear host discovery even for
the disabled path, suppress plugins per ADR-0006, and test the full argument
vectors while retaining ignore-config, readiness and socks5h. Parent T064 is
not DONE and this partial implementation must not be merged as completed T064.

Filesystem existence/regular-file checks, materialization/hash/ownership and
process environment are separate mandatory prelaunch gates, not path-type claims.
