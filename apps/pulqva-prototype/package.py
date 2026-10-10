"""Package exact built prototype + already pinned sidecars. Never launches Tor."""
from __future__ import annotations
import argparse, hashlib, json, os, shutil, stat, tempfile, unittest, zipfile
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
VERSION = '0.1.0-alpha.1'

def sha(path: Path) -> str:
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def pin(sums: Path, asset: str) -> str:
    found = [line.split()[0] for line in sums.read_text().splitlines() if len(line.split()) == 2 and line.split()[1] == asset]
    if len(found) != 1 or len(found[0]) != 64:
        raise ValueError('Missing or ambiguous binary pin')
    return found[0]

def require_binary(path: Path, platform: str, expected: str | None = None) -> None:
    if path.is_symlink() or not path.is_file() or path.stat().st_size == 0:
        raise ValueError('Missing or unsafe binary')
    with path.open('rb') as stream:
        magic = stream.read(4)
    if (platform.startswith('windows') and magic[:2] != b'MZ') or (platform.startswith('linux') and magic != b'\x7fELF'):
        raise ValueError('Wrong binary format')
    if expected is not None and sha(path) != expected:
        raise ValueError('Binary pin mismatch')

def package(binary: Path, arti: Path, ytdlp: Path, platform: str, out: Path, source: str) -> Path:
    if len(source) != 40 or any(c not in '0123456789abcdef' for c in source):
        raise ValueError('Invalid source commit')
    win = platform == 'windows-x86_64'
    if platform not in ('windows-x86_64','linux-x86_64'):
        raise ValueError('Unsupported platform')
    artiname = 'arti.exe' if win else 'arti'
    ytname = 'yt-dlp.exe' if win else 'yt-dlp_linux'
    require_binary(binary, platform)
    require_binary(arti, platform, pin(ROOT/'sidecars/arti/SHA256SUMS', platform+'/'+artiname))
    require_binary(ytdlp, platform, pin(ROOT/'sidecars/yt-dlp/SHA256SUMS', ytname))
    out.mkdir(parents=True, exist_ok=True)
    name = f'PULQVA-v{VERSION}-{platform}'
    destination = out/(name+'.zip')
    if destination.exists():
        raise FileExistsError(destination)
    with tempfile.TemporaryDirectory(prefix='pulqva-package-') as temp:
        stage = Path(temp)/name; (stage/'sidecars').mkdir(parents=True)
        paths = [('PULQVA.exe' if win else 'PULQVA', binary),('sidecars/'+artiname,arti),('sidecars/'+ytname,ytdlp)]
        for rel, path in paths:
            shutil.copyfile(path,stage/rel)
            (stage/rel).chmod(0o755)
        shutil.copyfile(ROOT/'apps/pulqva-prototype/README.md',stage/'START_HERE.md')
        for sidecar in ('arti','yt-dlp'):
            for filename in ('VERSION','SHA256SUMS','SOURCE_PROOF.json'):
                path=ROOT/'sidecars'/sidecar/filename
                if path.is_file():
                    target=stage/'provenance'/sidecar/filename; target.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(path,target)
        manifest={'schema':1,'version':VERSION,'kind':'engineering-prototype-candidate','source_commit':source,'platform':platform,
            'user_e2e_verified':False,'privacy_release_acceptance':False,'clean_machine_verified':False,
            'ai_connected':False,'autopilot_connected':False,'ui':'local-browser-loopback','files':{}}
        for path in sorted(stage.rglob('*')):
            if path.is_file(): manifest['files'][path.relative_to(stage).as_posix()]={'sha256':sha(path),'bytes':path.stat().st_size}
        (stage/'prototype-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
        with zipfile.ZipFile(destination,'x',compression=zipfile.ZIP_DEFLATED) as archive:
            for path in sorted(stage.rglob('*')):
                if path.is_file():
                    rel=path.relative_to(Path(temp)).as_posix(); info=zipfile.ZipInfo(rel,date_time=(2026,10,10,0,0,0))
                    mode=0o755 if path.relative_to(stage).as_posix() in [p[0] for p in paths] else 0o644
                    info.create_system=3; info.external_attr=(stat.S_IFREG|mode)<<16; info.compress_type=zipfile.ZIP_DEFLATED
                    archive.writestr(info,path.read_bytes())
    with zipfile.ZipFile(destination) as archive:
        if archive.testzip() is not None: raise ValueError('Archive CRC mismatch')
        saved=json.loads(archive.read(name+'/prototype-manifest.json'))
        for path, expected in saved['files'].items():
            data=archive.read(name+'/'+path)
            if len(data)!=expected['bytes'] or hashlib.sha256(data).hexdigest()!=expected['sha256']: raise ValueError('Archive readback mismatch')
    (out/(destination.name+'.sha256')).write_text(sha(destination)+'  '+destination.name+'\n')
    return destination

class PackageTests(unittest.TestCase):
    def test_pin_rejects_missing_or_duplicate(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/'sums'; p.write_text('a'*64+'  x\n'+'b'*64+'  x\n')
            for name in ('x','missing'):
                with self.assertRaises(ValueError): pin(p,name)
    def test_wrong_format_and_digest_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/'bin'; p.write_bytes(b'\x7fELFfixture')
            require_binary(p,'linux-x86_64',sha(p))
            with self.assertRaises(ValueError): require_binary(p,'windows-x86_64')
            with self.assertRaises(ValueError): require_binary(p,'linux-x86_64','0'*64)
    def test_source_identity_is_required(self):
        with self.assertRaises(ValueError): package(Path('x'),Path('x'),Path('x'),'linux-x86_64',Path('x'),'bad')

if __name__=='__main__':
    import sys
    if '--self-test' in sys.argv:
        unittest.main(argv=[sys.argv[0]],verbosity=2)
    else:
        p=argparse.ArgumentParser(); p.add_argument('--binary',type=Path,required=True);p.add_argument('--arti',type=Path,required=True);p.add_argument('--ytdlp',type=Path,required=True);p.add_argument('--platform',required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--source',required=True)
        a=p.parse_args(); print(package(a.binary,a.arti,a.ytdlp,a.platform,a.out,a.source))
