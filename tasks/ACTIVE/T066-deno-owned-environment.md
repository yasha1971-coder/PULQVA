# T066 — Owned Deno environment and cache boundary

Status: ACTIVE. PR70 merged; main 10ece61f6b42919cea8fc4c8d07e6216007ef4f5 passed all 12 workflows.

T066-A: verified at 488d996a5c4affe62422ffffb2157f87c0cc2588, all 11 workflows green.
Desktop run 36241456647: Linux 108402634096 and Windows 108402634265 both
passed the two environment policy tests.
Inspected ADR-0006, T063 clean_environment and desktop launch boundary.
Clear inherited and explicit overrides, redirect HOME/temp/cache, suppress updates,
retain only explicitly supplied Windows system directory. No caller enables Deno.
Path syntax is NOT ownership or trusted native directory discovery.
T066-B1: verified at 8bc24d62b2d0a5c2dfcf9552b18a7465c8b71766, all 11 workflows green.
Desktop run 36242334885: Windows 108405062986 / Linux 108405063013 both
emitted PULQVA_DENO_CHILD_ENVIRONMENT_OK and passed native child test.
A poisoned intermediate
process verifies poison exists, applies the policy, then launches a clean child
that checks its actual environment and working directory. No global env mutation,
network access or Deno execution. Each subprocess wait has a 20-second bound.

T066-B: owned directory lifecycle, native system-directory discovery, actual child
environment probe, ownership-loss and success/failure cleanup tests remain.
Unavoidable analysis cache must stay in owned DENO_DIR until child exit.

T066-B2: verified at 626225107073ecdf5ac0b7112166350f26cd1fdd, all 11 workflows green.
Desktop run 36243575970: Windows 108408506722 passed two cleanup tests;
Linux 108408506759 passed those plus replacement-preservation test.
Private workspace home/tmp/cache with identity-checked apply/cleanup,
bounded postorder cleanup (4096 entries, depth 32), no symlink/special-file
traversal. Three tests added (replacement test Unix only); CI pending.
Caller must wait for all children before cleanup/Drop; no process guard yet.
Limits: same-user races are not atomically prevented; Windows ACL isolation,
Windows replacement coverage, production system-directory discovery and actual
Deno cache execution remain unverified. Oversized/special trees remain on disk
with an explicit cleanup error (Drop is best effort), not a zero-retention claim.

T066-B3: native system-directory lookup implemented, CI pending. Windows uses
GetSystemWindowsDirectoryW with bounded UTF-16 buffer and fail-closed validation;
Linux supplies no Windows variables. No inherited environment fallback or public
arbitrary system-root constructor. Workspace apply uses this boundary. Child
fixture now poisons WINDIR and checks the native result after process creation.
Source: https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getsystemwindowsdirectoryw
Reviewed 2026-09-26; shared OS directory avoids per-user Terminal Services path.

T066-B3 native execution passed on both platforms; full green anchor711b294
after media diagnostic recovery (historical network cause still unresolved).
T066-B4: enable original workspace replacement test on Windows as well as Linux;
add real filesystem substitutions of each tracked child and parent directory.
Assert apply fails without changing Command, explicit cleanup fails, Drop leaves
foreign and displaced original data untouched. No synthetic handle substitution
or skip-on-rename-error. Native CI pending. This is sequential replacement
coverage, not hostile concurrent race prevention or Windows ACL isolation.

B4 recovery: head0d0c489 desktop36248605915 Windows108422309634 failed
at root/parent rename with OS code5; child cache/home/tmp replacement passed.
Linux108422309479 passed. Root/parent tests now distinguish observed native
rename denial (identity intact, apply and cleanup still work) from completed
replacement (refuse apply/cleanup and preserve foreign files). Only Windows
code5 enters prevention assertions; all other rename errors fail. No production
handles weakened or test-only identity substitution. New CI pending. Prevention
evidence must not be described as successful root/parent replacement detection.

B4 recovery verified1379f1167990126c62c0d75b41e15d18883348ec all11 green.
Desktop36249829776: Windows108425656837 / Linux108425656974 five cache tests passed.

B5: TEST-ONLY single-child trial runner owns workspace through observed exit.
Spawn error drops unused workspace; normal/nonzero exits explicitly clean;
timeout kills and polls termination for at most2s before cleanup. Unknown
termination intentionally retains workspace/handles instead of deleting live
process data. Drop handles panic/error paths. Two native tests and three invoked
ignored helpers cover success, nonzero exit, timeout and missing executable.
CI pending. No production call path, Deno execution or descendant guarantee.
This supports the next pinned Deno cache trial; production process-tree lifetime
and egress confinement remain separate mandatory activation gates.

B5 diagnostic recovery: head7d02ee6 desktop36252748613 Windows108433721400
failed expected-success fixture (nonzero exit); Linux108433721545 passed.
Spawn-failure cleanup passed Windows. Original child output was discarded and
status reduced to boolean, so root cause unknown. Add fixture-only inherited
stdout/stderr and --nocapture plus raw ExitStatus in parent assertion. Default
trial runner output stays null; no environment dump or production change.
Diagnostic CI pending; no timeout, path assertion or ownership weakening.

B5 cause confirmed at55b01ae: Windows108439831192 (desktop36254943323)
success_fixture panics comparing extended-length HOME path with ordinary cwd
spelling of the same path; child exits101. Linux108439831295 passes.
Replace lexical assertion with same_file filesystem identity, retaining directory
existence and actual writes. No production prefix stripping or environment change.
Native verification pending; last fully verified anchor remains1379f11.
Outcome: explicit backend environment policy and owned cache lifecycle for later
bundled-runtime integration, following ADR-0006. No default Deno activation.

Acceptance:
- Inspect existing process launch and T063 restricted fixture evidence first.
- Typed allowlist clearing inherited environment; only documented native launch
  requirements retained. Reject host runtime/plugin/config/proxy injection.
- Owned DENO_DIR and required temporary/home locations, update checks suppressed,
  cleanup restricted to owned contents. Document unavoidable analysis cache.
- Native Windows/Linux tests for poisoned environment, ownership loss, cache
  lifecycle and success/failure cleanup. Distinguish fixtures from live EJS.
- Preserve Tor readiness and remote DNS. Environment policy is not descendant
  network confinement, file-race protection, or a release-ready sandbox.

Split implementation/native verification if needed. Prelaunch identity revalidation
and descendant egress enforcement remain separate mandatory gates before activation.
