//! Rules registry — typed catalogue of OmegaOS operational rules.
//!
//! Each rule has an id (R-NN), title, what it does, which agents it
//! applies to, when it was added, and why. The Info tab renders this
//! registry so users can see the system's actual behavior model.

use serde::{Deserialize, Serialize};

use crate::aisb_agents::AisbAgent;

/// LAW vs RULE tier. Laws are inviolable, universal, render first
/// everywhere, and outrank every rule or task. Rules are operational
/// guidelines that implement the Laws in practice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleKind {
    /// Inviolable, universal — binds every agent, always, outranks every rule or task.
    Law,
    /// Operational guideline — categorized, scoped per agent level.
    Rule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleCategory {
    /// Universal — applies to every agent
    Universal,
    /// Quality gate — enforces verification before "done"
    QualityGate,
    /// Orchestration — controls how agents dispatch & coordinate
    Orchestration,
    /// Reporting — controls how outcomes flow back
    Reporting,
    /// Safety — prevents footguns
    Safety,
}

/// Which agent LEVEL a rule is injected into. This is the single source
/// of truth for "which rules go into which agent's prompt" — the prompt
/// builder calls `rules_for_scope(scope)` when assembling a Master /
/// Oracle / Worker prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleScope {
    /// AISB Master — the global dispatcher chat brain.
    Master,
    /// Every agent everywhere (global system invariants).
    Global,
    /// Oracle — strategic per-project planner/dispatcher.
    Oracle,
    /// Worker — ephemeral task executor.
    Worker,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub id: &'static str,
    pub title: &'static str,
    pub kind: RuleKind,
    pub category: RuleCategory,
    pub description: &'static str,
    /// Specific agents this rule applies to (empty = all agents).
    pub applies_to: &'static [AisbAgent],
    pub added_at: &'static str,
    pub reason: &'static str,
}

