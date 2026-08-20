#!/usr/bin/env python3
"""Generate crates/omega-core/src/os_products.rs from OS/_registry.json.

The OS tab of `omega menu` renders `OsProduct::all()`. That roster is now
derived from the suite registry rather than hand-maintained, so adding an OS
means editing `suite.py` and re-running, never editing Rust by hand.

Authored `commands:` arrays are PRESERVED: they are parsed out of the current
file, keyed by slug, remapped across the renames and splits, and re-emitted
verbatim. Nothing hand-written is lost.

Only the header (module doc + OsGroup + OsProduct + `all()`) is regenerated.
Everything from OsReadinessLevel onward is spliced through untouched.

Usage:
    python3 gen_os_products.py --check     print what would change
    python3 gen_os_products.py --write     rewrite the file
"""
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
OS_DIR = os.path.dirname(HERE)
REPO = os.path.dirname(OS_DIR)
RS = os.path.join(REPO, "crates", "omega-core", "src", "os_products.rs")
REG = json.load(open(os.path.join(OS_DIR, "_registry.json"), encoding="utf-8"))

# old slug in os_products.rs  ->  new canonical slug that inherits its commands
RENAME = {
    "books-os": "librarian-os",
    "seductive-os": "social-intelligence-os",
    "relationship-network-os": "network-os",
    "delivery-customer-success-os": "delivery-cs-os",
    "quality-evaluation-release-os": "quality-evaluation-os",
    "operations-automation-os": "operations-os",
    "wealth-capital-os": "wealth-os",
}

VARIANT = {g["key"]: g["rust_variant"] for g in REG["groups"]}
GROUPS = {g["key"]: g for g in REG["groups"]}


def parse_commands(src):
    """slug -> the raw text inside its `commands: &[ ... ]`, verbatim."""
    start = src.index("pub fn all() -> &'static [OsProduct] {")
    body = src[start:]
    out = {}
    # Split into per-entry blocks FIRST. Matching `slug ... commands` with a
    # lazy `.*?` across the whole array is wrong: an entry whose array is the
    # empty `commands: &[],` has no `\n  ],` terminator, so the match runs past
    # the entry boundary and steals the NEXT unit's array, silently shifting
    # every mapping after it.
    blocks = body.split("\n            OsProduct {")[1:]
    for block in blocks:
        m_slug = re.search(r'^\s*(?:num: \d+,\s*)?slug: "([^"]+)"', block, re.M)
        if not m_slug:
            continue
        # Only a multi-line array carries authored content; `&[]` carries none.
        m_cmd = re.search(r"commands: &\[(\n.*?)\n\s*\],", block, re.S)
        if m_cmd:
            out[m_slug.group(1)] = m_cmd.group(1)
    if not out:
        raise SystemExit(
            "REFUSING to generate: parsed 0 command arrays from the current "
            "os_products.rs. That would wipe every authored command block. "
            "Fix the parser before re-running."
        )
    return out


TAIL_ANCHOR = "/// Coarse stage derived from concrete local surfaces."


def splice_tail(src):
    """Everything from OsReadinessLevel onward, unchanged.

    The `impl OsProduct` block holds both `all()` and `chain_position()`, so the
    splice starts at the first item AFTER that impl closes.
    """
    return "\n" + src[src.index(TAIL_ANCHOR):]


def rust_str(s):
    return s.replace("\\", "\\\\").replace('"', '\\"')


