# NEXT

## CURRENT: T068-B0 native HTTPS candidate VERIFIED; graph adoption prepared

Branch feat/T068-live-tor-e2e; PR #73 remains draft; main unchanged.
Exact head cd785a38eb66b11721a4d65788dd9ba332169719: all 12 associated workflows
succeeded, including discovery-https-candidate-check 36303595381 on Windows/Linux.
The Windows accepted-socket portability repair is therefore verified.

Both run artifacts were retrieved and inspected. Their generated Cargo.lock files are
byte-identical: SHA-256 ea282fedb7128d918b428cb30e5563b44075c770cb6672bffa682fe10091f5e5,
33037 bytes. Their candidate pulqva-discovery manifests are byte-identical:
SHA-256 c5e3083ce803118a1e6166628309af07bea194a45ecac2606f0abd96cb1401f6.
Both receipts pin reqwest 0.13.5, rustls 0.23.43, webpki-roots 1.0.9 and tokio 1.53.1,
and report preservation of existing registry identities. features.txt differs across
OS as expected from target-specific dependencies; do not require byte identity there.

ONE next action: adopt the exact generated manifest and Cargo.lock into the shipping
workspace together with the substantive endpoint-private CommonsTransport executor,
then launch one native B implementation CI. The executor must retain no_proxy +
explicit socks5h, HTTPS-only, HTTP/1, no redirects/retries/referer/decompression/
keylog, bounded streaming body and total deadline. Add controlled transport tests
for proxy refusal/remote hostname, TLS success+certificate/hostname rejection,
redirect/status/oversize/truncation/deadline and poisoned ambient proxy settings.
No generic caller URL/client override.

B0 remains local dependency/SOCKS evidence, not live Commons/Tor E2E. C remains the
same-coordinator real discovery -> explicit choice -> existing retrieval -> actual
file/digest receipt on positive Windows/Linux, plus fail-closed evidence.