/// Hard-coded registry. Adding a new rule = adding an entry here.
pub fn all_rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "L1",
            title: "Code lies — only runtime tells the truth",
            kind: RuleKind::Law,
            category: RuleCategory::Universal,
            description: "Verify behaviour by running the program. Logs, traces, screenshots > assumptions. Before the 3rd code change on the same bug, live runtime evidence is MANDATORY.",
            applies_to: &[],
            added_at: "2026-03-11",
            reason: "Three sessions in a row, agents shipped 'fixed' code that didn't even compile. Runtime is the only proof.",
        },
        Rule {
            id: "L2",
            title: "Researcher, not sycophant",
            kind: RuleKind::Law,
            category: RuleCategory::Universal,
            description: "Challenge flawed premises before coding. Push back with reasoning. Senior engineer standard. No agree-and-code, no fake confidence.",
            applies_to: &[],
            added_at: "2026-03-11",
            reason: "Agents kept saying 'you're right' and re-implementing the same broken fix. The user wanted real engineering pushback.",
        },
        Rule {
            id: "L3",
            title: "Decide and proceed — never wait in a dispatched session",
            kind: RuleKind::Law,
            category: RuleCategory::Orchestration,
            description: "When dispatched as a worker, never ask the user 'should I continue?'. Pick the best path, log the decision, execute. The only legal stop is .done.json or .worker-blocked.json.",
            applies_to: &[AisbAgent::Morpheus, AisbAgent::Niobe, AisbAgent::Seraph, AisbAgent::Keymaker, AisbAgent::Architect],
            added_at: "2026-04-15",
            reason: "A worker stopped mid-mission asking 'which path?' for 10+ minutes. The user wasn't watching. Workers must be autonomous.",
        },
        Rule {
            id: "R-14",
            title: "Ship verification (deploy returns 200)",
            kind: RuleKind::Rule,
            category: RuleCategory::QualityGate,
            description: "When a mission ships, the deploy URL must respond 200 within the timeout window. Push pipeline is part of the gate, not after.",
            applies_to: &[AisbAgent::Oracle, AisbAgent::Morpheus, AisbAgent::Seraph],
            added_at: "2026-04-08",
            reason: "Multiple missions reported 'done' while prod was returning 500. Now no mission is done until prod is healthy.",
        },
        Rule {
            id: "R-18",
            title: "Hybrid dispatch — long missions = rmux, short = Agent tool",
            kind: RuleKind::Rule,
            category: RuleCategory::Orchestration,
            description: "MORPHEUS picks between dispatching a worker to an rmux session (long-running, >5 min) vs spawning an in-process Agent subagent (fast research, <2 min). Don't waste a tmux pane on a 30-second job.",
            applies_to: &[AisbAgent::Morpheus, AisbAgent::Oracle],
            added_at: "2026-04-20",
            reason: "Spawned 40 sub-agents for trivial questions, wasting context. Hybrid dispatch reduced spawn cost by ~70%.",
        },
        Rule {
            id: "R-19",
            title: "Rubric defined before execution",
            kind: RuleKind::Rule,
            category: RuleCategory::QualityGate,
            description: "ORACLE/KEYMAKER writes the success criteria to `outcomes/{oracle}.rubric.md` BEFORE workers start. Grading happens against this rubric, not against vibes.",
            applies_to: &[AisbAgent::Oracle, AisbAgent::Keymaker],
            added_at: "2026-04-08",
            reason: "Workers self-graded with shifting criteria. Rubric upfront forces explicit success.",
        },
        Rule {
            id: "R-21",
            title: "Multi-grader consensus (≥ 2/3 lenses agree)",
            kind: RuleKind::Rule,
            category: RuleCategory::QualityGate,
            description: "Outcomes verified by 3 independent lenses (code-reviewer, debugger, general-purpose). 2/3 must say SATISFIED before status flips to done_clean.",
            applies_to: &[AisbAgent::Seraph, AisbAgent::Oracle],
            added_at: "2026-04-12",
            reason: "Single grader hallucinated passes. Three independent passes = much harder to fool.",
        },
        Rule {
            id: "R-22",
            title: "Regression detection across iterations",
            kind: RuleKind::Rule,
            category: RuleCategory::QualityGate,
            description: "Compare current iteration's artifacts to the previous one. Semantic diff (not just textual). Zero regressions required to ship.",
            applies_to: &[AisbAgent::Seraph],
            added_at: "2026-04-15",
            reason: "Re-runs sometimes broke what previous runs fixed. Diff-based check catches the regression.",
        },
        Rule {
            id: "R-28",
            title: "Cost tracking — token budget per mission",
            kind: RuleKind::Rule,
            category: RuleCategory::QualityGate,
            description: "Track spend per mission. Default cap: 500K tokens. Missions that exceed get a hard stop + escalation to the user.",
            applies_to: &[AisbAgent::Oracle, AisbAgent::Zion],
            added_at: "2026-04-25",
            reason: "Runaway missions burned 2M+ tokens with no signal. Cap forces explicit go-ahead for the expensive ones.",
        },
        Rule {
            id: "R-30",
            title: "Adversarial Popper falsification — ≥12 challenges",
            kind: RuleKind::Rule,
            category: RuleCategory::QualityGate,
            description: "A challenger worker explores ≥12 edge cases trying to falsify the claim. Each challenge must have file:line/log evidence. NO citations = REJECTED.",
            applies_to: &[AisbAgent::Seraph],
            added_at: "2026-05-02",
            reason: "Passing without trying to break it = false confidence. Popper-style falsification before declaring victory.",
        },
        Rule {
            id: "R-35",
            title: "Every claim cited — no citation = rejected",
            kind: RuleKind::Rule,
            category: RuleCategory::Reporting,
            description: "Claims in adversarial passes, audits, and grading require citations (file:line, log line, screenshot). Uncited assertions are auto-rejected.",
            applies_to: &[AisbAgent::Seraph, AisbAgent::Niobe],
            added_at: "2026-05-08",
            reason: "Findings without evidence = noise. Citations make them auditable.",
        },
        Rule {
            id: "TG-SEC",
            title: "Telegram security: chat_id + sender_id allow-list",
            kind: RuleKind::Rule,
            category: RuleCategory::Safety,
            description: "Omega's Telegram bridge accepts messages only from the configured chat_id; if --user-id allow-list is set, sender_id must match. Everything else is silently dropped + logged.",
            applies_to: &[AisbAgent::Link],
            added_at: "2026-05-27",
            reason: "Anyone with the bot token could potentially DM it. Two-level filter ensures only the owner controls the VPS.",
        },
        Rule {
            id: "AISB-AUTOSPAWN",
            title: "Master AISB auto-spawned on every launch",
            kind: RuleKind::Rule,
            category: RuleCategory::Orchestration,
            description: "When `omega menu` starts, the AISB Master session is auto-created if missing. Pinned at the top of the session list, system prompt loaded via --append-system-prompt-file (invisible), conversation resumed via --continue.",
            applies_to: &[AisbAgent::Oracle],
            added_at: "2026-05-27",
            reason: "The user wants a persistent always-on chat with AISB. No setup step, no manual launch.",
        },
        Rule {
            id: "SCOPE-CLAIM",
            title: "File-lock scope claims prevent concurrent edits",
            kind: RuleKind::Rule,
            category: RuleCategory::Safety,
            description: "Workers declare `files_owned` on spawn. A new worker is rejected if its files overlap with an active claim. Claims auto-release on done_clean.",
            applies_to: &[AisbAgent::Oracle, AisbAgent::Morpheus],
            added_at: "2026-05-26",
            reason: "Two workers editing the same file produced merge conflicts and lost work. Hard locks at dispatch time fix this.",
        },
        Rule {
            id: "AUTO-NAMING",
            title: "Auto-generated session names (claude-1, codex-2, ...)",
            kind: RuleKind::Rule,
            category: RuleCategory::Orchestration,
            description: "When creating a new agent session via the menu, the name is generated automatically from agent + count. User skips the name-input step entirely.",
            applies_to: &[],
            added_at: "2026-05-27",
            reason: "Forcing the user to invent a name every time was friction. Auto-naming + chat focus = zero clicks to talk to a new agent.",
        },
        Rule {
            id: "SIMPLICITY-COMPLETE",
            title: "Simplicity does not mean incomplete",
            kind: RuleKind::Rule,
            category: RuleCategory::Universal,
            description: "Solve problems with the smallest correct design. No speculative abstractions, no parallel re-implementations of an existing pattern (Hermès, OpenClaw, claude-mux), but the result must still cover the full feature surface the user asked for.",
            applies_to: &[],
            added_at: "2026-05-27",
            reason: "Earlier Telegram bridge attempts ballooned into duplicate pipelines that did less than the simple version. Simplicity is the constraint, not an excuse to ship half-features.",
        },
        Rule {
            id: "FILE-SIZE-LIMIT",
            title: "Files over 1500 lines must be split",
            kind: RuleKind::Rule,
            category: RuleCategory::Universal,
            description: "Any source file that crosses 1500 lines is a refactor signal. Split by responsibility (handlers / oauth / callbacks / menus) before adding more code. Hard cap: 2000 lines = build alarm.",
            applies_to: &[],
            added_at: "2026-05-27",
            reason: "Files like telegram_bridge.rs grew past 2400 lines and became impossible to navigate / review. Keeping files small forces cohesion.",
        },
        Rule {
            id: "RUST-BUN-DEFAULT",
            title: "Default to Rust + Bun for everything written in this repo",
            kind: RuleKind::Rule,
            category: RuleCategory::Universal,
            description: "OmegaOS itself: Rust for core/CLI/TUI/SDK. Scripts/tooling: Bun (TypeScript) over Node when a runtime is needed. Python only for ML/data-science niches. No bash mega-scripts.",
            applies_to: &[],
            added_at: "2026-05-27",
            reason: "Cold-start speed + single-binary distribution matter for an OS-grade tool. Bun gives ~25ms startup + native TS so scripting stays fast.",
        },
        Rule {
            id: "MASTER-CHANNEL-ONLY",
            title: "AISB Master never works — it only dispatches to named oracles",
            kind: RuleKind::Rule,
            category: RuleCategory::Orchestration,
            description: "The Telegram/AISB Master is a discussion channel, not a worker. It is forbidden from editing files, running builds, audits, fixes, or any artifact-producing work. ALL work is dispatched to a correctly-named Oracle (project work → oracle-<Project>-<n>; internal VPS/OmegaOS work → oracle-OmegaOS-<n>). Master only clarifies intent, classifies, dispatches, and relays reports.",
            applies_to: &[AisbAgent::Oracle],
            added_at: "2026-05-28",
            reason: "The bot kept doing work inline instead of dispatching, blurring the channel/worker boundary and bypassing the oracle pipeline + quality gates.",
        },
        Rule {
            id: "PROMPT-COMPLETENESS",
            title: "Every prompt = task list + final completeness check",
            kind: RuleKind::Rule,
            category: RuleCategory::QualityGate,
            description: "On any user prompt, enumerate the implied tasks, track them (TaskCreate/TaskUpdate), and at end-of-turn verify every task was completed. Missed tasks become explicit pending items reported back, not silent gaps.",
            applies_to: &[AisbAgent::Oracle, AisbAgent::Morpheus],
            added_at: "2026-05-27",
            reason: "Multi-part prompts (UI change + scroll + GitHub alignment + symlink explanation) were losing the secondary tasks. Tracked list + end-of-turn verification stops the drift.",
        },
    ]
}

