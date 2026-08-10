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
    /// What you can do with this OS — the command surface shown in the OS-tab
    /// detail pane (integrated OSes only). One line per entry; empty for a
    /// pre-integration OS (its detail shows the integration pipeline instead).
    pub commands: &'static [&'static str],
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
                commands: &[
                    "Claude / Codex:",
                    "  /brainstorm · /ideation-os   run the imagination + decision council",
                    "  depths: spark(quick) · imagination(wild) · council(multi-agent) · deep",
                    "          red-team(attack it) · converge(decide) · audit(gaps)",
                    "omega-ideation CLI (session state):",
                    "  init         start a session (--title --project-id --depth)",
                    "  frame        set scope, constraints, non-goals",
                    "  dna          set Founder DNA + signature tension",
                    "  add          add a typed ledger item (idea/tension/finding)",
                    "  surface      record + select a product surface candidate",
                    "  surface-config   configure Surface Lab / multi-surface roles",
                    "  evolve       record an evolutionary generation + selection pressure",
                    "  portfolio    record active ideas + portfolio coherence",
                    "  checkpoint   record a challenge-cycle delta",
                    "  audit        compute structural quality gates",
                    "  freeze       version + freeze the selected concept",
                    "  export       export a readable Markdown session",
                    "  handoff      export a structured downstream handoff (→ Research/Blueprint)",
                    "  summary      compact session summary   ·   validate   check the session",
                ],
            },
            OsProduct {
                slug: "researcher-os",
                name: "Researcher OS",
                tagline: "Market Research {OS}: evidence, validation and a bounded decision before Blueprint.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex:",
                    "  /research · /researcher-os   Market Research {OS} (evidence + validation)",
                    "  /market-research <idea>   scan · validate · diligence · deep",
                    "                            audit · delta · continue · status · score · handoff",
                    "  depths: SIGNAL(desk) · VALIDATION(primary) · INVESTMENT_GRADE",
                    "  decisions: GO · PIVOT · HOLD · NO-GO · INSUFFICIENT (always bounded)",
                    "omega-research CLI (workspace state):",
                    "  init         create a research workspace (--project-id --decision --depth)",
                    "  validate     validate structure + machine-checkable rules",
                    "  status       current research status",
                    "  allocate     allocate stable IDs",
                    "  checkpoint   restart-safe checkpoint",
                    "  score        gate + hypothesis diagnostics",
                    "  export       export state or a status view (→ Blueprint input manifest)",
                    "  demo         a small validated demo workspace",
                ],
            },
            OsProduct {
                slug: "blueprint-os",
                name: "Blueprint OS",
                tagline: "The product-definition compiler: idea to a traceable, gated definition pack.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex:",
                    "  /blueprint <idea> · /blueprint-os   the product-definition compiler",
                    "  modes: NEW(compile) · RECOVER(rebuild truth) · EXTEND(add module)",
                    "         REVISE(supersede) · AUDIT(gaps/gates) · DELTA(diff+impact)",
                    "  continue(resume) · status · export <view>",
                    "omega-blueprint CLI (canonical state):",
                    "  init         create the state file (--project-id --project-name --request)",
                    "  validate     schema + 20 gates + handoff + checksum (exit 1 on critical)",
                    "  status       progress, revision, next section, checksum",
                    "  checkpoint   advance revision + save the continuation pointer",
                    "  demo         a valid minimal state to read",
                    "→ stops at BLUEPRINT COMPLETE — STEPPER READY, frozen handoff downstream",
                ],
            },
            OsProduct {
                slug: "designer-os",
                name: "Designer OS (UX/UI)",
                tagline: "Design {OS}: challenge the blueprint into flows, screens, states and a validated Design Handoff.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex:",
                    "  /design-os · /designer-os   the UX/UI compiler + adversarial flow challenger",
                    "  it does: challenge the flow · information architecture · screen/surface",
                    "           contracts · every async state · visual system · responsive + a11y",
                    "  maps to shadcn/ui + STAX (the OmegaOS stack)",
                    "omega-designer CLI (contract validators):",
                    "  intake <file>     validate the Blueprint intake schema",
                    "  handoff <file>    validate the Design Handoff (flows/surfaces/evals/seeds)",
                    "  self-test         run the validator self-test",
                    "→ consumes the Blueprint handoff; emits the Design Handoff for Stepper",
                ],
            },
            OsProduct {
                slug: "stepper-os",
                name: "Stepper OS",
                tagline: "Step-by-step execution: a blueprint walked one verified step at a time.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex:",
                    "  /stepper-os   execute a blueprint step by step (verifier-gated DONE)",
                    "omega-stepper CLI (execution DAG):",
                    "  init         scaffold the project (manifest + sources + first step)",
                    "  validate     schema + DAG + audit Blueprint/Design references resolve",
                    "  status       weighted + raw progress, per-status counts",
                    "  plan         ranked READY candidates + the safe execution wave",
                    "  show / agent-brief   one step's spec / the self-contained agent brief",
                    "  start        claim a READY step (prints the brief: Blueprint + Design refs)",
                    "  verify       run the deterministic checks (no state change)",
                    "  done         verify then close — DONE only if every check passes",
                    "  fail / block / unblock   record a failure / block / lift a block",
                    "  review       record a review verdict (role gate)",
                    "  resume       reconcile interrupted attempts after a restart",
                    "  release-check   PASS only when every target-priority step is DONE",
                    "  report / events   status report / the append-only event log",
                    "→ references BOTH the Blueprint and the Design docs per step",
                ],
            },
            OsProduct {
                slug: "builder-os",
                name: "Builder OS",
                tagline: "The implementation runtime: the Stepper roadmap executed into release-ready code.",
                group: OsGroup::BuildChain,
                commands: &[
                    "Claude / Codex:",
                    "  /build · /builder-os   the autonomous implementation runtime",
                    "  /build preflight · status · plan · run · step <id> · test · verify",
                    "         repair <id> · audit · resume · pause · release-check · report",
                    "omega-builder CLI (evidence state):",
                    "  init         init from the frozen Blueprint handoff + BUILD READY graph",
                    "  validate / status   structure / evidence-backed status",
                    "  sync-step    mirror one Stepper step (spec-hash, required-for-release)",
                    "  claim        claim a READY step (locks, worktree, branch)",
                    "  transition   apply a valid attempt transition",
                    "  record-check   record deterministic check evidence",
                    "  mark-step    mirror a Stepper status after the Verifier decision",
                    "  gate         evaluate a release gate BG01–BG20",
                    "  checkpoint   a recovery checkpoint",
                    "  set-release / finalize   set candidate / final handoff when all gates pass",
                    "  release-check   evaluate terminal release readiness   ·   demo (self-test)",
                    "→ executes the Stepper roadmap; never a competing TODO list",
                ],
            },
            OsProduct {
                slug: "mindset-os",
                name: "Mindset OS",
                tagline: "Jim Rohn identity/wellbeing/wealth OS: evidence-labeled coaching, philosophy compiler, 90-day program.",
                group: OsGroup::Personal,
                commands: &[
                    "Claude / Codex:",
                    "  /mindset-os   Jim Rohn identity/wellbeing/wealth coach",
                    "  labels every claim E1(established) · E2(promising) · S(spiritual)",
                    "                     P(personal) · C(clinical → a professional)",
                    "omega-mindset CLI (workspaces):",
                    "  new --name --output      the weekly workspace (19 files)",
                    "  score <scorecard.json>   validate + summarize a weekly scorecard",
                    "  challenge --output       the 6-month identity challenge",
                    "                           (180 daily + 26 weekly + 6 monthly + state)",
                    "  coach <dir>              one AI growth pass now (Telegram card)",
                    "  coach <dir> --arm        daily 07:00 auto-coaching (disarmed by default)",
                    "  coach <dir> --disarm     stop the loop",
                ],
            },
            OsProduct {
                slug: "habits-os",
                name: "Habits OS",
                tagline: "Habit Tracker {OS}: conversation-first habit system, deterministic state, adaptive reviews and seasons.",
                group: OsGroup::Personal,
                commands: &[
                    "Claude / Codex:",
                    "  /habits · /habits-os   conversation-first habit tracker (chat is the interface)",
                    "omega-habits CLI (SQLite state):",
                    "  init         create/update your profile (--user --season)",
                    "  add          create an accepted habit contract",
                    "  update       a superseding habit-contract version",
                    "  list         list habit contracts",
                    "  log          append explicit or observed evidence",
                    "  correct      supersede a wrong log (invalidates derived reviews)",
                    "  today        rank today's primary habits",
                    "  review       an evidence-bounded review",
                    "  chart        render Mermaid from known opportunities",
                    "  context      compact LLM context   ·   export   user-owned state",
                    "  season       change the operating season (build/crisis/maintain/recover/travel)",
                    "  experiment   a bounded behavior experiment",
                    "  delete       delete user-owned logs/habits/all   ·   doctor   integrity check",
                ],
            },
            OsProduct {
                slug: "execution-os",
                name: "Execution OS",
                tagline: "LLM-first personal execution: ambitions into focused commitments, protected work and shipped evidence.",
                group: OsGroup::Personal,
                commands: &[
                    "/execution-os · /execute   personal execution as a closed control loop",
                    "  Capture → Clarify → Select → Commit → Focus → Prove → Review → Adapt",
                    "omega-execution CLI:",
                    "  init --owner        create your execution state",
                    "  boot                open the day (capacity GREEN/AMBER/RED + must-win)",
                    "  capture             inbox a raw outcome/commitment/idea",
                    "  add-outcome         a measurable outcome to pursue",
                    "  add-commitment      a commitment with one physical next action",
                    "  start / complete    complete requires evidence + acceptance",
                    "  focus / focus-end   protect a 15/25/50/90-min block",
                    "  block / unblock     adaptive recovery, each with a next action",
                    "  defer / cancel / delegate   move or hand off a commitment",
                    "  add-promise         a stakeholder promise (notice-by + consequence)",
                    "  halt                close the day (proof + tomorrow's first action)",
                    "  reset               weekly reset (truth + next-week win + experiment)",
                    "  audit               monthly system audit (system change)",
                ],
            },
            OsProduct {
                slug: "books-os",
                name: "Books OS",
                tagline: "Your library as an operating system: reading, retention and living knowledge.",
                group: OsGroup::Personal,
                commands: &[
                    "Claude / Codex ( /books-os = /alexandria — the librarian ):",
                    "  /setup       calibrate on how YOU learn   ·   /language   reply language",
                    "  /book        full X-Ray of a book   ·   /espresso   90-second version",
                    "  /chapter     a book chapter by chapter   ·   /idea   atlas across many books",
                    "  /compare (/vs)   authors in combat   ·   /apply   to a real business",
                    "  /challenge   10-round sparring on your idea   ·   /decision   decision lab",
                    "  /council     3-5 perspectives   ·   /teach   Feynman triple explanation",
                    "  /quiz /drill   adaptive recall   ·   /cards   flashcards   ·   /review   spaced rep",
                    "  /map (/visual)   diagram   ·   /memory   memory forge   ·   /focus   5-min session",
                    "  /best [topic]   the 50 best books + 50 tips   ·   /bestsellers [niche]   top 100",
                    "  /gem   an underrated idea   ·   /capture /save /applylog   feed the ledger",
                    "omega-books CLI:   open the librarian master agent in a terminal session",
                ],
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
                "execution-os",
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
