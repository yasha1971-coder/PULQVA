# PULQVA

## Tell AI what you want. Get the file.

**Intent → Object. Privacy by design. Not by promise.**

> **What if AI could search the Internet for you without turning you into the product?**

PULQVA is an open-source experiment in a different interface to the Internet: start with human intent, not a URL. The product goal is simple — describe what you want, see real choices, choose, and receive the actual file — while the architecture makes privacy constraints explicit and fail-closed instead of asking you to trust a slogan.

**AI without surveillance. Internet without browsing. Intent without URLs.**

**По-русски:** скажи ИИ, что тебе нужно → получи реальные варианты → выбери → получи сам файл. Приватность должна обеспечиваться архитектурой, а не обещанием. [Подробнее](#по-русски).

> **Pre-alpha. The Linux backend request → choice → file path has now been reproduced on two fresh GitHub runners and on the owner's WSL2 laptop.**
> The complete AI-powered, one-click Windows/Linux product is **not ready yet**.
> Native Windows live Tor readiness is still open, and the successful trace uses a fixed test query rather than the finished AI/UI journey.

**[→ Help build PULQVA](CONTRIBUTING.md)** — solve native Windows/Tor · attack the privacy assumptions · build the Intent → Object future

[Verified result](#verified-result) · [Planned experience](#planned-experience) · [Current status](#current-status) · [Related projects](#related-projects) · [Privacy](#privacy)

## Planned experience

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

**On 2026-09-28, one Linux backend run completed the real combined journey:**

```text
"countdown" → Commons search over Tor → 10 real choices
            → choose the second result (index 1)
            → download that selected file through the existing Tor/yt-dlp path
            → verify size and SHA-1 → save selected.webm + receipt.json
```

The query and selection were fixed test inputs. They were not an AI interpretation,
a user-interface interaction, or a hardcoded media URL. The downloaded file was
2,131,934 bytes. Its size and SHA-1 matched the selected Commons metadata;
SHA-256 identified the saved bytes. The artifact was downloaded from Actions and
checked independently, including the ZIP digest, receipt source commit and file hashes.

**Evidence is in the experimental branch, not a merged feature in `main`:**
[implementation commit `0ed2af1`](https://github.com/yasha1971-coder/PULQVA/tree/0ed2af16d8a83194d8a4e967a3989fa2c2649452),
[Linux job and verification log](https://github.com/yasha1971-coder/PULQVA/actions/runs/36420551778/job/108921941999),
[artifact: video + receipt](https://github.com/yasha1971-coder/PULQVA/actions/runs/36420551778/artifacts/10969217080),
and [active work / evidence record in PR #73](https://github.com/yasha1971-coder/PULQVA/pull/73).
The Actions artifact may require GitHub sign-in and is scheduled to expire on
2026-10-05. **It is a test result, not the PULQVA application download.**

<details>
<summary>Exact saved-file identity</summary>

```text
Source commit: 0ed2af16d8a83194d8a4e967a3989fa2c2649452
File bytes:    2131934
File SHA-1:    a785d429082eab4ff173d3d9ec2577f8efe84eb6
File SHA-256:  0d77b81c7670ff7766240766a83a7fab4a3ad3aeb81d72f039113285b0acf423
ZIP SHA-256:   a7a913fc3f6d50912b29f89b6942bd8e1d4a60f5c35acb09423b4a0a7911faad
```

SHA-1 is compatibility with provider metadata, not publisher authentication.
SHA-256 is local content identity, not proof that a file is safe.

</details>

## Current status

Status of the scoped development evidence above, as of **2026-09-28**:

| Capability | What is established |
| --- | --- |
| Real Linux request → choices → selected file | One positive backend run, with the saved artifact independently checked. |
| Reproducibility on a second fresh environment | Not yet verified. A single success is not a reliability guarantee. |
| Windows live path | Compilation/contracts passed, but the strict live discovery check failed. Windows file E2E remains open. |
| Natural-language AI interpretation, UI and Autopilot | Full user journey not yet verified. |
| Self-contained, zero-configuration Windows/Linux package | Release goal; not yet validated on clean machines. |
| YouTube compatibility and security hardening | Separate open gates; a Commons download does not establish them. |

**Can I use it now?** Developers can inspect the source and evidence. Nontechnical
users should wait for a documented, clean-machine-tested release. GitHub's
**Code → Download ZIP** gives source code, not a ready-to-run application.

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

## По-русски

**PULQVA — разрабатываемое приложение: описал нужный медиаконтент, увидел реальные
варианты, выбрал и получил файл через Tor.** Пользователю не должно требоваться
самостоятельно искать ссылку, устанавливать зависимости или настраивать прокси.

Уже подтверждён один настоящий технический проход в Linux: запрос `countdown` →
10 результатов Commons → выбор второго → скачанный файл → проверка размера и
хешей. Сам файл сохранён и проверен отдельно от CI. Это результат из экспериментальной
ветки PR #73, а не готовая версия приложения в `main`.

**Пока не подтверждены:** повторный чистый прогон, полный Windows-сценарий,
ИИ-интерпретация вместе с интерфейсом и автономный пользовательский пакет.
«Один архив, открыл и пользуешься» — цель выпуска, а не доступная сейчас функция.
Аналоги отдельных частей и комбинаций существуют; заявления «аналогов нет» здесь нет.

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
