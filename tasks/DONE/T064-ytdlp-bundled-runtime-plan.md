# T064 — Typed bundled Deno selection in yt-dlp launch plans

DONE in typed-plan scope at db06221939d50b268c4a57f3406bba4a7bc27532.
All 11 triggered workflows passed. Metadata workflow 36216469372:
Windows job 108333306568 and Linux job 108333306720 each executed 26 ytdlp_
library tests successfully. Logs confirm native OS path rejection tests and
bundled_runtime_preserves_complete_media_and_metadata_vectors on both systems.

Acceptance evidence:
- Default Disabled resets JS runtimes and remote components; no host fallback.
- Explicit BundledDenoPath emits reset, one deno:absolute-path selection, then
  no remote components. Spaces/metacharacters remain one native argument.
- Missing input, relative, control, non-Unicode and native ambiguous/network
  path cases rejected by syntax tests. This is NOT file existence verification.
- Base/media/metadata plans suppress plugins and preserve ignore-config,
  ReadyTorTransport and socks5h route. Full argv and process fixtures tested.
- Materialization/hash/ownership/environment obligations documented separately.

Linux live metadata succeeded; Windows returned its existing fail-closed marker,
not positive retrieval. No live Deno/EJS/YouTube, binary verification or runtime
confinement claim. Explicit bundled selection is not enabled by current callers.
PR #69 must pass closeout exact-head CI before merge. T065 follows after merge.
