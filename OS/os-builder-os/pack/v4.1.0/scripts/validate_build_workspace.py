#!/usr/bin/env python3
from __future__ import annotations
import argparse, json
from pathlib import Path
import re

FORBIDDEN_SECRET_PATTERNS=[
    ("OPENAI_API_KEY", re.compile(r"OPENAI_API_KEY", re.I)),
    ("ANTHROPIC_API_KEY", re.compile(r"ANTHROPIC_API_KEY", re.I)),
    ("generic API key assignment", re.compile(r"(?:api[_ -]?key|token|secret)\s*[:=]\s*[\"\'][A-Za-z0-9_\-]{12,}", re.I)),
    ("OpenAI-style secret", re.compile(r"sk-[A-Za-z0-9_-]{12,}")),
]

def scan_for_secrets(ws:Path):
    findings=[]
    text_ext={'.md','.txt','.yaml','.yml','.json','.jsonl','.csv','.py','.ts','.tsx','.js','.jsx','.toml','.ini','.env'}
    for p in ws.rglob('*'):
        if not p.is_file() or p.suffix.lower() not in text_ext: continue
        try: text=p.read_text(errors='ignore')
        except Exception: continue
        for label,rx in FORBIDDEN_SECRET_PATTERNS:
            if rx.search(text): findings.append(f"forbidden credential material: {p.relative_to(ws)} ({label})")
    return findings

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('workspace'); args=ap.parse_args()
    ws=Path(args.workspace).resolve(); root=Path(__file__).resolve().parents[1]
    fmap=json.loads((root/'templates/build_workspace_file_map.json').read_text())
    missing=[]
    for phase,files in fmap.items():
        for f in files:
            p=ws/phase/f
            if not p.exists(): missing.append(str(p.relative_to(ws)))
    required_dirs=['15_interfaces/WORKFLOWS','16_runtime_design/AGENTS','16_runtime_design/SKILLS','16_runtime_design/PROMPTS','17_handoffs/HANDOFF_CONTRACTS','18_implementation/runtime','18_implementation/commands','18_implementation/workflows','18_implementation/agents','18_implementation/skills','18_implementation/prompts','18_implementation/tools','18_implementation/memory','18_implementation/schemas','18_implementation/registry','18_implementation/tests','20_evals/SCENARIOS','24_release/dist']
    for d in required_dirs:
        if not (ws/d).is_dir(): missing.append(d+'/')
    state=ws/'00_control/BUILD_STATE.json'
    try:
        data=json.loads(state.read_text())
        if len(data.get('phases',[]))!=26: missing.append('BUILD_STATE must contain 26 phases')
    except Exception as e: missing.append('invalid BUILD_STATE.json: '+str(e))
    missing.extend(scan_for_secrets(ws))
    if missing:
        print('FAIL'); [print(' -',x) for x in missing]; raise SystemExit(1)
    print('PASS: workspace skeleton complete')
if __name__=='__main__': main()
