#!/usr/bin/env python3
from __future__ import annotations
import argparse, json, re, shutil
from datetime import datetime, timezone
from pathlib import Path

def slugify(v:str)->str:
    v=re.sub(r'[^a-z0-9]+','-',v.strip().lower()).strip('-')
    return v or 'unnamed'

def default_content(path:Path, name:str, slug:str):
    suffix=path.suffix.lower()
    if path.name=='BUILD_REQUEST.md': return f'# Build request\n\n/os-build {name} {{OS}}\n'
    if path.name=='BUILD_CONTRACT.yaml': return f'os_name: "{name} {{OS}}"\nos_slug: "{slug}"\nmode: ultimate\nstatus: draft\nassumptions: []\noutcomes: []\nexclusions: []\nrelease_thresholds:\n  critical_eval_min: 0.90\n'
    if path.name=='OS_MANIFEST.yaml': return f'name: "{name} {{OS}}"\nslug: "{slug}"\nversion: 0.1.0\nstatus: build\ncapabilities: []\ncommands: []\nworkflows: []\n'
    if suffix in ['.yaml','.yml']: return 'status: template\nitems: []\n'
    if suffix=='.json': return '{}\n'
    if suffix=='.jsonl': return ''
    if suffix=='.csv': return 'id,status,notes\n'
    if suffix=='.md': return f'# {path.stem.replace("_"," ").title()}\n\nStatus: template\n'
    if suffix=='.txt': return ''
    return ''

def main():
    ap=argparse.ArgumentParser(description='Create a complete Builder {OS} v4 workspace')
    ap.add_argument('name')
    ap.add_argument('--output',default='builds')
    ap.add_argument('--force',action='store_true')
    args=ap.parse_args()
    root=Path(__file__).resolve().parents[1]
    slug=slugify(args.name)+'-os'
    target=Path(args.output).resolve()/slug
    if target.exists():
        if not args.force: raise SystemExit(f'Workspace already exists: {target}')
        shutil.rmtree(target)
    target.mkdir(parents=True)
    fmap=json.loads((root/'templates/build_workspace_file_map.json').read_text())
    artifact_entries=[]
    for phase, files in fmap.items():
        d=target/phase; d.mkdir(parents=True,exist_ok=True)
        for f in files:
            p=d/f; p.write_text(default_content(p,args.name,slug),encoding='utf-8')
            artifact_entries.append({'artifact_id':f'{phase}.{p.stem.lower()}','phase':phase.split('_')[0],'path':str(p.relative_to(target)),'status':'template'})
    extras=['15_interfaces/WORKFLOWS','16_runtime_design/AGENTS','16_runtime_design/SKILLS','16_runtime_design/PROMPTS','17_handoffs/HANDOFF_CONTRACTS','18_implementation/runtime','18_implementation/commands','18_implementation/workflows','18_implementation/agents','18_implementation/skills','18_implementation/prompts','18_implementation/tools','18_implementation/memory','18_implementation/schemas','18_implementation/registry','18_implementation/tests','20_evals/SCENARIOS','24_release/dist']
    for d in extras: (target/d).mkdir(parents=True,exist_ok=True)
    now=datetime.now(timezone.utc).isoformat()
    phases=[{'id':f'{i:02d}','status':'pending'} for i in range(26)]
    phases[0]['status']='active'
    state={'build_id':f'{slug}-{datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")}', 'os_name':f'{args.name} {{OS}}','os_slug':slug,'builder_version':'4.1.0','status':'initialized','current_phase':'00','phases':phases,'failed_gates':[],'open_findings':[],'created_at':now,'updated_at':now}
    (target/'00_control/BUILD_STATE.json').write_text(json.dumps(state,indent=2)+'\n',encoding='utf-8')
    (target/'00_control/ARTIFACT_INDEX.json').write_text(json.dumps({'artifacts':artifact_entries},indent=2)+'\n',encoding='utf-8')
    (target/'00_control/BUILD_EVENT_LOG.jsonl').write_text(json.dumps({'ts':now,'event':'build_initialized','build_id':state['build_id']})+'\n',encoding='utf-8')
    (target/'README.md').write_text(f'# {args.name} {{OS}} build workspace\n\nBuilder {{OS}} v4.1.0. Follow `00_control/BUILD_STATE.json` and the canonical roadmap.\n',encoding='utf-8')
    print(target)
if __name__=='__main__': main()
