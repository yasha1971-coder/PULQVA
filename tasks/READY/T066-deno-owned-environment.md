# T066 — Owned Deno environment and cache boundary

Status: READY after PR70 merges.
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
