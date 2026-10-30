"""Only inherited GUI transport context plus trusted per-run overrides."""
import os
INHERITED_KEYS=frozenset({"PATH","LANG","LC_ALL","DISPLAY","WAYLAND_DISPLAY",
    "XDG_RUNTIME_DIR","DBUS_SESSION_BUS_ADDRESS","XAUTHORITY"})
def capture_environment(manifest, inherited=None):
    source=os.environ if inherited is None else inherited
    env={k:v for k,v in source.items() if k in INHERITED_KEYS}
    env.update(manifest.get("isolated_environment",{}))
    env["PULQVA_TOR_DIAGNOSTICS"]="1"
    return env
