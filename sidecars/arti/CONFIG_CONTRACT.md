# Arti 2.6.0 configuration contract evidence

Verified against upstream Arti documentation/source on 2026-09-20.

PULQVA uses these exact configuration keys:

- `application.defer_bootstrap`
- `application.watch_configuration`
- `proxy.socks_listen`
- `proxy.dns_listen`
- `storage.cache_dir`
- `storage.state_dir`

For `proxy.socks_listen`, a numeric port maps to Arti's localhost listener.
PULQVA therefore uses `19050`, not a wildcard address.

The fixture is parsed and resolved through Arti 2.6.0's own configuration types in CI on
Windows and Linux. The validator does not call `arti::run_proxy`, does not spawn the
`arti` binary, and does not bootstrap Tor.
