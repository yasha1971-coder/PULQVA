# PULQVA threat model — development draft

This describes intended boundaries and current limitations, not a completed audit.

## Assets and trust boundaries

Protect query contents, source URLs, network identity, local files and integrity of
executables. The frontend sends typed requests to the trusted Rust backend.
Provider/AI output is untrusted data and must not become shell commands.
Arti supplies the approved external transport; packaged sidecars are separately
verified components. The operating system and machine are assumed uncompromised.

## Observers and limits

| Observer or hazard | Intended control | Remaining exposure |
| --- | --- | --- |
| Local network / ISP | External application requests through Tor, including remote DNS | Tor use, traffic timing and volume may remain visible |
| Search, AI or source provider | No required account/key; Tor-routed requests | Provider still sees submitted text and requests; content can identify the user |
| Tor exit | End-to-end HTTPS where applicable | Plain HTTP has no equivalent content protection; Tor alone is not encryption to the destination |
| Malicious provider output | Typed parsing and validation; no generated shell text | Incorrect results and hostile content remain possible |
| Malicious or compromised download | Verify packaged tool identities; validate output paths | Downloaded user files are not automatically safe; opening them may access the network outside PULQVA |
| Local attacker or compromised OS | Minimize persistent metadata and restrict owned paths | No protection against an attacker controlling the machine |
| Transport failure | Stop external operations; no intentional direct fallback | Availability may be lost; a green component test is not complete egress verification |

## Release evidence still required

- Packaged-app traffic and DNS checks on both supported OSes, including child processes.
- Transport-loss tests that prove no direct fallback across the complete journey.
- Dependency/source/license inventory for the exact redistributed binaries.
- Review of logs, retention, updates and executable publication.
- Independent review before claims of suitability for high-risk users.

Existing CI covers individual boundaries and selected live probes. It does not prove
resistance to global traffic correlation, all filesystem races, censorship in every
network, or safe opening of arbitrary files.
