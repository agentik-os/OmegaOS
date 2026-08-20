#!/usr/bin/env python3
"""Package an OS directory into a REPRODUCIBLE release ZIP.

The release gate has an item that reads "reproducible ZIP". A naive
`zipfile.write()` does not satisfy it: it stamps each entry with the file's
mtime and copies its permission bits, so zipping the same unchanged directory
twice produces two archives with different bytes and different hashes. That
turns the gate item into a claim nobody can check, which is worse than not
having the item at all.

This tool makes the claim checkable:

  - entries are emitted in sorted order, so the archive layout is stable
  - every entry carries a fixed timestamp, so mtime cannot leak in
  - every entry carries fixed 0644 permissions, so umask cannot leak in
  - build noise is excluded by name and by suffix
  - the SHA-256 of the finished archive is printed

Zip the same content twice and the two hashes are identical. If they are not,
something in the directory changed, and that is exactly what a release gate is
supposed to notice.

Usage:
    create_zip.py <path/to/os-dir>                 -> <os-dir>.zip beside it
    create_zip.py <path/to/os-dir> <out.zip>       -> explicit output path
    create_zip.py <path/to/os-dir> --list          -> what would be included
    create_zip.py <path/to/os-dir> --json          -> machine-readable result

Exit codes: 0 written, 2 usage or resolution error.
"""
import hashlib
import json
import sys
import zipfile
from pathlib import Path

# Directory names that never belong in a release archive.
EXCLUDE_DIRS = {
    "__pycache__", ".git", ".hg", ".svn", ".pytest_cache", ".mypy_cache",
    ".ruff_cache", "node_modules", ".venv", "venv", ".idea", ".vscode",
    "outputs",  # outputs belong to whoever ran the OS, never to the package
}
EXCLUDE_NAMES = {".DS_Store", "Thumbs.db", ".gitkeep"}
EXCLUDE_SUFFIXES = {".pyc", ".pyo", ".swp", ".swo", ".orig", ".rej"}

# The zip epoch. Any fixed value works; this is the earliest one the format
# accepts, which makes it obvious that it is deliberate and not a real date.
FIXED_TIME = (1980, 1, 1, 0, 0, 0)
FIXED_MODE = 0o644 << 16


def included(root):
    """Every file that belongs in the archive, in a stable order."""
    keep = []
    for path in sorted(root.rglob("*"), key=lambda p: p.as_posix()):
        if not path.is_file() or path.is_symlink():
            continue
        rel = path.relative_to(root)
        if any(part in EXCLUDE_DIRS for part in rel.parts):
            continue
        if path.name in EXCLUDE_NAMES or path.suffix in EXCLUDE_SUFFIXES:
            continue
        keep.append(path)
    return keep


def build(root, out):
    """Write the archive. Returns (file_count, sha256)."""
    files = included(root)          # materialised BEFORE the archive exists,
    prefix = root.name              # so an output inside root cannot zip itself
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as archive:
        for path in files:
            arc = f"{prefix}/{path.relative_to(root).as_posix()}"
            info = zipfile.ZipInfo(arc, date_time=FIXED_TIME)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = FIXED_MODE
            archive.writestr(info, path.read_bytes())
    return len(files), hashlib.sha256(out.read_bytes()).hexdigest()


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    as_json = "--json" in sys.argv
    listing = "--list" in sys.argv

    if not args:
        print(__doc__.strip().split("Usage:")[1].strip(), file=sys.stderr)
        return 2

    root = Path(args[0]).resolve()
    if not root.is_dir():
        print(f"not a directory: {root}", file=sys.stderr)
        return 2

    if listing:
        for path in included(root):
            print(path.relative_to(root).as_posix())
        return 0

    out = Path(args[1]).resolve() if len(args) > 1 else root.with_suffix(".zip")
    out.parent.mkdir(parents=True, exist_ok=True)
    count, digest = build(root, out)

    if as_json:
        print(json.dumps({
            "source": str(root),
            "archive": str(out),
            "files": count,
            "sha256": digest,
            "reproducible": True,
        }, indent=2))
        return 0

    print(f"archive : {out}")
    print(f"files   : {count}")
    print(f"sha256  : {digest}")
    print()
    print("Reproducible: re-running over unchanged content yields this same hash.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
