"""CI fixtures for pinned yt-dlp-style Deno invocation; not an OS sandbox proof."""
import hashlib
import json
from pathlib import Path
import socket
import subprocess
import threading

from verify_deno_artifact import clean_environment

FLAGS = ["run", "--ext=js", "--no-code-cache", "--no-prompt", "--no-remote",
         "--no-lock", "--node-modules-dir=none", "--no-config", "--no-npm",
         "--cached-only", "-"]


def validate_result(completed, expected):
    if completed.returncode != 0 or completed.stderr.strip():
        raise ValueError("fixture process failed: " + completed.stderr[:2000])
    if json.loads(completed.stdout) != expected:
        raise ValueError("fixture did not produce the exact expected result")


def run_restricted(executable, directory):
    executable = executable.resolve()
    env = clean_environment(directory)
    fixture_file = directory / "read-fixture.txt"
    fixture_file.write_text("public fixture", encoding="utf-8")
    write_file = directory / "write-fixture.txt"
    command = [str(executable), *FLAGS]
    results = []
    connections = []
    stop = threading.Event()
    # A real owned listener distinguishes blocked permission from connection failure.
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.bind(("127.0.0.1", 0))
        listener.listen()
        listener.settimeout(0.1)
        port = listener.getsockname()[1]

        def monitor():
            while not stop.is_set():
                try:
                    conn, _ = listener.accept()
                    connections.append(True)
                    conn.close()
                except socket.timeout:
                    continue

        thread = threading.Thread(target=monitor)
        thread.start()
        try:
            fixtures = {
                "compute": 'console.log(JSON.stringify({value: [1,2,3].reduce((a,b)=>a+b,0)}));',
                "net": f'await fetch("http://127.0.0.1:{port}/denied");',
                "read": f'await Deno.readTextFile({json.dumps(str(fixture_file))});',
                "write": f'await Deno.writeTextFile({json.dumps(str(write_file))}, "denied");',
                "env": 'Deno.env.get("PULQVA_DENIED_FIXTURE");',
                "run": f'await new Deno.Command({json.dumps(str(executable))}, {{args:["--version"]}}).output();',
                "remote_import": f'await import("http://127.0.0.1:{port}/module.js");',
            }
            for name, operation in fixtures.items():
                if name == "compute":
                    source = operation
                    expected = {"value": 6}
                elif name == "remote_import":
                    # Require --no-remote's diagnostic, not a generic network error.
                    source = ('try {' + operation +
                              '; throw new Error("unexpected success"); } catch(e) {'
                              'if (!/remote/i.test(e.message) || !/no-remote/i.test(e.message)) throw e;'
                              'console.log(JSON.stringify({blocked:"remote_import"})); }')
                    expected = {"blocked": name}
                else:
                    source = ('try {' + operation +
                              '; throw new Error("unexpected success"); } catch(e) {'
                              'if (e.name !== "NotCapable") throw e;'
                              'console.log(JSON.stringify({denied:' + json.dumps(name) + '})); }')
                    expected = {"denied": name}
                completed = subprocess.run(command, input=source, cwd=directory, env=env,
                                           capture_output=True, text=True, timeout=10)
                validate_result(completed, expected)
                results.append({"fixture": name, "result": expected,
                                "source_sha256": hashlib.sha256(source.encode()).hexdigest()})
        finally:
            stop.set()
            thread.join(timeout=2)
        if connections or thread.is_alive():
            raise ValueError("unexpected loopback connection or monitor did not stop")
    if write_file.exists() or fixture_file.read_text(encoding="utf-8") != "public fixture":
        raise ValueError("unexpected fixture filesystem mutation")
    return {"flags": FLAGS, "fixtures": results, "loopback_connections": len(connections),
            "scope": "fixture permission denials only; external egress monitoring and OS isolation NOT established"}
