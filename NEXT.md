# NEXT

## T069-G2 — desktop zero-timeout assertion correction PENDING CI

Parent PR80 HEAD a3a934c6c04167ecb5b7874345bb590c4deb433b. Focused follow-up targets feat/T069-intent-contract from fix/T069-ffmpeg-monthly-source. Recovery envelope remains anchored to parent PR80; this follow-up does not replace that development route.

Adopted byte-verified September monthly snapshot in VERSION, SHA256SUMS, SOURCE_PROOF, both existing FFmpeg workflows and desktop resource paths/assertions. Kernel, Tor routing, TLS and archive verification unchanged. Source commit resolved through upstream API. Fresh two-OS CLI/remux and desktop tests PENDING; local Cargo/Windows unavailable. Archive identity is not execution acceptance.

Diagnosis checkpoint: PR85 comment6027034923, source HEAD 88c99d033df324bf0be597ec2dc717687a3732f2. desktop-shell-check run37392298919 failed on Ubuntu job112039977678 and Windows job112039977957 at the same stale message assertion. Duration::ZERO returns Listener; the privacy Display contract explicitly retains that stage. Corrected only the desktop expected string, preserving error-code, cleanup and no-cache/state/download assertions. Production behavior and Kernel unchanged. FFmpeg sidecar37392299425, real-remux37392298867, rust37392299039 and continuity37392299088 passed at the old head; those are scoped historical evidence, not new-head acceptance. Recovery seals updated for this pending correction; last_verified_commit preserved.

ONE NEXT ACTION: observe the published correction HEAD automatic desktop-shell-check on both OSes and continuity/recovery. Record exact run/job/head identities, inspect first concrete failure before another correction; no blind retry. Corrected native tests remain PENDING until observed. Local source review and Python continuity/recovery checks do not prove native execution. Retained intent->choice->file workflow remains NOT_TESTED. No merge/release/scheduler changes.
