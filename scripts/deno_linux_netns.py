"""CI-only Linux network namespace gate. Requires sudo on the CI runner."""
import errno
import json
import os
from pathlib import Path
import socket
import subprocess
import sys

from deno_restricted_probe import run_restricted


def main():
    executable, directory, parent_namespace, uid, gid = sys.argv[1:]
    namespace = os.readlink("/proc/self/ns/net")
    if namespace == parent_namespace or os.geteuid() != 0:
        raise RuntimeError("expected fresh privileged network namespace")
    subprocess.run(["/usr/sbin/ip", "link", "set", "lo", "up"], check=True, timeout=5)
    interfaces = sorted(name for _, name in socket.if_nameindex())
    if interfaces != ["lo"]:
        raise RuntimeError("namespace has external interfaces")
    # Drop privilege before running the fixture harness or Deno.
    os.setgroups([])
    os.setgid(int(gid))
    os.setuid(int(uid))
    if os.geteuid() == 0:
        raise RuntimeError("refusing privileged runtime fixture")
    # UDP connect performs route selection; no datagram is sent.
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as probe:
        try:
            probe.connect(("192.0.2.1", 9))
        except OSError as error:
            if error.errno != errno.ENETUNREACH:
                raise
        else:
            raise RuntimeError("unexpected external IPv4 route")
    result = run_restricted(Path(executable), Path(directory))
    result["network_gate"] = {
        "kind": "fresh_linux_network_namespace",
        "different_from_parent": True, "interfaces": interfaces,
        "external_ipv4_route": "ENETUNREACH",
        "runtime_uid": os.geteuid(),
        "scope": "CI fixture external IP transport blocked; not an application sandbox",
    }
    print(json.dumps(result))


if __name__ == "__main__":
    main()
