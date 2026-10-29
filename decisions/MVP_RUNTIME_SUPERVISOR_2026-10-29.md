# MVP runtime admission and supervisor — 2026-10-29

Status: deterministic policy/controller implemented; native backend and live acceptance PENDING.

The prior Windows test spent its launch-relative deadline before a complete acknowledged-query readiness interval was demonstrated. This correction extends the existing matrix codec, rather than changing Tor or its pins.

AdmissionWindow checks a monotonic absolute deadline and reserves 90s readiness,10s capture,20s cleanup. PersistedMatrix writes an exclusive plan before work and closes a complete existing-format receipt on early exit/exception. RuntimeSupervisor wires these into one submitted query and always requests owned-job cleanup in finally. Missing readiness suppresses dependent results; missing/failed cleanup and remaining owned ports prevent acceptance.

Backend contract: prepare does not launch; launch_owned creates an isolated owned job/process group; members queries CURRENT ownership, not a static PID list; terminate_owned_job never targets unrelated PIDs. A restarted child must remain contained. Windows Job Object kill-on-close/assignment and Linux process-group behavior are NOT implemented or natively verified by this controller. Synthetic backend tests do not prove PID reuse safety or OS containment. Before live admission, implement/review native adapter with race-free ownership, bounded output, graceful-close timeout, independent socket verification and emergency cleanup. No prototype launched in this change.

Local Windows Python3.11:66 tests PASS (39 existing,17 admission/persistence,10 controller). Two-OS deterministic workflow added; hosted result pending exact new SHA. No product source, kernel, protected State/NEXT, dependency lock, sidecar pin or USB package changes. Existing PR99 Windows Arti rebuild failure remains independent and unresolved.

Next: inspect exact new SHA CI; add native backend and real harmless process-fixture ownership/CLEANUP tests before another live Windows generation. Linux root-pin/binary provenance remains an independent blocker. The mission is PENDING.

Primary OS reference: https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects
