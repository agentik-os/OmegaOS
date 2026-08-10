//! The AgentikOS operative-systems suite — registry + status for the OS tab
//! (TUI). Six products, in operator order: Mindset, Habits, Brainstorm,
//! Blueprint, Stepper, Builder. Each lives under `OS/<slug>/` in the repo
//! (installed to `~/.omega/os/`); payloads arrive as zips via the Deposit box
//! and are unpacked in place. This module answers, cheaply and with NO network:
//! which OSes exist, where they live on THIS machine, and whether their
//! payload has been integrated yet.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// One operative system of the suite — the static half (identity). The single
/// source of truth: the TUI tab, `OS/README.md` and install parity all derive
/// from `OsProduct::all()`; add or reorder an OS HERE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OsProduct {
    /// Directory name under `OS/` — also the id used everywhere.
    pub slug: &'static str,
    /// Display name.
    pub name: &'static str,
    /// One-line focus shown in the detail pane.
    pub tagline: &'static str,
}

impl OsProduct {
    /// The suite, in the operator's product order.
    pub fn all() -> &'static [OsProduct] {
        &[
            OsProduct {
                slug: "mindset-os",
                name: "Mindset OS",
                tagline: "Mental models and mindset engineering: how you think before you build.",
            },
            OsProduct {
                slug: "habits-os",
                name: "Habits OS",
                tagline: "Habit design, tracking and consistency: intent turned into daily execution.",
            },
            OsProduct {
                slug: "brainstorm-os",
                name: "Brainstorm OS",
                tagline: "Idea generation and capture: produce, rank and store ideas.",
            },
            OsProduct {
                slug: "blueprint-os",
                name: "Blueprint OS",
                tagline: "Product blueprints and design: an idea turned into a complete build plan.",
            },
            OsProduct {
                slug: "stepper-os",
                name: "Stepper OS",
                tagline: "Step-by-step execution: a blueprint walked one verified step at a time.",
            },
            OsProduct {
                slug: "builder-os",
                name: "Builder OS",
                tagline: "Building and shipping: assemble, test and deliver the product.",
            },
        ]
    }
}

/// The dynamic half: has this OS's payload been integrated on this machine?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OsStatus {
    /// Directory absent or only the placeholder README — the zip has not
    /// landed yet.
    AwaitingDrop,
    /// The directory carries a payload beyond the placeholder.
    Integrated,
}

/// One row for the OS tab: identity + where it lives here + integration state.
#[derive(Debug, Clone)]
pub struct OsEntry {
    pub product: OsProduct,
    pub status: OsStatus,
    /// `<os_root>/<slug>` when a root was found (the dir itself may not exist
    /// yet for an OS added to the registry before its folder).
    pub path: Option<PathBuf>,
}

/// Locate the `OS/` suite root. Order: `OMEGA_OS_ROOT` env override, then the
/// repo relative to the running exe (a dev box running `target/…/omega`), then
/// a walk up from the current dir, then well-known checkouts, then the
/// INSTALLED copy `~/.omega/os` — last, so a checkout always wins and an
/// operator editing the suite sees it immediately. Same resolution grammar as
/// `marketing::capabilities_toml_path`.
pub fn os_root() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("OMEGA_OS_ROOT") {
        let p = p.trim();
        if !p.is_empty() {
            let pb = PathBuf::from(p);
            if pb.is_dir() {
                return Some(pb);
            }
        }
    }

    let is_suite_root = |d: &Path| -> Option<PathBuf> {
        let cand = d.join("OS");
        // Require a known slug inside, so a random `OS/` dir on the walk-up
        // path can't hijack the suite.
        if cand.is_dir() && cand.join("mindset-os").is_dir() {
            return Some(cand);
        }
        None
    };

    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(|p| p.to_path_buf());
        while let Some(d) = dir {
            if let Some(found) = is_suite_root(&d) {
                return Some(found);
            }
            dir = d.parent().map(|p| p.to_path_buf());
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        let mut dir = Some(cwd);
        while let Some(d) = dir {
            if let Some(found) = is_suite_root(&d) {
                return Some(found);
            }
            dir = d.parent().map(|p| p.to_path_buf());
        }
    }

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));
    for base in [
        home.join("Station").join("SideBusiness").join("OmegaOS"),
        home.join("OmegaOS"),
    ] {
        if let Some(found) = is_suite_root(&base) {
            return Some(found);
        }
    }

    let omega_dir = std::env::var("OMEGA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".omega"));
    let installed = omega_dir.join("os");
    if installed.is_dir() {
        return Some(installed);
    }
    None
}

/// Integrated = the OS dir contains anything beyond its placeholder
/// (`README.md` / dotfiles). Fast + local — safe on tab entry / F5.
fn dir_status(dir: &Path) -> OsStatus {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return OsStatus::AwaitingDrop;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == "README.md" || name.starts_with('.') {
            continue;
        }
        return OsStatus::Integrated;
    }
    OsStatus::AwaitingDrop
}

/// The whole suite with per-machine status, in product order.
pub fn list_os_entries() -> Vec<OsEntry> {
    let root = os_root();
    OsProduct::all()
        .iter()
        .map(|p| {
            let path = root.as_ref().map(|r| r.join(p.slug));
            let status = path
                .as_deref()
                .filter(|d| d.is_dir())
                .map(dir_status)
                .unwrap_or(OsStatus::AwaitingDrop);
            OsEntry {
                product: *p,
                status,
                path,
            }
        })
        .collect()
}

impl OsEntry {
    /// Status glyph for the list: 🟢 integrated / ⚪ awaiting its drop.
    pub fn glyph(&self) -> &'static str {
        match self.status {
            OsStatus::Integrated => "🟢",
            OsStatus::AwaitingDrop => "⚪",
        }
    }

    pub fn status_label(&self) -> &'static str {
        match self.status {
            OsStatus::Integrated => "integrated",
            OsStatus::AwaitingDrop => "awaiting drop (zip via Deposit)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suite_has_the_six_products_in_operator_order() {
        let slugs: Vec<&str> = OsProduct::all().iter().map(|p| p.slug).collect();
        assert_eq!(
            slugs,
            vec![
                "mindset-os",
                "habits-os",
                "brainstorm-os",
                "blueprint-os",
                "stepper-os",
                "builder-os"
            ]
        );
    }

    #[test]
    fn placeholder_only_dir_is_awaiting_and_payload_is_integrated() {
        let tmp = std::env::temp_dir().join(format!("os-products-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("README.md"), "# placeholder").unwrap();
        assert_eq!(dir_status(&tmp), OsStatus::AwaitingDrop);
        std::fs::write(tmp.join("app.py"), "payload").unwrap();
        assert_eq!(dir_status(&tmp), OsStatus::Integrated);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn entries_cover_the_whole_suite() {
        let entries = list_os_entries();
        assert_eq!(entries.len(), OsProduct::all().len());
    }
}
