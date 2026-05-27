//! Rules registry — typed catalogue of OmegaOS operational rules.
//!
//! Each rule has an id (R-NN), title, what it does, which agents it
//! applies to, when it was added, and why. The Info tab renders this
//! registry so users can see the system's actual behavior model.

use serde::{Deserialize, Serialize};

use crate::aisb_agents::AisbAgent;

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

#[derive(Debug, Clone)]
pub struct Rule {
    pub id: &'static str,
    pub title: &'static str,
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
            category: RuleCategory::Universal,
            description: "Verify behaviour by running the program. Logs, traces, screenshots > assumptions. Before the 3rd code change on the same bug, live runtime evidence is MANDATORY.",
            applies_to: &[],
            added_at: "2026-03-11",
            reason: "Three sessions in a row, agents shipped 'fixed' code that didn't even compile. Runtime is the only proof.",
        },
        Rule {
            id: "L2",
            title: "Researcher, not sycophant",
            category: RuleCategory::Universal,
            description: "Challenge flawed premises before coding. Push back with reasoning. Senior engineer standard. No agree-and-code, no fake confidence.",
            applies_to: &[],
            added_at: "2026-03-11",
            reason: "Agents kept saying 'you're right' and re-implementing the same broken fix. The user wanted real engineering pushback.",
        },
        Rule {
            id: "L3",
            title: "Decide and proceed — never wait in a dispatched session",
            category: RuleCategory::Orchestration,
            description: "When dispatched as a worker, never ask the user 'should I continue?'. Pick the best path, log the decision, execute. The only legal stop is .done.json or .worker-blocked.json.",
            applies_to: &[AisbAgent::Morpheus, AisbAgent::Niobe, AisbAgent::Seraph, AisbAgent::Keymaker, AisbAgent::Architect],
            added_at: "2026-04-15",
            reason: "A worker stopped mid-mission asking 'which path?' for 10+ minutes. The user wasn't watching. Workers must be autonomous.",
        },
        Rule {
            id: "R-14",
            title: "Ship verification (deploy returns 200)",
            category: RuleCategory::QualityGate,
            description: "When a mission ships, the deploy URL must respond 200 within the timeout window. Push pipeline is part of the gate, not after.",
            applies_to: &[AisbAgent::Oracle, AisbAgent::Morpheus, AisbAgent::Seraph],
            added_at: "2026-04-08",
            reason: "Multiple missions reported 'done' while prod was returning 500. Now no mission is done until prod is healthy.",
        },
        Rule {
            id: "R-18",
            title: "Hybrid dispatch — long missions = rmux, short = Agent tool",
            category: RuleCategory::Orchestration,
            description: "MORPHEUS picks between dispatching a worker to an rmux session (long-running, >5 min) vs spawning an in-process Agent subagent (fast research, <2 min). Don't waste a tmux pane on a 30-second job.",
            applies_to: &[AisbAgent::Morpheus, AisbAgent::Oracle],
            added_at: "2026-04-20",
            reason: "Spawned 40 sub-agents for trivial questions, wasting context. Hybrid dispatch reduced spawn cost by ~70%.",
        },
        Rule {
            id: "R-19",
            title: "Rubric defined before execution",
            category: RuleCategory::QualityGate,
            description: "ORACLE/KEYMAKER writes the success criteria to `outcomes/{oracle}.rubric.md` BEFORE workers start. Grading happens against this rubric, not against vibes.",
            applies_to: &[AisbAgent::Oracle, AisbAgent::Keymaker],
            added_at: "2026-04-08",
            reason: "Workers self-graded with shifting criteria. Rubric upfront forces explicit success.",
        },
        Rule {
            id: "R-21",
            title: "Multi-grader consensus (≥ 2/3 lenses agree)",
            category: RuleCategory::QualityGate,
            description: "Outcomes verified by 3 independent lenses (code-reviewer, debugger, general-purpose). 2/3 must say SATISFIED before status flips to done_clean.",
            applies_to: &[AisbAgent::Seraph, AisbAgent::Oracle],
            added_at: "2026-04-12",
            reason: "Single grader hallucinated passes. Three independent passes = much harder to fool.",
        },
        Rule {
            id: "R-22",
            title: "Regression detection across iterations",
            category: RuleCategory::QualityGate,
            description: "Compare current iteration's artifacts to the previous one. Semantic diff (not just textual). Zero regressions required to ship.",
            applies_to: &[AisbAgent::Seraph],
            added_at: "2026-04-15",
            reason: "Re-runs sometimes broke what previous runs fixed. Diff-based check catches the regression.",
        },
        Rule {
            id: "R-28",
            title: "Cost tracking — token budget per mission",
            category: RuleCategory::QualityGate,
            description: "Track spend per mission. Default cap: 500K tokens. Missions that exceed get a hard stop + escalation to the user.",
            applies_to: &[AisbAgent::Oracle, AisbAgent::Zion],
            added_at: "2026-04-25",
            reason: "Runaway missions burned 2M+ tokens with no signal. Cap forces explicit go-ahead for the expensive ones.",
        },
        Rule {
            id: "R-30",
            title: "Adversarial Popper falsification — ≥12 challenges",
            category: RuleCategory::QualityGate,
            description: "A challenger worker explores ≥12 edge cases trying to falsify the claim. Each challenge must have file:line/log evidence. NO citations = REJECTED.",
            applies_to: &[AisbAgent::Seraph],
            added_at: "2026-05-02",
            reason: "Passing without trying to break it = false confidence. Popper-style falsification before declaring victory.",
        },
        Rule {
            id: "R-35",
            title: "Every claim cited — no citation = rejected",
            category: RuleCategory::Reporting,
            description: "Claims in adversarial passes, audits, and grading require citations (file:line, log line, screenshot). Uncited assertions are auto-rejected.",
            applies_to: &[AisbAgent::Seraph, AisbAgent::Niobe],
            added_at: "2026-05-08",
            reason: "Findings without evidence = noise. Citations make them auditable.",
        },
        Rule {
            id: "TG-SEC",
            title: "Telegram security: chat_id + sender_id allow-list",
            category: RuleCategory::Safety,
            description: "Omega's Telegram bridge accepts messages only from the configured chat_id; if --user-id allow-list is set, sender_id must match. Everything else is silently dropped + logged.",
            applies_to: &[AisbAgent::Link],
            added_at: "2026-05-27",
            reason: "Anyone with the bot token could potentially DM it. Two-level filter ensures only the owner controls the VPS.",
        },
        Rule {
            id: "AISB-AUTOSPAWN",
            title: "Master AISB auto-spawned on every launch",
            category: RuleCategory::Orchestration,
            description: "When `omega menu` starts, the AISB Master session is auto-created if missing. Pinned at the top of the session list, system prompt loaded via --append-system-prompt-file (invisible), conversation resumed via --continue.",
            applies_to: &[AisbAgent::Oracle],
            added_at: "2026-05-27",
            reason: "The user wants a persistent always-on chat with AISB. No setup step, no manual launch.",
        },
        Rule {
            id: "SCOPE-CLAIM",
            title: "File-lock scope claims prevent concurrent edits",
            category: RuleCategory::Safety,
            description: "Workers declare `files_owned` on spawn. A new worker is rejected if its files overlap with an active claim. Claims auto-release on done_clean.",
            applies_to: &[AisbAgent::Oracle, AisbAgent::Morpheus],
            added_at: "2026-05-26",
            reason: "Two workers editing the same file produced merge conflicts and lost work. Hard locks at dispatch time fix this.",
        },
        Rule {
            id: "AUTO-NAMING",
            title: "Auto-generated session names (claude-1, codex-2, ...)",
            category: RuleCategory::Orchestration,
            description: "When creating a new agent session via the menu, the name is generated automatically from agent + count. User skips the name-input step entirely.",
            applies_to: &[],
            added_at: "2026-05-27",
            reason: "Forcing the user to invent a name every time was friction. Auto-naming + chat focus = zero clicks to talk to a new agent.",
        },
    ]
}

pub fn rules_by_category(cat: RuleCategory) -> Vec<Rule> {
    all_rules().into_iter().filter(|r| r.category == cat).collect()
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
}