pub fn rules_by_category(cat: RuleCategory) -> Vec<Rule> {
    all_rules().into_iter().filter(|r| r.category == cat).collect()
}

/// All Laws (the inviolable tier). Order preserved from `all_rules()`.
pub fn laws() -> Vec<Rule> {
    all_rules().into_iter().filter(|r| r.kind == RuleKind::Law).collect()
}

/// All operational rules (everything that is NOT a Law).
pub fn operational_rules() -> Vec<Rule> {
    all_rules().into_iter().filter(|r| r.kind == RuleKind::Rule).collect()
}

impl Rule {
    /// Which agent levels this rule is injected into. Per-id overrides
    /// first, then a category-based fallback. This is the single source
    /// of truth consumed by the prompt builder (`rules_for_scope`).
    pub fn scopes(&self) -> Vec<RuleScope> {
        use RuleScope::*;
        // Laws are universal by definition — bind every agent everywhere.
        if self.kind == RuleKind::Law {
            return vec![Master, Global, Oracle, Worker];
        }
        // Explicit per-rule overrides (the ones with a clear home).
        match self.id {
            // Master-only orchestration behaviors.
            "AISB-AUTOSPAWN" | "AUTO-NAMING" | "MASTER-CHANNEL-ONLY" => return vec![Master],
            // Oracle-only dispatch/coordination rules.
            "SCOPE-CLAIM" | "R-18" | "R-19" => return vec![Oracle],
            // Quality gates the executor + planner both honor.
            "R-14" | "R-21" | "R-22" | "R-30" | "R-35" | "R-28" => {
                return vec![Oracle, Worker]
            }
            // Code-quality rules a worker editing files must follow.
            "RUST-BUN-DEFAULT" | "FILE-SIZE-LIMIT" | "SIMPLICITY-COMPLETE" => {
                return vec![Oracle, Worker]
            }
            // Completeness discipline for the planning levels.
            "PROMPT-COMPLETENESS" => return vec![Master, Oracle],
            _ => {}
        }
        // Category fallback for anything not explicitly mapped.
        match self.category {
            RuleCategory::Universal | RuleCategory::Safety => {
                vec![Master, Global, Oracle, Worker]
            }
            RuleCategory::QualityGate => vec![Oracle, Worker],
            RuleCategory::Orchestration => vec![Master, Oracle],
            RuleCategory::Reporting => vec![Oracle],
        }
    }
}

