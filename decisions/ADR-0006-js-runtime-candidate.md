# ADR-0006 — Deno candidate for bundled YouTube challenges

Status: Proposed; T061 assessment, runtime adoption deferred pending executable evidence
Date: 2026-09-25

## Decision and kernel purpose

Select Deno 2.9.7 for a bounded compatibility trial with existing yt-dlp 2026.08.19
and EJS 0.8.0. Do not change the shipping runtime in this task. This supports
ONE PACKAGE / ZERO CONFIG retrieval without asking users to install JavaScript.
It is not natural-language search, Autopilot, or proof of a complete demo.

Deno is recommended by upstream and supports restricted challenge execution.
The pinned yt-dlp source declares Deno >=2.6.6 and optional pin 2.9.5; 2.9.7
is a deliberate trial candidate because subsequent upstream fixes cover network
permission checks, cache and lockfile validation. These fixes do not establish
that 2.9.5 is exploitable in our path, nor prove 2.9.7 compatible.
QuickJS-NG is deferred: possible package-size benefit is unmeasured, and upstream
documents temporary script files and slow older versions. Node adds another
candidate without a demonstrated benefit. No runtime auto-upgrades.

## Source evidence

yt-dlp source: https://github.com/yt-dlp/yt-dlp/tree/3a08beaf031ab68f966401ead017ac81fe8486cf
At that commit:
- pyproject.toml pins yt-dlp-ejs==0.8.0.
- yt_dlp/__pyinstaller/hook-yt_dlp.py collects yt_dlp_ejs **/*.js.
- yt_dlp/extractor/youtube/jsc/_builtin/vendor/_info.py names version 0.8.0
  and expected hashes for core/lib minified and unminified scripts.
- ejs.py consumes package core()/lib() as minified scripts.
- deno.py launches runtime_info.path directly with run and stdin, --no-prompt,
  --no-remote, --no-config, --no-lock, --node-modules-dir=none, --no-code-cache;
  the non-NPM variant additionally uses --no-npm and --cached-only.
- options.py supports --no-js-runtimes and --no-remote-components.

This establishes the declared bundle contract, not inspection of both executable
payloads. The trial must verify packaged EJS version/scripts against this contract.

Deno source: https://github.com/denoland/deno/tree/0c071246a412575e07423263404a5d13e7ed6aa2
Release: https://github.com/denoland/deno/releases/tag/v2.9.7
EJS release: https://github.com/yt-dlp/ejs/releases/tag/0.8.0
Upstream guide: https://github.com/yt-dlp/yt-dlp/wiki/EJS
Platform guide: https://docs.deno.com/runtime/getting_started/installation/
Sources inspected 2026-09-25. Versioned source takes precedence over mutable wiki.

## Required launch boundary (not implemented here)

Typed argument vector, never shell. Supply --ignore-config, --no-js-runtimes,
then --js-runtimes deno:<verified absolute executable>, --no-remote-components.
Retain approved SOCKS remote-DNS route and readiness gate. Missing or mismatched
bundled EJS/runtime must fail; do not fetch npm/GitHub components or discover a
host-installed runtime. Disable plugins and user configuration in the trial.
Challenge JavaScript receives no permission grants for network/files/processes.

deno.py inherits environment and explicitly notes an unavoidable analysis cache.
Design an environment allowlist with owned DENO_DIR and cleanup, suppress update
checks, reject proxy/permission/config injection. Verify these controls at execution;
CLI permissions are not an OS sandbox or a guarantee of zero local writes.
Distribution must include third-party notices and required source offers as
applicable; MIT/Unlicense project labels do not cover every bundled dependency.

## Platform and size implications

Candidate targets are Windows x86_64 MSVC and Linux x86_64 GNU only.
Upstream installation docs state Windows 10/Server 2016 version 1709 minimum.
Our complete application's minimum Windows build is not established by that fact.
Linux GNU asset requires ABI verification on the eventual minimum supported distro;
Alpine/musl and ARM are not established by these candidates.
Archives add about 42.6 MB Windows / 41.6 MB Linux before extraction; installed
size, startup latency and memory remain unmeasured. No tiny-package claim.

## Bounded acceptance experiment (follow-up)

Use clean Windows/Linux CI environments and only verified bundled dependencies.
Separate artifact acquisition in CI setup from monitored application execution.
First inspect executable payloads, record EJS 0.8.0 identity, recompute archive
hashes, measure extracted bytes and run local permission-negative fixtures.
Include attempted network/DNS, file access and child process, environment/config
poisoning, missing runtime, corrupt EJS, and empty owned cache. Observe descendants.

Then use one fixed public short YouTube fixture approved for retrieval, no
accounts, cookies, API keys or host runtime. Record URL and expected identity in
the test before launch. Bound bootstrap to 180 s, retrieval to 120 s, output to
20 MiB via supervisor, whole job to 8 min, zero automatic retries.
Require evidence of actual Deno/EJS challenge execution and a nonempty validated
media file; metadata success or a URL that bypasses challenges is insufficient.
Observe child-process network/DNS so only Arti can reach external endpoints.
Kill the owned process tree and remove partial output on timeout/cancellation.

A blocked Tor exit, challenge denial or bootstrap timeout is availability failure,
not retrieval success. A negative test may separately prove fail-closed behavior.
Both OSes need positive retrieval plus negative privacy checks before adoption.
No direct fallback, cookies or remote component enablement to make tests green.

## Readiness evidence

| Requirement | Current evidence | Missing gate |
| --- | --- | --- |
| Source identity | Exact sources and upstream asset hashes in candidate manifest | Locally recomputed downloads and extracted executable receipts |
| Bundled EJS | Pinned dependency, collection hook, vendor identity | Inspection of both executable payloads |
| Tor transport | Existing component CI; main b89990d has 10 successful triggered workflows | Deno descendants network/DNS observation |
| Windows retrieval | Earlier fail-closed probe only | Positive bounded media retrieval |
| User demo | Pre-alpha components | Natural request -> choices -> file in one clean package |
| Funding readiness | Honest README, sponsorship and Apache-2.0 documentation | Runnable demo and complete binary licensing inventory |

Review this ADR after the trial; do not represent it as a completed runtime integration.
