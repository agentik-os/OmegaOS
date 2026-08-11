#!/usr/bin/env python3
"""Score wav/ogg files with UTMOS22Strong (predicted MOS, 1-5, higher = better).

Usage: judge_mos.py <file> [file ...]   → one "score<TAB>path" line per file.
Needs the omnivoice venv (torch + torchaudio) and the weights at
~/.omega/tts/models/eval/mos/utmos22_strong_step7459_v1.pt
"""
import os
import sys

import torch
import torchaudio

sys.path.insert(0, os.path.expanduser("~/Station/Tools/OmniVoice"))
from omnivoice.eval.models.utmos import UTMOS22Strong  # noqa: E402

WEIGHTS = os.path.expanduser("~/.omega/tts/models/eval/mos/utmos22_strong_step7459_v1.pt")

model = UTMOS22Strong()
model.load_state_dict(torch.load(WEIGHTS, map_location="cpu"))
model.eval()

for path in sys.argv[1:]:
    try:
        wave, sr = torchaudio.load(path)
        if sr != 16000:
            wave = torchaudio.functional.resample(wave, sr, 16000)
            sr = 16000
        wave = wave.mean(0, keepdim=True)
        with torch.inference_mode():
            score = model(wave, sr)
        print(f"{float(score.mean()):.3f}\t{path}")
    except Exception as e:  # noqa: BLE001
        print(f"ERR\t{path}\t{e}")
