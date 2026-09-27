import hashlib,json,os,shutil,sys,tempfile
from pathlib import Path
ROOT=Path.cwd()
PINS={"reqwest":"0.13.5","rustls":"0.23.43","webpki-roots":"1.0.9","tokio":"1.53.1"}
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def prepare():
 d=Path(tempfile.mkdtemp(prefix="pulqva-t068-prod-")); shutil.copytree(ROOT,d,dirs_exist_ok=True)
 m=d/"crates/pulqva-discovery/Cargo.toml"
 with m.open("a",encoding="utf-8") as f:
  f.write('\nreqwest = { version = "=0.13.5", default-features = false, features = ["rustls-no-provider", "socks"] }\nrustls = { version = "=0.23.43", default-features = false, features = ["ring", "std", "tls12"] }\nwebpki-roots = "=1.0.9"\ntokio = { version = "=1.53.1", default-features = false, features = ["rt", "net", "time"] }\n')
 (d/"crates/pulqva-discovery/examples").mkdir(parents=True,exist_ok=True)
 shutil.copy2(ROOT/"tools/t068_https_executor.rs",d/"crates/pulqva-discovery/examples/t068_https_executor.rs")
 with (Path(os.environ["GITHUB_ENV"])).open("a") as f: f.write(f"T068_PROD={d.as_posix()}\n")
def verify():
 d=Path(os.environ["T068_PROD"]); lock=d/"Cargo.lock"; manifest=d/"crates/pulqva-discovery/Cargo.toml"
 text=lock.read_text()
 for n,v in PINS.items():
  if f'name = "{n}"' not in text or f'version = "{v}"' not in text: raise SystemExit(f"missing pin {n} {v}")
 rec={"lock_sha256":sha(lock),"lock_bytes":lock.stat().st_size,"manifest_sha256":sha(manifest),"pins":PINS,"runner_os":os.environ.get("RUNNER_OS"),"head":os.environ.get("GITHUB_SHA"),"scope":"production graph candidate; no live Tor/E2E"}
 (d/"production-receipt.json").write_text(json.dumps(rec,sort_keys=True,indent=2)+"\n")
 print("PULQVA_T068_PROD",json.dumps(rec,sort_keys=True))
if __name__=="__main__":
 {"prepare":prepare,"verify":verify}[sys.argv[1]]()
