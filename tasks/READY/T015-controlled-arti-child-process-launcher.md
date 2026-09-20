# T015 — Add a controlled Arti child-process launcher

Parent: PRIVACY FOUNDATION  
Status: READY

## Goal

Add the first process side effect while keeping the prepared-runtime capability as the mandatory
boundary and keeping Tor bootstrap untriggered.

## Acceptance criteria

- launcher accepts `PreparedArtiRuntime`, not `ArtiRuntimePlan`;
- explicit Arti executable and deterministic argument vector are used directly;
- no shell invocation is used;
- successful launch returns typed `RunningArti`;
- child PID/handle is retained;
- deterministic stop/wait cleanup exists;
- config keeps `defer_bootstrap = true`;
- tests use a controlled local fixture/process strategy on Windows and Linux;
- no SOCKS request is sent;
- no DNS request is sent;
- no external network request is made;
- no Tor bootstrap is intentionally triggered;
- all existing checks remain green.

## Out of scope

- SOCKS readiness probing;
- dynamic port discovery;
- yt-dlp/FFmpeg;
- AI;
- UI.
