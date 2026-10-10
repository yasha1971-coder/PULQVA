# T069-PROTOTYPE-V01 — alpha candidate vertical slice

Base: native PR92 54aa65874a5cc8a47777b8c1c15a22b53d42b98c.
Owner requested creation of a v0.1 prototype, not a redefinition of Kernel.

One independent executable reuses existing core/discovery/privacy crates and unchanged workspace lock. Its local-browser UI is an explicit alpha delivery adapter, not replacement of the primary Tauri app. It supports live Commons WebM search, nonce-bound manual choice and saved byte verification. No AI, Autopilot, broader sources, or full release acceptance is claimed.

GO only for offline tests/build/package candidate. No automatic public release/tag or live probe. The regular Kernel, v1 recovery guard, sealed INDEX/State/NEXT, historical evidence, all existing sidecar pins and production Tauri UI remain unchanged.

Before any user/private release, observe exact CI source and artifact bytes; validate actual request -> choice -> file on clean Windows and Linux; independently test DNS/egress/fail-closed and process tree termination; settle browser extension/profile boundary, notices and package requirements. Hash consistency is not publisher authenticity or proof of private execution.

Method references (reviewed 2026-10-10): official Tauri Windows runtime/command documentation, rustls CryptoProvider/SecureRandom and Rust std::fs TOCTOU documentation. The local browser was chosen to avoid claiming an unbundled WebView2 is a self-contained Tauri release. This trades installer complexity for an explicitly unaccepted ordinary-browser privacy boundary; do not hide that limitation.

ONE NEXT ACTION: observe the first prototype-alpha-build generation and preserve its first failure or exact original candidate/evidence. No blind rebuild and no stable v0.1 tag based on offline tests alone.
