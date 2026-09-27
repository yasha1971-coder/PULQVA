# NEXT

## CURRENT: T068-B0 Windows local SOCKS fixture repair; native CI pending

Branch feat/T068-live-tor-e2e; PR #73 remains draft. Main is unchanged.
Last verified PR head b223f3a8825256c002b91bffd8508ecb9ba275be remains the anchor.
Parent 3c84e61182e8c2b7a700f843e32ce9aac085c034: 11/12 associated workflows
succeeded. Only discovery-https-candidate-check 36302674079 was red.
Linux job 108573162130 passed. Windows job 108573162014 resolved and verified the
candidate graph and compiled, then local read_exact failed with Winsock 10035 /
WouldBlock. The same helper caused the poisoned-environment child failure.

Repair is confined to the test fixture: explicitly set accepted TcpStream to
blocking mode before applying the existing finite read/write timeouts. Keep the
listener nonblocking for bounded accept and no-retry observation. A new regression
forces a nonblocking accepted stream on either OS and proves the reset honors an
idle-read timeout while the listener still returns WouldBlock without a new peer.
No production client, dependency pin, shipping manifest/lock or Tor policy changed.
Root cause/source review and exact publication head are checkpointed in PR #73.
Local rustc/Cargo remain absent; repaired Rust/native tests are NOT RUN locally.

ONE next action: inspect the repair head's discovery-https-candidate-check. Diagnose
red first. If green, retrieve BOTH platform candidate artifacts, verify receipt
hashes and compare Cargo-generated locks/features before adopting the graph with
the real endpoint-private CommonsTransport executor. Retire the one-off generator
when the real lock is imported. No blind dependency upgrade or parser expansion.

B0 is local dependency/SOCKS evidence, not successful TLS, live Tor, Commons search
or E2E. B still needs HTTPS positive/negative, streaming/deadline/liveness tests.
C must obtain real choices and download the selected file through the SAME
coordinator and existing retrieval boundary. Separate positive Windows/Linux and
fail-closed evidence remain required. T066 limitations and packaging gates remain.
