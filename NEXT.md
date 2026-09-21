# NEXT

## Current verified state

T016 is complete.

The real pinned Arti 2.6.0 sidecar has been started and stopped through PULQVA's prepared-runtime
path on Windows and Linux with `application.defer_bootstrap = true`.

Verified PR head:
`1326fe255304fa72899d876f0b842260cac3f6b6`

Verified checks:

- arti-lifecycle-check: success;
- arti-materialization-check: success;
- rust-check: success;
- continuity-guard: success;
- arti-config-contract: success;
- arti-sidecar-check: success.

## Active atomic task

**T017 — Add a readiness-gated Tor transport capability**

External-network code must not be able to turn a raw SOCKS port into an approved route.

T017 introduces a public `ReadyTorTransport` capability with no public constructor. The raw
`TorSocksEndpoint` remains useful as internal runtime data, but its proxy URL is no longer exposed
as a public network-routing API.

A future readiness verifier inside `pulqva-privacy` will be the only path that can mint the ready
capability.

## Do not do yet

- no Tor bootstrap trigger;
- no external request;
- no SOCKS readiness probe;
- no dynamic port discovery;
- no yt-dlp/FFmpeg;
- no AI;
- no UI.

## Success

Future external-network adapters can be designed to require `ReadyTorTransport`, while callers
outside the privacy crate cannot manufacture that capability from raw endpoint data.
