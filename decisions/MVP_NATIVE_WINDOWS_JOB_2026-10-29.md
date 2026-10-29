# Windows native owned-job proof — 2026-10-29
Status: native primitive tested; full GUI/Tor backend integration still PENDING.

Adds runtime_windows_job.py to the existing runtime harness. CreateProcessW uses STARTUPINFOEX/PROC_THREAD_ATTRIBUTE_JOB_LIST, so job assignment occurs at creation instead of an after-launch PID race. The unnamed job handle is not inherited; kill-on-close is enabled and breakaway is not. Cleanup terminates only this job and polls CURRENT membership; it never kills arbitrary cached PIDs. A monotonic timer provides an owned-job wall deadline independently of the orchestrator.

Six actual Windows/Python3.11 fixture tests PASS: root cleanup; child inheritance and loopback listener release; watchdog; second unrelated owned test job remains alive; invalid executable creates no member; closing the last job handle terminates the root, checked through a retained synchronization handle. Corrected test output:6 tests/0.376s/PASS.
First run retained locally:5 pass,1 error; job membership became empty before an immediate exclusive loopback rebind succeeded. The correction waits at most4s for actual rebind and checks empty membership again. This is a measurement correction, not relaxed cleanup or proof of a general socket root cause. No external network action, PULQVA or Tor launch.

Limits: this is not a complete desktop backend, not Linux process containment, not general OS security isolation, and not proof against every process-creation interruption. No graceful GUI-close callback, bounded stderr collector or UI-to-receipt bridge is supplied by this primitive. The existing controller is not yet admitted for live Tor. Fixture tests are ordinary native unit tests; no retroactive complete MVP matrix receipt is claimed. Current script runs the inherited fixture environment; live environment admission remains separate.

Next: bind this minimal primitive to candidate execution with sanitized opt-in stderr capture and GUI evidence, graceful-close request, independent socket verification and predeclared absolute deadline/receipt. Use the already verified b217 Windows app unchanged. No further generic harness expansion. Record the first missing real Tor transition, then correct that cause in a separate branch.

Kernel/State/NEXT/pins/locks/product code/USB packages unchanged; no merge. Windows Arti rebuild identity mismatch and Linux pin split remain independent.

References:
https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects
https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags
https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute
