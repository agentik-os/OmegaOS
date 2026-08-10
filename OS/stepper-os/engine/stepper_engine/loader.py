"""Load + validate a Stepper project from disk.

Layout (created by `stepper init`, mirrored from the pack's recommended tree):

    <project>/
    ├── stepper.yaml            the manifest (pack 11)
    └── stepper/
        ├── modules/*.yaml      one ModuleSpec per file
        ├── epics/*.yaml        one EpicSpec per file
        ├── slices/*.yaml       one SliceSpec per file
        └── steps/*.yaml        one StepSpec per file

Validation: schema (pydantic), id uniqueness, and referential integrity
(step -> module/epic/slice, hard deps -> existing steps). Graph acyclicity
is `graph.StepGraph`'s job - the loader only guarantees well-formed specs.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path

import yaml

from .models import EpicSpec, Manifest, ModuleSpec, SliceSpec, StepSpec

MANIFEST_NAME = "stepper.yaml"
SPEC_ROOT = "stepper"


class LoadError(Exception):
    """A spec failed schema or integrity validation."""


@dataclass
class Project:
    root: Path
    manifest: Manifest
    modules: dict[str, ModuleSpec] = field(default_factory=dict)
    epics: dict[str, EpicSpec] = field(default_factory=dict)
    slices: dict[str, SliceSpec] = field(default_factory=dict)
    steps: dict[str, StepSpec] = field(default_factory=dict)

    @property
    def state_dir(self) -> Path:
        return self.root / ".stepper"


def _read_yaml(path: Path) -> dict:
    try:
        data = yaml.safe_load(path.read_text())
    except yaml.YAMLError as e:
        raise LoadError(f"{path}: invalid YAML: {e}") from e
    if not isinstance(data, dict):
        raise LoadError(f"{path}: expected a mapping at top level")
    return data


def _load_dir(root: Path, sub: str, model, id_field: str) -> dict:
    out: dict = {}
    directory = root / SPEC_ROOT / sub
    if not directory.is_dir():
        return out
    for path in sorted(directory.glob("*.yaml")) + sorted(directory.glob("*.yml")):
        data = _read_yaml(path)
        try:
            spec = model(**data)
        except Exception as e:  # pydantic ValidationError, kept broad for one message
            raise LoadError(f"{path}: {e}") from e
        spec_id = getattr(spec, id_field)
        if spec_id in out:
            raise LoadError(f"{path}: duplicate {id_field} '{spec_id}'")
        out[spec_id] = spec
    return out


def load_project(root: Path) -> Project:
    root = root.resolve()
    manifest_path = root / MANIFEST_NAME
    if not manifest_path.is_file():
        raise LoadError(
            f"{manifest_path} not found - not a Stepper project (run `stepper init`)"
        )
    manifest = Manifest(**_read_yaml(manifest_path))

    project = Project(
        root=root,
        manifest=manifest,
        modules=_load_dir(root, "modules", ModuleSpec, "module_id"),
        epics=_load_dir(root, "epics", EpicSpec, "epic_id"),
        slices=_load_dir(root, "slices", SliceSpec, "slice_id"),
        steps=_load_dir(root, "steps", StepSpec, "step_id"),
    )
    _check_integrity(project)
    return project


def _check_integrity(project: Project) -> None:
    problems: list[str] = []
    for step in project.steps.values():
        if project.modules and step.module and step.module not in project.modules:
            problems.append(f"{step.step_id}: unknown module '{step.module}'")
        if project.epics and step.epic and step.epic not in project.epics:
            problems.append(f"{step.step_id}: unknown epic '{step.epic}'")
        if project.slices and step.slice and step.slice not in project.slices:
            problems.append(f"{step.step_id}: unknown slice '{step.slice}'")
        for dep in step.dependencies.hard + step.dependencies.soft:
            if dep not in project.steps:
                problems.append(f"{step.step_id}: unknown dependency '{dep}'")
    for epic in project.epics.values():
        if project.modules and epic.module_id and epic.module_id not in project.modules:
            problems.append(f"{epic.epic_id}: unknown module '{epic.module_id}'")
    if problems:
        raise LoadError("integrity errors:\n  " + "\n  ".join(problems))
