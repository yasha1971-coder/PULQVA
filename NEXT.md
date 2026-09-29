# NEXT

## CURRENT: T068 active — Windows Tor transport prerequisite is proven

T067 exact head `51a21c476d49738184678bb6e5c3d8ff29be68fd` proved deterministic typed
request -> >=2 choices -> explicit selection -> bounded local file/receipt orchestration.
It did not prove live Internet retrieval.

On 2026-09-29, a real Windows host removed the Tor transport blocker without weakening
Windows security:

- Smart App Control remained in ENFORCE mode;
- local Cargo/Arti compilation was rejected by Windows Code Integrity event 3077;
- official Tor Project Expert Bundle 15.0.23 was hash-verified;
- `tor.exe` 0.4.9.12 executed successfully;
- Tor reached bootstrap 100%;
- loopback SOCKS5 became ready;
- the Tor Project check API returned `IsTor=true`;
- the bounded Tor process was stopped after the proof.

ADR-0007 therefore selects the pinned official Tor Expert Bundle for the Windows
production sidecar. No end-user Arti compilation, no disabling Smart App Control, and
no silent runtime download of executable components.

The architecture remains unchanged above the sidecar boundary:
frontend networkless, Tor fail-closed, remote DNS through SOCKS, and external adapters
authorized only by `ReadyTorTransport`.

T068 is **not done yet**. Its acceptance still requires the SAME T067 coordinator to
perform one bounded live path:

`natural-language request -> >=2 real external choices -> explicit selection -> real file -> receipt`.

Current-world review continues to keep YouTube/EJS/POT/SABR outside this first generic
live proof. Use a stable small public fixture with no account/API key/JS challenge.

## ONE next action

Implement the pinned Windows official-Tor runtime beneath the existing privacy boundary:
materialize the verified packaged runtime, launch it hidden, certify its loopback SOCKS
endpoint into the existing `ReadyTorTransport`, and then drive the unchanged T067
coordinator through the first real request -> choice -> file fixture.

Do not add a clearnet fallback. Do not fetch Tor at application runtime. Do not mark
T068 complete until the selected real file and typed receipt exist and the Tor-unavailable
negative path still fails closed.
