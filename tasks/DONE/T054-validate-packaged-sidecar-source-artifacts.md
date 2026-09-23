# T054 — Validate packaged sidecar source artifacts before materialization

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-23

## Result

Added a backend-only local source-validation boundary over the T053 resolved plan. The resource
root must be a real directory; source path components reject symlinks and invalid file types.
Canonical source files must remain beneath the canonical resource root. The plan requires exactly
one Arti, yt-dlp, and FFmpeg entry.

yt-dlp and FFmpeg source bytes are checked against their repository-pinned SHA-256 values using
streaming reads. Arti retains its pinned version metadata and receives path/file-type checks only;
this task does not authenticate its binary contents or execute it to confirm the version.

Validated results retain sidecar kind, canonical source, destination, identity, and byte size.
The desktop blocking task validates these sources before preparing runtime directories or starting
sidecars. The validation itself does not copy bytes, alter permissions, or launch any process.

Corrected the FFmpeg identity mapping: the stored digest is for the pinned platform archive, not
for the extracted executable. Its package source now names that archive. Future materialization
must verify and extract it, never copy the archive to `ffmpeg(.exe)`.

## Verification

PR #56 implementation head: `a1caa4eb310e472333eb4263223b28c0ff22327b`.
All 11 triggered workflows were observed completed/success for that exact head:

| Workflow | Run ID |
| --- | --- |
| continuity-guard | 35887078818 |
| rust-check | 35887078921 |
| desktop-shell-check | 35887078870 |
| arti-sidecar-check | 35887078839 |
| arti-materialization-check | 35887078915 |
| arti-config-contract | 35887078871 |
| arti-lifecycle-check | 35887078858 |
| tor-readiness-check | 35887078894 |
| ytdlp-sidecar-check | 35887078889 |
| ytdlp-tor-metadata-check | 35887078879 |
| ytdlp-tor-media-check | 35887078913 |

The desktop workflow executes tests on Windows and Linux. New T054 regression cases exercise
contained regular-file fixtures with matching digests, a hash mismatch, and a non-file source.
This is not proof of an installed three-sidecar release package. The closeout SHA has separate CI
and must pass before PR #56 merges; see its latest checkpoint.

## Retained limitations

Verified paths are not immutable snapshots. A later copying boundary must verify the bytes it
actually publishes. Arti content authentication, FFmpeg extraction, installed-executable validation,
and release packaging are not completed by this task.
