# ADR-0004 — Use the official Arti binary as the Tor sidecar

Status: Accepted for the current Linux path; superseded for Windows by ADR-0007  
Date: 2026-09-20

## Context

PULQVA's kernel requires Tor-by-default, fail-closed external networking and a zero-configuration
user package. Future consumers such as yt-dlp need a SOCKS endpoint.

T008 tested the current official Arti binary package before adding runtime behavior.

## Decision

Use the official `arti` binary as PULQVA's Tor transport sidecar, pinned independently from the
PULQVA core, for the existing Arti implementation path.

Historical verified baseline:

- Arti: `2.6.0`;
- Rust build baseline for this proof: `1.91.0`;
- Linux: default locked Arti build;
- Windows: locked Arti build with `static-sqlite`.

The sidecar boundary is preferred over embedding Arti directly into the application core at this
stage.

On 2026-09-29, real-machine Windows evidence showed that Smart App Control in ENFORCE mode blocked
Cargo-generated build-script executables during local Arti installation even though Rust itself
worked. ADR-0007 therefore supersedes this ADR for the Windows production sidecar and selects the
pinned official Tor Project Expert Bundle. This ADR remains the historical Arti decision and the
currently verified Linux Arti path until a separate Linux migration is proven.

## Evidence

GitHub Actions run `35519121511` passed on:

- Ubuntu 24.04;
- Windows.

The jobs built the exact pinned binary and verified `arti --version` and `arti help proxy`
without starting a Tor proxy or bootstrapping the Tor network.

Later Windows reproducibility work is retained under `sidecars/arti/` as historical build evidence.
It must not be interpreted as a requirement to compile Arti on an end-user Windows machine.

## Consequences

- Tor transport remains behind the sidecar boundary and can be upgraded independently from the core.
- Network adapters continue to consume the existing `ReadyTorTransport` SOCKS capability.
- Linux may continue using the verified Arti path until separately migrated.
- Windows production packaging follows ADR-0007 and must not require local Cargo/Arti compilation.
- Runtime launch, readiness, fail-closed enforcement and leak testing remain separate gates.
