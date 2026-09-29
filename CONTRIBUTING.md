# Help build PULQVA

**You do not need permission to challenge PULQVA.**

Find a hole. Reproduce a failure. Propose a smaller design. Bring evidence.

PULQVA is pre-alpha. The product goal is **Tell AI what you want. Get the file.** The engineering rule is equally important: **privacy by design, not by promise.** Contributions should make those statements more testable, not merely more impressive.

## Three useful ways in

### 1. Help solve native Windows + Tor

This is the clearest open engineering gate today.

The fixed Linux backend path has reproduced request → real choices → selected file on fresh GitHub runners and on the owner's WSL2 laptop. Native Windows is not established. The current hosted-Windows live failure reaches the Arti SOCKS endpoint and SOCKS5 negotiation, then times out at destination readiness before Commons HTTPS or file retrieval.

Useful contributions include:
- reproduce the exact failure on native Windows;
- isolate Arti/bootstrap, destination-connect, firewall, runtime or platform differences;
- improve diagnostics without weakening fail-closed behavior;
- produce a minimal reproduction or evidence that disproves the current hypothesis.

**Do not “fix” this by silently allowing direct networking or by converting a timeout into success.**

Start with the active engineering record: [PR #73](https://github.com/yasha1971-coder/PULQVA/pull/73).

### 2. Attack the privacy assumptions

Treat privacy claims as things to falsify.

Useful questions:
- Can any required network path escape the Tor boundary?
- Can a child process inherit a direct-network route?
- What happens if Tor disappears during search or download?
- Are DNS, redirects, subprocesses and retries constrained as intended?
- Can stale state, races or cancellation produce a different route?
- Does a proposed AI provider learn more than the UI tells the user?
- Is a “verified” statement actually supported by evidence?

A finding is valuable even if it makes PULQVA look worse today.

When reporting a privacy/security problem, avoid posting secrets or unnecessary exploit detail in public. If disclosure would create real user risk, contact the repository owner privately first.

### 3. Build the Intent → Object future

The destination is a consumer journey:

```text
human request
    → intent interpretation
    → private search
    → real choices
    → explicit choice or Autopilot
    → retrieved file
    → visible privacy/evidence state
```

High-value work includes:
- local or no-user-key intent models;
- typed intent schemas and adversarial validation;
- result ranking without hidden tracking;
- desktop UX and accessibility;
- self-contained Windows/Linux packaging;
- observable privacy state and failure UX;
- reproducible release/evidence tooling.

Do not present Product Vision UI or future AI behavior as already shipped.

## Evidence standard

A useful contribution states:

1. **Exact source** — commit/ref and platform.
2. **Exact claim** — what you expected.
3. **Observation** — what actually happened.
4. **Reproduction** — minimal steps or automated test.
5. **Boundary** — what the result does *not* establish.

Prefer a failing test, receipt, trace or minimal reproduction over a confident paragraph.

## Before changing code

- Keep changes small enough to explain.
- Preserve fail-closed behavior.
- Do not weaken existing tests just to make CI green.
- Do not introduce a second hidden downloader/network path.
- Do not add credentials, personal data, generated binaries or unrelated artifacts.
- For dependency changes, explain why the new dependency is necessary.
- For UI/brand work, distinguish **PRODUCT VISION / PRE-ALPHA** from **VERIFIED BACKEND** evidence.

## Pull requests

A good PR title says what changed, not how exciting it is.

In the description include:
- problem;
- approach;
- tests/evidence;
- privacy/security impact;
- platform impact;
- remaining limitations.

If you are unsure whether an idea fits, a small issue or draft PR with evidence is better than a large speculative rewrite.

## Current truth

PULQVA is **not** a finished anonymous downloader and does not promise absolute anonymity. The complete AI-powered one-click Windows/Linux product is not shipped. The strongest current evidence is a fixed Linux backend request → choices → file fixture; native Windows, finished AI/UI, packaging and broader failure-path confinement remain open.

That incompleteness is not something to hide. It is the work.

---

**Intent → Object. Privacy by design. Not by promise.**
