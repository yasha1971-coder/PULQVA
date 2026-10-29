"""Frozen tracked-byte verification and exact Tauri generated-output allowlist."""
import argparse, hashlib, json, pathlib, subprocess, sys
PREFIX = "apps/pulqva-desktop/src-tauri/gen/schemas/"
def git_names(root, others=False):
    args = ["git", "-c", "safe.directory="+str(root), "-C", str(root), "ls-files"]
    if others: args += ["--others", "--exclude-standard"]
    data = subprocess.check_output(args+["-z"])
    return data.decode("utf-8").rstrip("\0").split("\0") if data else []
def checked(root, name):
    rel = pathlib.PurePosixPath(name)
    if rel.is_absolute() or ".." in rel.parts: raise ValueError("unsafe input path")
    path = root
    for part in rel.parts:
        path = path / part
        if path.is_symlink(): raise ValueError("symlink input/output")
    if not path.is_file(): raise ValueError("missing input/output")
    return path
def snapshot(root, names):
    return {n: hashlib.sha256(checked(root,n).read_bytes()).hexdigest() for n in names}
def verify(root, before, names, untracked, platform):
    if snapshot(root,names) != before: raise ValueError("tracked source bytes changed")
    allowed = {PREFIX+n for n in ("acl-manifests.json","capabilities.json","desktop-schema.json",platform+"-schema.json")}
    if set(untracked)-allowed: raise ValueError("unexpected untracked build output: "+repr(sorted(set(untracked)-allowed)))
    generated = {}
    for name in untracked:
        path = checked(root,name)
        if path.stat().st_size > 2*1024*1024: raise ValueError("oversized schema")
        data = path.read_bytes()
        if not isinstance(json.loads(data), dict): raise ValueError("schema must be JSON object")
        generated[name] = dict(bytes=len(data),sha256=hashlib.sha256(data).hexdigest())
    return dict(tracked_inputs_unchanged=True,generated=generated)
def main():
    p=argparse.ArgumentParser();p.add_argument("mode",choices=["snapshot","verify"]);p.add_argument("root");p.add_argument("baseline");p.add_argument("platform",nargs="?");a=p.parse_args()
    root=pathlib.Path(a.root).resolve();baseline=pathlib.Path(a.baseline)
    if a.mode=="snapshot":
        if git_names(root,True): raise ValueError("untracked inputs before build")
        with baseline.open("x",encoding="utf-8") as f: json.dump(snapshot(root,git_names(root)),f,sort_keys=True)
    else:
        if a.platform not in ("linux","windows"): raise ValueError("unsupported platform")
        print(json.dumps(verify(root,json.loads(baseline.read_text()),git_names(root),git_names(root,True),a.platform),sort_keys=True))
if __name__=="__main__": main()
