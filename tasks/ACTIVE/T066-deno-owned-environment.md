# T066 — Owned Deno environment and cache boundary

Status: ACTIVE. PR70 merged; main 10ece61f6b42919cea8fc4c8d07e6216007ef4f5 passed all 12 workflows.

T066-A: explicit command environment policy implemented; CI pending.
Inspected ADR-0006, T063 clean_environment and desktop launch boundary.
Clear inherited and explicit overrides, redirect HOME/temp/cache, suppress updates,
retain only explicitly supplied Windows system directory. No caller enables Deno.
Path syntax is NOT ownership or trusted native directory discovery.
Tests inspect Command state; actual child inheritance remains for T066-B.

T066-B: owned directory lifecycle, native system-directory discovery, actual child
environment probe, ownership-loss and success/failure cleanup tests remain.
Unavoidable analysis cache must stay in owned DENO_DIR until child exit.
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
