#!/usr/bin/env python3
"""Validate the Builder {OS} package structure with standard-library Python."""

from __future__ import annotations

import json
import sys
from pathlib import Path

REQUIRED = [
    "README.md",
    "AGENTS.md",
    "MASTER_PROMPT.md",
    "OS_DEFINITION.md",
    "HOW_TO_USE.md",
    "commands/COMMAND_REFERENCE.md",
    "docs/CANONICAL_PIPELINE.md",
    "docs/RESEARCH_AND_CORPUS_PROTOCOL.md",
    "docs/KNOWLEDGE_SYNTHESIS_PROTOCOL.md",
    "docs/OS_ARCHITECTURE_STANDARD.md",
    "docs/QUALITY_GATES.md",
    "workflows/os-build-ultimate.yaml",
    "schemas/source.schema.json",
    "schemas/claim.schema.json",
    "schemas/book-analysis.schema.json",
    "schemas/build-state.schema.json",
    "schemas/os-manifest.schema.json",
    "evals/ULTIMATE_OS_RUBRIC.yaml",
    "registry/command-registry.yaml",
    "codex/PROMPT_TO_INSTALL.md",
]


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    errors: list[str] = []

    for rel in REQUIRED:
        path = root / rel
        if not path.exists():
            errors.append(f"Missing required file: {rel}")
        elif path.stat().st_size == 0:
            errors.append(f"Empty required file: {rel}")

    for path in root.rglob("*.json"):
        try:
            json.loads(path.read_text(encoding="utf-8"))
        except Exception as exc:
            errors.append(f"Invalid JSON {path.relative_to(root)}: {exc}")

    forbidden = "\u2014"
    for path in root.rglob("*"):
        if path.is_file() and path.suffix.lower() in {".md", ".yaml", ".yml", ".json", ".jsonl", ".csv", ".py"}:
            try:
                text = path.read_text(encoding="utf-8")
            except UnicodeDecodeError:
                continue
            if forbidden in text:
                errors.append(f"Forbidden em dash found in {path.relative_to(root)}")

    if errors:
        print("Package validation failed:")
        for error in errors:
            print(f"- {error}")
        return 1

    print("Builder {OS} package validation passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())


# PUBLIC_NO_SECRETS: package-level secret scanning is enforced by validate_build_workspace.py for generated OS workspaces.
