# Arti 2.6.0 configuration contract evidence

Verified against the published Arti 2.6.0 documentation/source on 2026-09-20.

Upstream evidence:

- Arti 2.6.0 example configuration:
  https://docs.rs/arti/2.6.0/arti/cfg/constant.ARTI_EXAMPLE_CONFIG.html
- Arti 2.6.0 crate documentation:
  https://docs.rs/arti/2.6.0/arti/
- Arti client storage configuration source:
  https://docs.rs/arti-client/0.46.0/src/arti_client/config.rs.html

PULQVA uses these exact configuration keys:

- `application.defer_bootstrap`
- `application.watch_configuration`
- `proxy.socks_listen`
- `proxy.dns_listen`
- `storage.cache_dir`
- `storage.state_dir`

The Arti 2.6.0 example config documents that a numeric `proxy.socks_listen` value listens on
loopback addresses by default. PULQVA uses `19050`, so it does not request a wildcard listener.
`proxy.dns_listen = 0` disables the DNS listener.

The storage source confirms `cache_dir` and `state_dir` are the client storage fields.

The fixture is parsed and resolved through Arti 2.6.0's own configuration types in CI on Windows
and Linux. The validator does not call `arti::run_proxy`, does not spawn the `arti` binary, and
does not bootstrap Tor.
