# NEXT

## Current verified state

T008 is complete.

Arti sidecar build proof is green on both kernel release targets:

- official `arti` binary version pinned to `2.6.0`;
- Rust `1.91.0`;
- Ubuntu 24.04 build/install succeeds;
- Windows build/install succeeds with Arti's `static-sqlite` feature;
- `arti --version` contains `2.6.0`;
- `arti help proxy` succeeds on both targets;
- no Tor bootstrap or runtime network connection was started;
- continuity and workspace Rust checks are green.

Verified PR head:
`756b6c574c80a8cc99b22261cb546838ddebb85b`

Verified Arti run:
`35519121511`

## Next atomic task

**T009 — Add a typed Arti sidecar launch specification**

Add a pure-data launch specification for the pinned Arti sidecar:

- executable identity/path input;
- argument vector for the proxy subcommand;
- explicit data/state directory inputs;
- no process spawn;
- no Tor bootstrap;
- no sockets/network.

This prepares a narrow boundary for the later runtime launcher while keeping process execution and
privacy-sensitive runtime behavior out of scope.

## Do not do yet

- no process spawn;
- no Tor bootstrap/network;
- no dynamic SOCKS-port discovery;
- no yt-dlp;
- no HTTP;
- no AI provider;
- no UI.

## Success

The launch specification is typed, deterministic, platform-neutral at the core boundary, covered
by tests, and all existing checks remain green.
