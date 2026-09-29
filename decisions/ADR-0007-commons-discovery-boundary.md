# ADR-0007 — Endpoint-specific Commons discovery for T068

Date: 2026-09-27
Status: accepted for T068-A request/parser implementation; live transport and fixture availability NOT verified.

## Why this directly advances the kernel journey

T067 has a CandidateSearch coordinator and a deterministic writer fixture. Existing
Tor-gated yt-dlp probes prove retrieval from a fixed URL, not search. A fixed media
URL is not a substitute for externally obtained choices derived from the request.
The pinned Wikimedia yt-dlp extractor accepts File: pages and performs metadata
lookups; it does not expose Commons text search. Preserve yt-dlp for retrieval and
add the missing narrow discovery boundary rather than pretend it already exists.

## Decision

Use the MediaWiki Action API search generator in File namespace, deriving the
query from SearchIntent and appending the documented filetype:video filter.
Combine imageinfo URL/size/SHA-1/MIME in one bounded query. Retain public page ID
provenance. Accept only small WebM originals on the exact upload.wikimedia.org
HTTPS host/path. No injected canned media URLs, automatic continuation, account,
API key or new JavaScript runtime. Too few valid candidates is a visible failure.
The first adapter returns raw text matches, NOT general AI intent understanding.

The new pulqva-discovery adapter depends on existing core/privacy and already
locked serde libraries. It adds no HTTP client, new third-party dependency version,
or sidecar. Its public constructor requires ReadyTorTransport and its typed plan
has one endpoint, socks5h, a 60-second total request budget, a 256-KiB response cap,
zero redirects and a project User-Agent. Those transport policies are not yet an
implemented or tested HTTPS executor. The transport trait is a test seam, NOT a
network sandbox. No production CommonsTransport implementation exists in A.

Server-declared size and SHA-1 remain untrusted metadata, not a content-authenticity
or completed-file receipt. A live retrieval must enforce actual byte/time limits,
validate source/selection binding, compare downloaded content and publish only
confirmed completion. Redirects, DNS, TLS and process lifetime require execution
checks, not merely a correct plan. No anonymity guarantee is inferred.

## Current-world evidence reviewed today

- https://www.mediawiki.org/wiki/API:Search — generator usage and query parameters.
- https://www.mediawiki.org/wiki/API:Imageinfo — URL, size, SHA-1 and MIME metadata.
- https://www.mediawiki.org/wiki/Help:CirrusSearch — filetype filters.
- https://foundation.wikimedia.org/wiki/Policy:Wikimedia_Foundation_User-Agent_Policy — application identification, not browser impersonation.
- https://github.com/yt-dlp/yt-dlp/blob/3a08beaf031ab68f966401ead017ac81fe8486cf/yt_dlp/extractor/wikimedia.py — exact pinned File: extractor and API/User-Agent behavior.

Official documents were fetched. The attempted Commons API browser probe returned
no usable JSON. No claim of a successful current query or two live matches is made.
This evidence supersedes earlier speculative protocol/availability claims, not the
kernel. No Deno/Tauri/nightly upgrade is needed for this data-boundary phase.

## Remaining bounded phases

A: typed request, bounded parser, CandidateSearch integration, native local tests.
B: approved endpoint-specific Tor-only HTTPS executor; no host curl dependency,
ambient proxy fallback, direct DNS or redirect escape. Streaming limits, TLS and
Tor-loss failure must be exercised. Recheck current provider documentation first.
C: the same coordinator plus existing yt-dlp retrieval, at least two externally
obtained candidates, explicit selection, real validated file and negative cases.
Both Windows/Linux need positive live evidence; fail-closed alone is not success.
End-of-milestone source recheck and real receipts precede declaring T068 DONE.
