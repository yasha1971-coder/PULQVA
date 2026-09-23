# T054 — Validate packaged sidecar source artifacts before materialization

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Validate the T053 resolved packaged sidecar source files locally and fail closed before any executable
bytes can be materialized into the app-owned runtime tree.

## Acceptance criteria

- input is a T053 `ResolvedBundledSidecarMaterializationPlan`;
- each source exists and is a real regular file;
- symlink and non-file sources fail closed;
- canonical source paths remain beneath the verified package resource root;
- yt-dlp and FFmpeg source bytes match their existing pinned SHA-256 identities;
- Arti preserves its pinned version identity and passes the same containment/file-type checks;
- validation returns typed verified source artifacts with unchanged runtime destination and identity
  metadata;
- no runtime destination is created or modified;
- no chmod occurs;
- no process is launched;
- no external network access occurs;
- Windows and Linux desktop tests remain green;
- existing privacy/media/remux checks remain green.

## Out of scope

- actual sidecar binary copy/materialization;
- installer/release packaging;
- external candidate search;
- AI provider integration.
