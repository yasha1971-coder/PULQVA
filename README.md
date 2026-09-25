# PULQVA

**Describe the file you need. Keep control of how you find it.**

PULQVA is building a desktop journey from a natural-language request to a useful file:
**request → private search → real choices → download**, with an optional Autopilot.

The goal: one package for Windows or Linux, no account or API key, and external
requests routed through Tor. These are product requirements; the complete packaged
journey is still under development.

[Support the maintainer](https://github.com/sponsors/yasha1971-coder) ·
[What support enables](SPONSORSHIP.md) · [Privacy boundaries](THREAT_MODEL.md)

## What exists today

- A Rust core and a Tauri desktop foundation.
- Typed boundaries for requests, sidecar identity verification and process launch.
- Arti/Tor transport and fail-closed checks.
- Automated Windows/Linux checks, including bounded live network probes.

**Pre-alpha, not a ready-to-use privacy product.** Complete natural-language search,
Autopilot and zero-configuration release packages are not yet demonstrated end to end.
Component tests are not an independent security audit. Live network checks can fail;
see [current checks](https://github.com/yasha1971-coder/PULQVA/actions) and [NEXT.md](NEXT.md)
for the current engineering state.

## Why support this work?

Privacy tools should be usable without a terminal, a service account or a setup guide.
PULQVA aims to connect intent, discovery and file retrieval behind a verifiable
privacy boundary.

Support can help advance three concrete priorities:

1. A real request-to-file journey that someone else can reproduce.
2. Self-contained Windows/Linux packages tested on clean machines.
3. Independent review of routing, dependencies and failure handling.

[Support development](SPONSORSHIP.md). Technical review and pilot feedback are welcome too.

## Privacy has boundaries

Tor routing does not make every action anonymous. A remote AI service can read text
submitted to it; downloaded files may contain tracking or malicious content. PULQVA
does not yet provide an audited safety guarantee for high-risk use.
Read the [threat model](THREAT_MODEL.md).

## License

PULQVA's original code and documentation are licensed under [Apache-2.0](LICENSE).
Third-party components keep their own licenses. Packaged FFmpeg and yt-dlp binaries
require separate distribution checks; see [THIRD_PARTY.md](THIRD_PARTY.md).

## Development

The repository is the source of truth for ongoing work. Start with [AGENTS.md](AGENTS.md),
[kernel/](kernel/), [PROJECT_STATE.json](PROJECT_STATE.json) and [NEXT.md](NEXT.md).

Run the continuity check:

```bash
python3 scripts/continuity_guard.py
```