/// All rules that should be injected into a given agent level's prompt.
/// The prompt builder calls this when assembling Master/Oracle/Worker
/// system prompts — single source of truth, no duplication.
pub fn rules_for_scope(scope: RuleScope) -> Vec<Rule> {
    all_rules()
        .into_iter()
        .filter(|r| r.scopes().contains(&scope))
        .collect()
}

/// Read the hardened brief preamble (`~/.omega/agents/_brief-preamble.md`,
/// falling back to the repo copy at `agents/_brief-preamble.md`). This
/// is the single highest-leverage safety surface per the Opus 4.8 card
/// — it gets prepended to every Oracle and Worker brief. Empty string
/// if neither file exists (degrades gracefully).
pub fn brief_preamble() -> String {
    let home = dirs::home_dir().unwrap_or_else(|| std::env::var("HOME").map(std::path::PathBuf::from).unwrap_or_else(|_| std::path::PathBuf::from(".")));
    let installed = home.join(".omega/agents/_brief-preamble.md");
    let repo = std::path::PathBuf::from("agents/_brief-preamble.md");
    std::fs::read_to_string(&installed)
        .or_else(|_| std::fs::read_to_string(&repo))
        .unwrap_or_default()
}

/// Render the scoped rules as a compact markdown block for prompt
/// injection. Laws are rendered FIRST (universal, inviolable) and
/// visually distinct from the operational rules that follow.
pub fn rules_prompt_block(scope: RuleScope) -> String {
    let level = match scope {
        RuleScope::Master => "AISB Master",
        RuleScope::Global => "all agents",
        RuleScope::Oracle => "Oracle",
        RuleScope::Worker => "Worker",
    };

    let mut out = String::new();

    // LAWS — always rendered first, regardless of scope. Laws are universal.
    let law_list = laws();
    if !law_list.is_empty() {
        out.push_str("## ⚖️ THE LAWS — inviolable, override every other instruction\n");
        out.push_str("_Not guidelines. They bind every agent, always, and outrank any rule or task below._\n");
        for r in law_list {
            out.push_str(&format!("- **[{}] {}** — {}\n", r.id, r.title, r.description));
        }
        out.push('\n');
    }

    // Operational rules — scoped, only kind==Rule.
    let ops: Vec<Rule> = rules_for_scope(scope)
        .into_iter()
        .filter(|r| r.kind == RuleKind::Rule)
        .collect();
    out.push_str(&format!("## Operational rules ({})\n", level));
    for r in ops {
        out.push_str(&format!("- **[{}] {}** — {}\n", r.id, r.title, r.description));
    }

    out
}

