# NEXT

## CURRENT: T068-B real HTTPS executor implemented; first locked native CI PENDING

PR #73, branch feat/T068-live-tor-e2e, main untouched. Verified parent:
c6ce5494d6432604bbf30f8f486d01f358362d87, all 13 associated workflows success.
The historical Python/Windows/LF/CRLF recovery is CLOSED, not the current task.

This implementation uses the exact staged B0 graph: Cargo.lock blob
8279158c7f7fd5dbd243942669f8b4c929a8831b (33037 bytes, SHA256
ea282fedb7128d918b428cb30e5563b44075c770cb6672bffa682fe10091f5e5),
manifest blob 5563edc728a4eff70213be391105a31f2dcbb737. No re-resolution.

CommonsHttpsTransport is now library code, not an example. Its constructor verifies
readiness on the owned RunningArti it borrows; into_search reuses the SAME
CommonsSearch/CandidateSearch/T067 coordinator. Requests have an explicit loopback
socks5h route, certificate/name validation with pinned roots, no ambient proxy,
redirect/retry/cookie/referer/decompression/keylog, bounded streaming response,
whole-request deadline, sticky cancellation and child-liveness supervision.
The adapter is synchronous: invoke on a backend blocking worker. Existing readiness
bootstrap has a separate 90s budget; request cancellation does not interrupt bootstrap.

The old parser/tests were moved byte-for-byte to commons.rs. The maintained native
discovery-contract-check now tests the actual locked library on Windows/Linux.
Temporary copy-generators, their example clients and two duplicate candidate
workflows are retired after adoption, not used to append dependencies again.
Required SOCKS refusal/domain/poison assertions moved into the real executor's tests.
New tests additionally exercise real local TLS, wrong-name/untrusted certificates,
redirect/status/encoding/type rejection, chunked/oversized/truncated responses,
stall timeout and cancellation/liveness. All network fixtures are LOOPBACK ONLY.
Their private test roots/keys are cfg(test), never production trust overrides.

ONE NEXT ACTION: inspect the new exact-head discovery-contract-check and existing
CI. On red, diagnose the precise log before further edits. If green, assess remaining
B evidence (actual Arti loss/cancellation while a real request is in flight) and wire
the SAME coordinator to live Commons choices and existing retrieval for C. Do not
start another dependency candidate generator or claim a complete E2E from these tests.

Native compilation/tests PENDING; no local rustc/Cargo exists in this container.
Only source-byte/hash/JSON checks and test-certificate generation ran locally.
No live Commons request, downloaded file, positive Windows E2E, UI or ZIP proof.
T066 race/retention, descendant confinement and SDK-content limitations remain.
Current-world review and acceptance boundaries: decisions/ADR-0008-commons-https-executor.md.
Publication head/run IDs are recorded in PR73's checkpoint, not inferred from old CI.
