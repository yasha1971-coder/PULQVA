# T058 — Atomically materialize the verified Arti direct-binary artifact

Parent: DESKTOP FOUNDATION  
Status: READY

## Goal

Publish the verified packaged Arti executable into its app-owned `runtime/bin/arti(.exe)`
destination without partial bytes or clobbering an unrelated existing file.

## Acceptance criteria

- input is the prepared app-runtime-directory capability plus one T057-verified Arti artifact;
- destination must equal the layout's platform-specific Arti executable under `runtime/bin`;
- require the pinned Arti version and platform SHA-256 established by T057;
- revalidate source containment, source type, receipt size, destination containment, and file
  identity before mutation;
- copy only into an exclusively created staging file inside verified `runtime/bin`;
- hash the bytes actually copied and require the T057 digest;
- on Unix, apply executable permission only to the owned staging file;
- synchronize the complete staged file before no-clobber publication;
- allow reuse only of an existing regular destination whose content matches the pinned Arti digest;
- reject unsafe/different existing destinations without chmod, overwrite, deletion, or truncation;
- clean only owned staging files on failure;
- no shell command, network access, Tor bootstrap, process launch, or frontend filesystem authority;
- deterministic Windows/Linux tests cover success/reuse, source mutation, hash mismatch, unsafe
  paths, aliases, competing publishers, and existing-file preservation;
- all privacy/media/desktop checks remain green.

## Trust limits

Do not claim complete protection against all concurrent parent-directory replacement or adversarial
Windows reparse behavior unless separately proven. Preserve the fail-closed behavior on unsupported
publication filesystems.

## Out of scope

Wiring Arti materialization into the live completed-file command, changing Tor lifecycle/readiness,
FFmpeg extraction, Tauri bundle activation, installer/release packaging, external providers, AI
providers, and direct-network fallback.
