# NEXT

## CURRENT: T067 verified; T068 is the next live vertical gate

T067 exact head 51a21c476d49738184678bb6e5c3d8ff29be68fd passed all 10 PR
workflows. It proves deterministic typed request -> >=2 choices -> explicit
selection -> bounded local file/receipt orchestration. It does NOT prove Internet,
Tor search or live download.

Current-world review 2026-09-27 preserves the architecture: keep frontend networkless,
Tor fail-closed and remote DNS; do not make YouTube the generic first-live criterion.
yt-dlp/Deno/EJS compatibility remains a subsequent adapter gate; remote executable
components must not be silently fetched.

T068 READY: use the SAME T067 coordinator for one bounded live Tor-backed vertical
fixture: natural-language request -> real externally obtained candidate choices ->
explicit selection -> real downloaded file -> receipt. Choose a stable small public
fixture that does not require account/API key/JS challenge. All external operations
must cross the existing ReadyTorTransport/privacy boundary. No clearnet fallback.

ONE next action: PREPARE T068 by selecting and documenting the stable live fixture
and exact existing adapters required. Do not implement or launch T068 in the T067
closeout phase.
