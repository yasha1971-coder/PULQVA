# PULQVA

**Tell it what you want. Get the file.**

A desktop media tool in development. Describe what you need, choose a result,
and save the actual file. That is the goal, not a finished feature list.

**Pre-alpha. No ready-to-run app yet.**

**[Sponsor the maintainer](https://github.com/sponsors/yasha1971-coder) · [PULQVA funding priorities](SPONSORSHIP.md)**

Support is optional. This is the maintainer's shared Sponsors profile; contributions
are not automatically earmarked for PULQVA and do not buy a released app.

[Current status](#current-status) · [Backend evidence](#verified-result) ·
[Privacy](#privacy) · [Contribute](CONTRIBUTING.md)

## What we're building

```text
Describe the media -> review real results -> pick one -> save the file
```

The intended default is local AI for interpreting the request, Tor for external
search and downloads, and no required cloud AI account or API key. Local inference
does not mean offline search: remote sources still receive search terms and download
requests. A Tor failure must stop retrieval, not switch to a direct connection.
The complete behavior is still under development.

**Target architecture:** Rust + Tauri + local llama.cpp + Tor + yt-dlp.
Windows and Linux are the release targets. The local intent work is in
[PR #80](https://github.com/yasha1971-coder/PULQVA/pull/80); the model is not a final product choice.

The release target is one self-contained package per OS. Users should not need to
install Python, Rust, Tor, yt-dlp, FFmpeg or a model separately. Autopilot is planned;
explicit result selection comes first.

## Current status

| Part | Status |
| --- | --- |
| Linux backend: request -> choices -> selected file | Reproduced for one fixed Commons fixture; evidence below |
| Native Windows Tor bootstrap and SOCKS route | Verified on a real host; this is not Windows file E2E |
| Native Windows request -> choices -> selected file | Open |
| Local AI intent handling | Experimental; not accepted as a finished feature |
| Desktop interface and Autopilot | Not shipped |
| Full-journey privacy and failure handling | Partially evidenced; review and failure cases remain |
| Self-contained Windows/Linux packages | Not shipped |

**Can I use it now?** The repository is for development and experimental reproduction.
There is no documented clean-machine consumer release. GitHub's **Code -> Download ZIP**
downloads source code, not the application.

## Verified result

The retained evidence covers a backend fixture, not the planned AI-driven desktop app.

```text
fixed query "countdown"
    -> Commons search through the PULQVA Tor transport
    -> 10 real choices
    -> explicit choice of result #2
    -> core retrieve_choice()
    -> yt-dlp through the existing Tor path
    -> size + SHA-1 verification
    -> selected.webm + receipt.json
```

| Environment | Recorded result |
| --- | --- |
| Fresh GitHub Linux runner #1 | PASS; retained artifact independently checked |
| Fresh GitHub Linux runner #2 | PASS; same source, different runner, byte-identical saved media |
| Owner laptop, Ubuntu 24.04 / WSL2 | Owner-reported replay; same size/SHA-1/SHA-256 and WebM opened successfully |
| Native Windows | Tor transport PASS; request -> choices -> selected file remains open |

The positive file observations used experimental source `0ed2af1` and the same fixed
query and selection. They do not establish arbitrary queries, AI interpretation,
desktop interaction or a released application.

```text
File bytes:    2131934
File SHA-1:    a785d429082eab4ff173d3d9ec2577f8efe84eb6
File SHA-256:  0d77b81c7670ff7766240766a83a7fab4a3ad3aeb81d72f039113285b0acf423
```

**File evidence:** [source](https://github.com/yasha1971-coder/PULQVA/tree/0ed2af16d8a83194d8a4e967a3989fa2c2649452) ·
[run](https://github.com/yasha1971-coder/PULQVA/actions/runs/36420551778) ·
[PR #73](https://github.com/yasha1971-coder/PULQVA/pull/73) ·
[owner replay](https://github.com/yasha1971-coder/PULQVA/pull/73#issuecomment-5874201467).

**Windows Tor evidence:** On 2026-09-29, the pinned official Tor Project Expert Bundle
was hash-verified and run with Smart App Control still in ENFORCE mode. Tor reached
bootstrap 100%, exposed loopback SOCKS5, and the Tor Project check returned `IsTor=true`.
See [ADR-0007](decisions/ADR-0007-windows-official-tor-runtime.md), the
[native receipt](sidecars/tor/WINDOWS_NATIVE_VERIFICATION.json), and the
[sidecar contract](sidecars/tor/README.md). The existing Linux path uses Arti; the
Windows sidecar baseline is the official Tor bundle.

Actions artifacts are temporary test evidence, not app downloads. SHA-1 is used for
compatibility with provider metadata; SHA-256 identifies the saved bytes. Neither
proves that a file is safe.

## Privacy

Privacy is a set of engineering constraints here, not a guarantee of anonymity.

The design requires external requests to use the approved Tor transport, remote DNS
to stay inside that path, and model output to pass typed validation before it can
influence a search. Model output must never become a shell command. See the
[privacy invariants](kernel/PRIVACY_INVARIANTS.md).

Tor does not make a request unidentifiable in every situation. Sources can see the
requests sent to them. A remote AI provider, if used, can read submitted text.
Downloaded files may contain tracking or malicious content.

Product-level cancellation during downloads, mid-flight Tor loss, confinement of
all child-process traffic, disk-growth limits and race-proof file ownership are
not established by the positive fixture. These are open engineering and review
work. Read [THREAT_MODEL.md](THREAT_MODEL.md).

**Not audited for high-risk use. Download only content you are authorized to save.**

## Related work

PULQVA is not the first tool to combine search, AI or media downloading.

- [Parabolic](https://github.com/NickvisionApps/Parabolic): a graphical yt-dlp frontend.
- [FreeTube](https://docs.freetubeapp.io/about/freetube/): desktop YouTube browsing without a Google account.
- [VidSnatch](https://github.com/sahajamit/VidSnatch): search, selection, downloads and AI-facing tools.
- [ytdlpllm](https://blog.maxrenke.com/posts/ytdlpllm-A-Natural-Language-Youtube-Downloader/): a natural-language-to-yt-dlp approach.

The goal here is request-first interaction, visible choices, local intent handling,
Tor-routed retrieval and a self-contained package. The whole combination is not
finished. This is not a comparative benchmark or a claim of uniqueness.

## Contributing

Useful work now: reproduce a failure, review the privacy boundary, or help finish
native Windows retrieval and packaging. See [CONTRIBUTING.md](CONTRIBUTING.md).

For branch-specific engineering state, read [AGENTS.md](AGENTS.md),
[PROJECT_STATE.json](PROJECT_STATE.json), [NEXT.md](NEXT.md), and the latest checkpoint
in the relevant PR. State files describe their branch snapshot; a green component
test does not mean the application is ready.

```bash
python3 scripts/continuity_guard.py
```

[Support development](https://github.com/sponsors/yasha1971-coder) ·
[What support funds](SPONSORSHIP.md). Supporting development is not buying a released app.

## По-русски

PULQVA — разрабатываемое приложение: описать нужное медиа, увидеть варианты,
выбрать и сохранить файл. Цель — локальный ИИ и поиск со скачиванием через Tor.
Готовой версии для обычного пользователя пока нет. Выше приведены проверенные
результаты отдельных частей и оставшиеся ограничения.

## License

Original code and documentation: [Apache-2.0](LICENSE).
Third-party components retain their own licenses. Bundled binaries require
separate distribution checks; see [THIRD_PARTY.md](THIRD_PARTY.md).
