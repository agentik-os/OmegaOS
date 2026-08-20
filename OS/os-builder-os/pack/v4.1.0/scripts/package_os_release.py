#!/usr/bin/env python3
from __future__ import annotations
import argparse, hashlib, json, shutil
from pathlib import Path

def sha256(p):
    h=hashlib.sha256()
    with p.open('rb') as f:
        for chunk in iter(lambda:f.read(1024*1024),b''): h.update(chunk)
    return h.hexdigest()

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('workspace'); ap.add_argument('--version',default='1.0.0'); args=ap.parse_args()
    ws=Path(args.workspace).resolve(); state=json.loads((ws/'00_control/BUILD_STATE.json').read_text())
    slug=state['os_slug']; dist=ws/'24_release/dist'; dist.mkdir(parents=True,exist_ok=True)
    base=dist/f'{slug}-v{args.version}'
    archive=Path(shutil.make_archive(str(base),'zip',root_dir=ws))
    checksum=sha256(archive)
    (ws/'24_release/CHECKSUMS.txt').write_text(f'{checksum}  {archive.name}\n')
    manifest={'os_slug':slug,'version':args.version,'archive':archive.name,'sha256':checksum,'builder_version':'4.1.0'}
    (ws/'24_release/RELEASE_MANIFEST.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(archive)
if __name__=='__main__': main()
