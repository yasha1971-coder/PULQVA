# T066-R — diagnose repeated public media SOCKS failure

Status: DONE for diagnostic implementation/unit verification only; root cause OPEN.
Verified head711b294c6c3c3d07ce1ec6528a717bd02e7565f8, all11 workflows green.
Media36247336054: Linux108418847826 five tests passed and retrieval attempt1;
Windows108418847924 five tests passed, live fail-closed only.
No live final-failure probe markers emitted; historical recovery is NOT proven.
T066 feature work may resume from this verified checkpoint.

Failure head af6dd14f537ea1bcce559406f5add16dcc94e5ae, run36244931622,
Linux108412266354: generic SOCKS REP1 for all three attempts, 2s/4s backoff,
exit1. Root cause unknown. This is not the raw GitHub 20s read timeout.
B3 desktop native tests passed; no overall green claim.

Added opt-in final-failure diagnostics to the fixed public CI example only:
Arti process alive status, then SOCKS CONNECT to upload.wikimedia.org:443 and
example.com:443 through numeric loopback19050 with remote DNS ATYP3. No TLS/HTTP
payload, local hostname resolution, raw Arti logs or arbitrary destination input.
Each probe gets at most5s within the unchanged180s media deadline. It records
handshake stage failure or numeric REP; successful CONNECT is not media success.
Original exit remains failed. No retry/timeout/URL/privacy policy weakening.

Two tests cover exact fixed-host remote-DNS framing, invalid reply headers,
host rejection and expired deadline without network. Native CI pending.
Comparative probes may distinguish destination-specific from broader SOCKS
failure, but do not prove exit identity, throttling or Arti internal cause.
If next run succeeds, this does not prove historical failure resolved; inspect
whether the diagnostic branch actually ran before claiming new evidence.
