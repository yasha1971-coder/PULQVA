# NEXT

## Verified base and PREPARE proof

T054 is merged. Main base for T055:
`dff37d8f4c2448e7c0ace03a02e28444e488ec6c`.

T055 PREPARE/prototype head:
`cd2ce4b47d6e5ad58ff31f0e13c89a0e3172e758`.

All 11 triggered workflows passed for that exact PREPARE head. That green proof validates the
test-only publication primitives, not completion of T055.

## Active task

**T055 — Atomically materialize the verified yt-dlp direct-binary artifact**

Phase: IMPLEMENT. T055 is NOT DONE.
Branch: `task/T055-atomic-ytdlp-sidecar-materialization`.
Draft PR: #57.

The current branch now adds the production internal boundary
`apps/pulqva-desktop/src-tauri/src/ytdlp_materialize.rs`.

Implemented behavior:

- input is the prepared app-runtime-directory capability, the verified package resource root, and one
  T054 verified artifact;
- only `BundledSidecarKind::YtDlp` is accepted;
- expected version and SHA-256 are resolved from backend-pinned yt-dlp metadata in production;
- artifact identity, source containment, source file type, receipt size, runtime destination, and
  source/destination aliases are revalidated before publication;
- `runtime/bin` is created only as a real app-owned directory and canonicalized beneath the
  prepared runtime root;
- a different or unsafe existing destination is preserved and rejected;
- a matching verified existing destination may be reused without mutation;
- copied bytes are streamed in bounded buffers into an exclusive staging file and SHA-256 checked
  again, so T054 path validation alone is never treated as an immutable byte snapshot;
- Unix executable permission is applied only to the owned staging file;
- the staged file is synchronized and published with no-clobber `hard_link`; no overwrite/rename
  fallback exists;
- staging cleanup checks the staging file identity before deletion;
- competing publishers may converge on the same verified destination;
- the production boundary does not run the executable, touch frontend authority, make network
  requests, or wire the live Download command.

Production-boundary tests cover success/reuse, source mutation, unsupported kind, identity mismatch,
escaped source, hard-link alias, preservation of a different destination, competing publishers, and
Unix symlink source/destination hazards. The earlier primitive proof remains as separate evidence.

## Trust limits that must not be lost

- FFmpeg's pinned digest authenticates its archive, not an extracted executable. Never route FFmpeg
  through this direct-binary materializer.
- Arti still has version metadata without a pinned executable content digest. Never route Arti
  through this boundary.
- Concurrent replacement of parent directories after validation is not claimed fully prevented.
- Windows ordinary CI does not prove adversarial reparse-point handling.
- `hard_link` requires filesystem support; unsupported publication fails closed rather than
  falling back to an overwriting operation.
- File `sync_all` is not a claim of crash-durable directory metadata on every filesystem.
- A post-publication staging-cleanup error must never trigger deletion of the verified destination.
- This task still does not install a real bundled yt-dlp resource in a release package.

## Recovery after first IMPLEMENT CI

IMPLEMENT head `75aea52626e5f70f5fd8a5864556ce7533470d0a` produced 10/11 green workflows.
Only `desktop-shell-check` failed, and only its Windows test job failed. Windows compilation passed.
Five production T055 tests all failed early with `ytdlp-materialization-source-escaped`.

Root cause: the test artifact source is canonicalized; on Windows that can carry the canonical
extended path form while the supplied resource-root path remains lexically non-canonical. The
containment helper compared those path spellings before canonicalizing the root, so valid contained
sources were rejected before their intended test paths ran.

The repair canonicalizes the trusted resource root first, accepts either the original trusted root
spelling or its canonical spelling as the containment base, then performs component/symlink
inspection from the canonical root and retains the final canonical containment check. This does not
weaken the escape or symlink fail-closed rules.

## Immediate next action

Inspect the exact repair head's triggered CI. If any check fails, diagnose only that concrete
failure. If all required checks are green, close T055 in a separate closeout phase; do not merge or
start T056 yet.

No Arti materialization, FFmpeg extraction, live command wiring, Tauri bundle activation, installer,
external provider, or direct-network fallback is introduced here.
