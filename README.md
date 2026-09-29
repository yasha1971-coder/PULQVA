# PULQVA

## Tell AI what you want. Get the file.

**Intent → Object. Privacy by design. Not by promise.**

> **What if AI could search the Internet for you without turning you into the product?**

PULQVA is an open-source experiment in a different interface to the Internet: start with human intent, not a URL. The product goal is simple — describe what you want, see real choices, choose, and receive the actual file — while the architecture makes privacy constraints explicit and fail-closed instead of asking you to trust a slogan.

**AI without surveillance. Internet without browsing. Intent without URLs.**

Скажи ИИ, что тебе нужно → получи реальные варианты → выбери → получи сам файл. Приватность должна обеспечиваться архитектурой, а не обещанием. [Подробнее](#по-русски).

> **Pre-alpha. Real backend evidence exists; the consumer product does not yet.**
>
> **Verified Linux fixture:** the same fixed `countdown → 10 real choices → selected file` path succeeded twice on fresh GitHub runners. The owner then reproduced the same saved-file identity on an Ubuntu 24.04 WSL2 laptop and opened the resulting WebM.
>
> **Open gate:** native Windows live Tor reaches the local SOCKS endpoint and completes SOCKS5 negotiation, but the hosted-Windows run times out at Tor destination readiness before Commons HTTPS or file retrieval.
>
> **Not shipped:** finished AI intent parsing, desktop UI/Autopilot, native Windows file E2E, self-contained Windows/Linux packages, or a claim of absolute anonymity.

**[See the evidence →](#verified-result)** · **[See what remains →](#current-status)**
## Build PULQVA with us

**Solve native Windows/Tor · Break our privacy assumptions · Build Intent → Object**

PULQVA is being built in public. The most useful contribution today is not hype — it is evidence: reproduce a failure, find a privacy hole, or make the path from human intent to a real file smaller and safer.

**[→ Start contributing](CONTRIBUTING.md)**

[Product vision](#product-vision) · [Verified result](#verified-result) · [Privacy](#privacy) · [Related projects](#related-projects)

## Product vision

For someone who wants to find and save media without hunting for links or learning download commands:

```text
Open PULQVA
    → describe the content: "Find a short countdown video"
    → review real search results
    → choose one and click Download
    → receive the file in a predictable local folder
```

**This is the product goal, not a claim that the whole interface works today.**
The intended release is one self-contained package per supported OS: unpack/open,
then use it without installing Python, Rust, Tor, yt-dlp, FFmpeg or an AI model yourself.
No account or API key should be required for the default journey. An explicit
Autopilot mode may choose a result for you.

The planned intent layer interprets a phrase into validated structured data, not
executable shell text. The trusted backend owns search and retrieval; Arti provides
the Tor transport and yt-dlp handles media downloads. A required Tor connection
failure must stop the operation, never silently switch to a direct connection.
The model/provider choice and full no-setup intent path are still open work.

## Verified result

The strongest evidence today is deliberately narrower than the product vision.

```text
fixed request "countdown"
    → Commons search through the PULQVA Tor transport
    → 10 real choices
    → explicit choice of result #2
    → core retrieve_choice()
    → yt-dlp through the existing Tor path
    → size + SHA-1 verification
    → selected.webm + receipt.json
```

**Reproduction record**

| Environment | Result |
| --- | --- |
| Fresh GitHub Linux runner #1 | PASS — retained artifact independently checked |
| Fresh GitHub Linux runner #2 | PASS — same source, different runner, byte-identical saved media |
| Owner laptop, Ubuntu 24.04 / WSL2 | PASS — owner-reported replay; same file size/SHA-1/SHA-256 and the resulting WebM opened successfully |
| Native Windows | **OPEN** — hosted run reaches SOCKS5 negotiation, then `DestinationTimeout` before Commons HTTPS |

All three positive observations used exact experimental source `0ed2af1` and the fixed test query/selection. They are **backend evidence**, not AI/UI interaction and not a released application.

Saved media identity from the fixture:

```text
File bytes:    2131934
File SHA-1:    a785d429082eab4ff173d3d9ec2577f8efe84eb6
File SHA-256:  0d77b81c7670ff7766240766a83a7fab4a3ad3aeb81d72f039113285b0acf423
```

**Evidence:** [implementation `0ed2af1`](https://github.com/yasha1971-coder/PULQVA/tree/0ed2af16d8a83194d8a4e967a3989fa2c2649452) · [GitHub run](https://github.com/yasha1971-coder/PULQVA/actions/runs/36420551778) · [PR #73 evidence record](https://github.com/yasha1971-coder/PULQVA/pull/73) · [owner-laptop record](https://github.com/yasha1971-coder/PULQVA/pull/73#issuecomment-5874201467) · [Windows diagnosis](https://github.com/yasha1971-coder/PULQVA/pull/73#issuecomment-5874285950)

The GitHub Actions artifacts are temporary test evidence, **not an application download**. SHA-1 is compatibility with provider metadata; SHA-256 identifies the local bytes. Neither proves that media is safe.

## Current status

| Product gate | Status |
| --- | --- |
| Linux backend: request → real choices → selected file | **Reproduced** for the fixed Commons fixture |
| Native Windows backend file E2E | **Open** — Tor destination readiness is the current blocker |
| Natural-language AI interpretation | **Open** — product goal, not part of the verified fixture |
| Desktop UI / explicit choice / Autopilot | **Open** |
| Tor fail-closed behavior across the complete product journey | **Partially evidenced; broader failure paths remain open** |
| Self-contained Windows/Linux package | **Open** |
| Clean-machine consumer release | **Not shipped** |

**Can I use it now?** Developers can inspect the source and reproduce the experimental path. Nontechnical users should wait for a documented clean-machine release. GitHub's **Code → Download ZIP** is source code, not the PULQVA application.

## Privacy

Tor routing is not absolute anonymity. Remote services can see the requests sent to
them; a remote AI provider can read submitted text. Downloaded files may contain
tracking or malicious content. A cryptographic hash does not make media safe.

The first positive test does **not** establish product-level cancellation during a
download, handling of mid-flight Tor loss, confinement of all child-process traffic,
bounded download disk usage, or race-proof file ownership. These remain engineering
and review work, not hidden guarantees. Read [THREAT_MODEL.md](THREAT_MODEL.md) and
[the privacy invariants](kernel/PRIVACY_INVARIANTS.md). This is not an audited product
for high-risk use. Use only content you are authorized to download.

## Related projects

PULQVA builds on existing work. **It does not claim to be the first or only tool
combining AI, downloading or portable distribution.** These primary sources show
real overlap; this is a documentation comparison, not a comparative execution audit.

| Project | Relevant documented overlap |
| --- | --- |
| [Parabolic](https://github.com/NickvisionApps/Parabolic) | Graphical yt-dlp frontend. Its [releases](https://github.com/NickvisionApps/Parabolic/releases) include a Windows portable ZIP launched with a `.bat` file. |
| [FreeTube](https://docs.freetubeapp.io/about/freetube/) | Desktop YouTube browsing without a Google account; its [Tor integration](https://docs.freetubeapp.io/usage/tor/) requires an externally running Tor client. |
| [VidSnatch](https://github.com/sahajamit/VidSnatch) | Search, result selection, downloads, a web interface and MCP tools for AI assistants; its documented setup requires Python and FFmpeg for relevant operations. |
| [ytdlpllm, author's description](https://blog.maxrenke.com/posts/ytdlpllm-A-Natural-Language-Youtube-Downloader/) | An existing natural-language → LLM → yt-dlp approach, with a web UI and package-install setup. |

PULQVA's intended distinction is the **whole default experience**: request-first
interaction, visible choices, a Tor-routed fail-closed backend, no default account/API
key, and one self-contained package. The complete combination is still a goal here;
this review neither proves it is unique nor proves another project meets every requirement.

## Support and development

[Support the maintainer](https://github.com/sponsors/yasha1971-coder) ·
[What support enables](SPONSORSHIP.md). Technical review and pilot feedback are welcome.
The next priorities are repeatability, verified Windows behavior, self-contained
packages and independent privacy review.

For contributors and AI agents, read [AGENTS.md](AGENTS.md), [kernel/](kernel/),
[PROJECT_STATE.json](PROJECT_STATE.json) and [NEXT.md](NEXT.md). State files describe
their own branch snapshot; consult [PR #73](https://github.com/yasha1971-coder/PULQVA/pull/73)
for the newer experimental T068 evidence. Do not infer feature completion from a
single workflow badge or from the product roadmap.

```bash
python3 scripts/continuity_guard.py
```

## License

PULQVA's original code and documentation are licensed under [Apache-2.0](LICENSE).
Third-party components keep their own licenses. Packaged FFmpeg and yt-dlp binaries
require separate distribution checks; see [THIRD_PARTY.md](THIRD_PARTY.md).
