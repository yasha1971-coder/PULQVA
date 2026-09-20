# ADR-0004 — Use the official Arti binary as the Tor sidecar

Status: Accepted  
Date: 2026-09-20

## Context

PULQVA's kernel requires Tor-by-default, fail-closed external networking and a zero-configuration
user package. Future consumers such as yt-dlp need a SOCKS endpoint.

T008 tested the current official Arti binary package before adding runtime behavior.

## Decision

Use the official `arti` binary as PULQVA's Tor transport sidecar, pinned independently from the
PULQVA core.

Current verified baseline:

- Arti: `2.6.0`;
- Rust build baseline for this proof: `1.91.0`;
- Linux: default locked Arti build;
- Windows: locked Arti build with `static-sqlite`.

The sidecar boundary is preferred over embedding Arti directly into the application core at this
stage.

## Evidence

GitHub Actions run `35519121511` passed on:

- Ubuntu 24.04;
- Windows.

The jobs built the exact pinned binary and verified `arti --version` and `arti help proxy`
without starting a Tor proxy or bootstrapping the Tor network.

## Consequences

- Tor can be upgraded independently from the PULQVA core.
- The application can enforce a single SOCKS boundary for external tools.
- Packaging must include the platform-specific Arti binary.
- Windows builds must retain a self-contained SQLite strategy; the verified baseline uses
  `static-sqlite`.
- Runtime launch, readiness, fail-closed enforcement and leak testing remain separate tasks.
