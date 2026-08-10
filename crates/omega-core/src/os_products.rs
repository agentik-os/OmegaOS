//! The AgentikOS operative-systems suite — registry + status for the OS tab
//! (TUI). Six products, in operator order: Mindset, Habits, Brainstorm,
//! Blueprint, Stepper, Builder. Each lives under `OS/<slug>/` in the repo
//! (installed to `~/.omega/os/`); payloads arrive as zips via the Deposit box
//! and are unpacked in place. This module answers, cheaply and with NO network:
//! which OSes exist, where they live on THIS machine, and whether their
//! payload has been integrated yet.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Which group of the suite an OS belongs to — the TUI renders one section
/// per group, build chain first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsGroup {
    /// The product pipeline, in chain order:
    /// 01 Ideation → 02 Researcher → 03 Blueprint → 04 Designer (UX/UI) →
    /// 05 Stepper → 06 Builder.
    BuildChain,
    /// The personal operative systems (Mindset, Habits, Books, …).
    Personal,
}

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
    /// The suite group this OS renders under.
    pub group: OsGroup,
}

impl OsProduct {
    /// The suite. The BUILD CHAIN first, in pipeline order (01→06), then the
    /// personal OSes. Chain position = index within the BuildChain group + 1.
    pub fn all() -> &'static [OsProduct] {
        &[
            OsProduct {
                slug: "ideation-os",
                name: "Ideation OS",
                tagline: "Brainstorm {OS} v3: multi-agent imagination + decision council, lineage and a frozen concept handoff.",
                group: OsGroup::BuildChain,
            },
            OsProduct {
                slug: "researcher-os",
                name: "Researcher OS",
                tagline: "Market Research {OS}: evidence, validation and a bounded decision before Blueprint.",
                group: OsGroup::BuildChain,
            },
            OsProduct {
                slug: "blueprint-os",
                name: "Blueprint OS",
                tagline: "The product-definition compiler: idea to a traceable, gated definition pack.",
                group: OsGroup::BuildChain,
            },
            OsProduct {
                slug: "designer-os",
                name: "Designer OS (UX/UI)",
                tagline: "UX and UI design: contracts turned into screens, flows and a design system.",
                group: OsGroup::BuildChain,
            },
            OsProduct {
                slug: "stepper-os",
                name: "Stepper OS",
                tagline: "Step-by-step execution: a blueprint walked one verified step at a time.",
                group: OsGroup::BuildChain,
            },
            OsProduct {
                slug: "builder-os",
                name: "Builder OS",
                tagline: "The implementation runtime: the Stepper roadmap executed into release-ready code.",
                group: OsGroup::BuildChain,
            },
            OsProduct {
                slug: "mindset-os",
                name: "Mindset OS",
                tagline: "Jim Rohn identity/wellbeing/wealth OS: evidence-labeled coaching, philosophy compiler, 90-day program.",
                group: OsGroup::Personal,
            },
            OsProduct {
                slug: "habits-os",
                name: "Habits OS",
                tagline: "Habit design, tracking and consistency: intent turned into daily execution.",
                group: OsGroup::Personal,
            },
            OsProduct {
                slug: "books-os",
                name: "Books OS",
                tagline: "Your library as an operating system: reading, retention and living knowledge.",
                group: OsGroup::Personal,
            },
        ]
    }

    /// "01"…"06" for build-chain OSes (their pipeline position), None for
    /// personal OSes. Derived from registry order — never hand-numbered.
    pub fn chain_position(&self) -> Option<usize> {
        if self.group != OsGroup::BuildChain {
            return None;
        }
        Self::all()
            .iter()
            .filter(|p| p.group == OsGroup::BuildChain)
            .position(|p| p.slug == self.slug)
            .map(|i| i + 1)
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
    /// A dedicated Telegram bot is wired for this OS (`os-<slug>` entry in
    /// `~/.omega/agent-bots.json`, linked via `omega-os-bot`).
    pub bot_linked: bool,
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

/// Integrated = the OS dir contains a real payload beyond its scaffold.
/// Scaffold files every OS carries from day one — the placeholder README,
/// the MASTER.md master-agent prompt, the ledger/ working dir a linked
/// Telegram bot accumulates, dotfiles — do NOT count as integration.
/// Fast + local — safe on tab entry / F5.
fn dir_status(dir: &Path) -> OsStatus {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return OsStatus::AwaitingDrop;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == "README.md" || name == "MASTER.md" || name == "ledger" || name.starts_with('.') {
            continue;
        }
        return OsStatus::Integrated;
    }
    OsStatus::AwaitingDrop
}

/// Bot keys present in `~/.omega/agent-bots.json` (one read for the list).
fn linked_bot_keys() -> std::collections::HashSet<String> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));
    let omega_dir = std::env::var("OMEGA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".omega"));
    let Ok(raw) = std::fs::read_to_string(omega_dir.join("agent-bots.json")) else {
        return Default::default();
    };
    serde_json::from_str::<serde_json::Value>(&raw)
        .ok()
        .and_then(|v| {
            v.as_object()
                .map(|map| map.keys().cloned().collect())
        })
        .unwrap_or_default()
}

/// The whole suite with per-machine status, in product order.
pub fn list_os_entries() -> Vec<OsEntry> {
    let root = os_root();
    let bots = linked_bot_keys();
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
                bot_linked: bots.contains(&format!("os-{}", p.slug)),
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
                "ideation-os",
                "researcher-os",
                "blueprint-os",
                "designer-os",
                "stepper-os",
                "builder-os",
                "mindset-os",
                "habits-os",
                "books-os"
            ]
        );
    }

    #[test]
    fn chain_positions_are_derived_and_personal_oses_have_none() {
        let by_slug = |slug: &str| {
            OsProduct::all()
                .iter()
                .find(|p| p.slug == slug)
                .unwrap()
                .chain_position()
        };
        assert_eq!(by_slug("ideation-os"), Some(1));
        assert_eq!(by_slug("blueprint-os"), Some(3));
        assert_eq!(by_slug("stepper-os"), Some(5));
        assert_eq!(by_slug("builder-os"), Some(6));
        assert_eq!(by_slug("books-os"), None);
        assert_eq!(by_slug("mindset-os"), None);
    }

    #[test]
    fn placeholder_only_dir_is_awaiting_and_payload_is_integrated() {
        let tmp = std::env::temp_dir().join(format!("os-products-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("README.md"), "# placeholder").unwrap();
        std::fs::write(tmp.join("MASTER.md"), "# master agent").unwrap();
        std::fs::create_dir_all(tmp.join("ledger")).unwrap();
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