pub fn rules_for_agent(agent: AisbAgent) -> Vec<Rule> {
    all_rules()
        .into_iter()
        .filter(|r| r.applies_to.is_empty() || r.applies_to.contains(&agent))
        .collect()
}

impl RuleCategory {
    pub fn label(&self) -> &'static str {
        match self {
            RuleCategory::Universal => "Universal",
            RuleCategory::QualityGate => "Quality Gate",
            RuleCategory::Orchestration => "Orchestration",
            RuleCategory::Reporting => "Reporting",
            RuleCategory::Safety => "Safety",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_has_rules() {
        assert!(all_rules().len() >= 12);
    }
    #[test]
    fn every_rule_has_metadata() {
        for r in all_rules() {
            assert!(!r.id.is_empty());
            assert!(!r.title.is_empty());
            assert!(!r.description.is_empty());
            assert!(!r.reason.is_empty());
            assert!(!r.added_at.is_empty());
        }
    }

    #[test]
    fn laws_count_and_kind() {
        let l = laws();
        assert_eq!(l.len(), 3, "expected exactly 3 laws (L1, L2, L3)");
        for r in &l {
            assert_eq!(r.kind, RuleKind::Law);
        }
        let ids: Vec<&str> = l.iter().map(|r| r.id).collect();
        assert_eq!(ids, vec!["L1", "L2", "L3"]);
    }

    #[test]
    fn laws_are_universal() {
        use RuleScope::*;
        for r in laws() {
            assert_eq!(
                r.scopes(),
                vec![Master, Global, Oracle, Worker],
                "law {} must be universal",
                r.id
            );
        }
    }

    #[test]
    fn operational_rules_have_no_laws() {
        for r in operational_rules() {
            assert_ne!(r.kind, RuleKind::Law, "operational_rules leaked a law: {}", r.id);
        }
    }

    #[test]
    fn prompt_block_renders_laws_before_operational() {
        let block = rules_prompt_block(RuleScope::Worker);
        assert!(block.contains("THE LAWS"), "missing LAWS header: {}", block);
        let laws_idx = block.find("[L1]").expect("L1 must appear in block");
        let first_op = block.find("[R-").expect("at least one R- rule must appear");
        assert!(
            laws_idx < first_op,
            "laws must render before operational rules: L1={} R-={}",
            laws_idx,
            first_op
        );
    }
}