def build(src):
    cmds = parse_commands(src)
    # remap authored command blocks onto their new slugs
    mapped = {}
    for old, block in cmds.items():
        mapped[RENAME.get(old, old)] = block

    L = []
    A = L.append
    total = REG["total"]
    A("//! The AGENTIK {OS} operative-systems suite — registry + status for the")
    A(f"//! OS tab (TUI). {total} operative systems in {len(REG['groups'])} groups along the")
    A("//! value chain:")
    A("//!")
    for g in REG["groups"]:
        members = len(g["members"])
        A(f"//!   {g['label']:<24} {members:>2} units — {g['purpose']}.")
    A("//!")
    A("//! Each lives under `OS/<slug>/` in the repo (installed to `~/.omega/os/`).")
    A("//! This module answers, cheaply and with NO network: which OSes exist,")
    A("//! where they live on THIS machine, and which concrete readiness surfaces")
    A("//! are present. Static presence is never reported as runtime verification.")
    A("//!")
    A("//! GENERATED from `OS/_registry.json` by `OS/_tools/gen_os_products.py`.")
    A("//! Add or reorder an OS in `OS/_tools/suite.py`, then re-run the generator.")
    A("//! Everything below `OsReadinessLevel` is hand-written and spliced through.")
    A("")
    A("use serde::{Deserialize, Serialize};")
    A("use std::path::{Path, PathBuf};")
    A("")
    A("/// Which group of the suite an OS belongs to — the TUI renders one section")
    A("/// per group, in declaration order.")
    A("#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
    A("pub enum OsGroup {")
    for g in REG["groups"]:
        A(f"    /// {g['label']} — {g['purpose']}.")
        A(f"    {g['rust_variant']},")
    A("}")
    A("")
    A("impl OsGroup {")
    A("    /// Every group, in render order. Callers iterate this instead of")
    A("    /// hand-listing variants, so adding a group never breaks a renderer.")
    A("    pub fn all() -> &'static [OsGroup] {")
    A("        &[")
    for g in REG["groups"]:
        A(f"            OsGroup::{g['rust_variant']},")
    A("        ]")
    A("    }")
    A("")
    A("    /// Short section heading shown in the TUI and the gateway.")
    A("    pub fn label(&self) -> &'static str {")
    A("        match self {")
    for g in REG["groups"]:
        A(f'            OsGroup::{g["rust_variant"]} => "{rust_str(g["label"])}",')
    A("        }")
    A("    }")
    A("")
    A("    /// What this group is for, one line.")
    A("    pub fn purpose(&self) -> &'static str {")
    A("        match self {")
    for g in REG["groups"]:
        A(f'            OsGroup::{g["rust_variant"]} => "{rust_str(g["purpose"])}",')
    A("        }")
    A("    }")
    A("}")
    A("")
    A("/// One operative system of the suite — the static half (identity). The single")
    A("/// source of truth: the TUI tab, `OS/README.md` and install parity all derive")
    A("/// from `OsProduct::all()`; add an OS in `OS/_tools/suite.py`.")
    A("#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
    A("pub struct OsProduct {")
    A("    /// Canonical suite number (00..71) in the AGENTIK {OS} tree.")
    A("    pub num: u8,")
    A("    /// Directory name under `OS/` — also the id used everywhere.")
    A("    pub slug: &'static str,")
    A("    /// Display name.")
    A("    pub name: &'static str,")
    A("    /// One-line focus shown in the detail pane.")
    A("    pub tagline: &'static str,")
    A("    /// The suite group this OS renders under.")
    A("    pub group: OsGroup,")
    A("    /// What you can do with this OS — the command surface shown in the OS-tab")
    A("    /// detail pane as declared capabilities. One line per entry; empty for a")
    A("    /// pre-integration OS (its detail shows the integration pipeline instead).")
    A("    pub commands: &'static [&'static str],")
    A("}")
    A("")
    A("impl OsProduct {")
    A(f"    /// The whole suite: {total} units, grouped and contiguous, in registry order.")
    A("    pub fn all() -> &'static [OsProduct] {")
    A("        &[")
    last_group = None
    preserved = 0
    for u in REG["os"]:
        g = GROUPS[u["group"]]
        if u["group"] != last_group:
            A(f"            // ── {g['label']} — {g['purpose']} " + "─" * max(0, 18 - len(g['purpose']) // 3))
            last_group = u["group"]
        A("            OsProduct {")
        A(f"                num: {u['num']},")
        A(f'                slug: "{rust_str(u["slug"])}",')
        A(f'                name: "{rust_str(u["display"])}",')
        A(f'                tagline: "{rust_str(u["tagline"])}",')
        A(f"                group: OsGroup::{g['rust_variant']},")
        block = mapped.get(u["slug"])
        if block is not None and block.strip():
            preserved += 1
            A("                commands: &[" + block)
            A("                ],")
        else:
            A("                commands: &[],")
        A("            },")
    A("        ]")
    A("    }")
    A("")
    A('    /// "01"..."08" for BUILD OSes (their pipeline position), None for')
    A("    /// every other group. Derived from registry order, never hand-numbered.")
    A("    pub fn chain_position(&self) -> Option<usize> {")
    A("        if self.group != OsGroup::Build {")
    A("            return None;")
    A("        }")
    A("        Self::all()")
    A("            .iter()")
    A("            .filter(|p| p.group == OsGroup::Build)")
    A("            .position(|p| p.slug == self.slug)")
    A("            .map(|i| i + 1)")
    A("    }")
    A("}")
    return "\n".join(L) + splice_tail(src), preserved, total


def main():
    write = "--write" in sys.argv
    src = open(RS, encoding="utf-8").read()
    new, preserved, total = build(src)
    print(f"units emitted            : {total}")
    print(f"groups                   : {len(REG['groups'])}")
    print(f"command arrays preserved : {preserved}")
    print(f"size {len(src)} -> {len(new)} bytes")
    if write:
        open(RS, "w", encoding="utf-8").write(new)
        print(f"WROTE {RS}")
    else:
        print("(dry run, pass --write)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
