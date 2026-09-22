# NEXT

## Current verified state

T042 is the last closed task.

T043 implementation head `7eb8834b4dc0bb07ac8e10dde4dea7fa5ffbb0ab` previously passed all 11
required workflows, but the later closeout head repeatedly failed the Ubuntu real-media proof. T043
is therefore active again until the repaired exact head is green.

## Active recovery

**T043 — backend-only FFmpeg remux planning boundary**

The FFmpeg planning implementation is unchanged. Recovery is limited to the external Tor media proof:

- previous proof fixture: immutable GitHub raw media URL;
- observed closeout failures: one timeout, then two Ubuntu yt-dlp exit-status 1 failures;
- Windows remained green;
- all non-media workflows remained green;
- replacement proof fixture: the 239,482-byte Wikimedia Commons original
  `Five-second_counter.webm`;
- Tor gating, typed yt-dlp request construction, bounded execution, artifact-size verification, and
  fail-closed behavior remain unchanged;
- no direct-network fallback is added.

Branch:
`task/T043-ffmpeg-remux-planning-boundary`

## Next task

T043 remains next until this repaired exact head is verified green.

## Do not do yet

- do not queue or start T044 until T043 closes green;
- no completed remux result surfacing to frontend;
- no external search provider;
- no AI provider;
- no packaging/release installers;
- no direct-network fallback.

## Success

All 11 required workflows are green for the exact repaired T043 head.
