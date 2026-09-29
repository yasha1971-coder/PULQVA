# Local test certificates only

These public test credentials are generated for the in-process SOCKS/TLS fixtures.
They are NOT Wikimedia certificates and MUST NOT be trusted in production.
Only cfg(test) code includes them. No host certificate store is modified.

ca.der is an ephemeral P-256 test CA. server.der is its test server certificate
for commons.wikimedia.org; wrong.der has the deliberately wrong wrong.invalid SAN.
server-key.der is their deliberately public PKCS#8 test key, not a user secret.
Certificates are valid 2025-01-01 through 2035-01-01. After that date regenerate
this test set; never disable date/name/signature verification to keep tests green.
Generated locally with Python cryptography; the shipping code has no Python need.
