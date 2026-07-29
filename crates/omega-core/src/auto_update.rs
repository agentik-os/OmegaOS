//! Daily update check + automatic apply — the decision layer.
//!
//! `omega update --auto` runs from cron on every install. This module owns the
//! part worth testing: given the state of the checkout, the machine and the
//! previous runs, WHAT should happen? The CLI does the git and install.sh work.
//!
//! Auto-applying code fetched from a remote is a real trust decision, so the
//! decision is deliberately conservative and every refusal is named:
//!
//! * nothing to pull → do nothing, cheaply and silently (the common case)
//! * local edits or unpushed commits → never touch them, ever
//! * an agent is mid-turn → defer to tomorrow rather than rebuild under it
//! * the same commit failed [`FAILURE_CAP`] times → stop, escalate to a human
//!   (R-LOOP: retrying a 4th time is thrash, not progress)
//!
//! The failure counter is keyed by TARGET COMMIT, not by a bare tally: a new
//! commit on the remote is a genuinely new attempt and starts from zero, while
//! one bad commit can never be retried forever.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::config::AutoUpdatePolicy;

/// Consecutive failed applies of the SAME target commit before the cron stops
/// trying and asks for a human. Mirrors `loop_guard::THRASH_CAP` — same reason.
pub const FAILURE_CAP: u32 = 3;

/// What the caller observed about the checkout before deciding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckoutState {
    /// Commits on origin that we do not have.
    pub behind: usize,
    /// Local commits not on origin.
    pub ahead: usize,
    /// Uncommitted changes present.
    pub dirty: bool,
    /// Short sha of the remote tip — the update target. Empty when unknown.
    pub target: String,
}

/// What the cron decided to do, and why. Every variant carries the sentence
/// that goes into the log, so the reason can never drift from the decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Policy is `off` — the cron does nothing at all.
    Disabled,
    /// Already current. The overwhelmingly common daily outcome.
    UpToDate,
    /// An update exists and will be installed now.
    Apply { behind: usize, target: String },
    /// An update exists; policy is `check`, so only report it.
    NotifyOnly { behind: usize, target: String },
    /// An update exists but applying it would risk something. Never retried
    /// blindly — the reason says what the user has to resolve.
    Skip { reason: SkipReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// Uncommitted local changes — an update would have to clobber them.
    DirtyTree,
    /// Local commits not pushed — fast-forward is impossible without losing them.
    LocalCommits { ahead: usize },
    /// An agent session is mid-turn; rebuilding under it can wait a day.
    AgentWorking { session: String },
    /// This exact commit already failed to install `FAILURE_CAP` times.
    RepeatedFailure { target: String, failures: u32 },
}

impl SkipReason {
    /// One line, plain words, for the log and the alert.
    pub fn describe(&self) -> String {
        match self {
            Self::DirtyTree => "the checkout has uncommitted changes — they are never touched, so the update is skipped".to_string(),
            Self::LocalCommits { ahead } => format!(
                "the checkout has {} local commit(s) not on origin — push or rebase them, then it resumes",
                ahead
            ),
            Self::AgentWorking { session } => format!(
                "an agent is working right now ({}) — deferred to the next daily run",
                session
            ),
            Self::RepeatedFailure { target, failures } => format!(
                "commit {} failed to install {} times — not retrying, this needs a human",
                target, failures
            ),
        }
    }

    /// True when the operator has to act before updates can resume. These are
    /// worth an alert; a one-day deferral is not.
    pub fn needs_human(&self) -> bool {
        !matches!(self, Self::AgentWorking { .. })
    }
}

impl Decision {
    pub fn describe(&self) -> String {
        match self {
            Self::Disabled => "auto-update is off (config: auto_update)".to_string(),
            Self::UpToDate => "already up to date".to_string(),
            Self::Apply { behind, target } => {
                format!("{} commit(s) behind — installing {}", behind, target)
            }
            Self::NotifyOnly { behind, target } => format!(
                "{} commit(s) behind ({}) — check-only policy, not installing",
                behind, target
            ),
            Self::Skip { reason } => reason.describe(),
        }
    }
}

/// The persisted memory of the daily cron: what it last saw, what it last
/// installed, and how often the current target has failed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutoUpdateState {
    pub last_check: Option<DateTime<Utc>>,
    pub last_applied: Option<DateTime<Utc>>,
    /// Commit installed by the last successful apply.
    pub last_applied_commit: Option<String>,
    /// Target commit the failures below refer to. A different target resets them.
    pub failing_commit: Option<String>,
    pub consecutive_failures: u32,
    /// Human-readable outcome of the last run, for `omega update --check`.
    pub last_outcome: Option<String>,
}

impl AutoUpdateState {
    pub fn path(state_dir: &Path) -> PathBuf {
        state_dir.join("auto-update.json")
    }

    /// Read the state, treating any error (missing, truncated by a crash mid-
    /// write, hand-edited) as "no history". A corrupt file must never be the
    /// thing that stops a machine from updating.
    pub fn load(state_dir: &Path) -> Self {
        std::fs::read_to_string(Self::path(state_dir))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, state_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(state_dir)?;
        let path = Self::path(state_dir);
        // Write-then-rename: a cron killed mid-write must not leave a truncated
        // file behind (load() tolerates it, but silently losing the failure
        // count would let a bad commit be retried forever).
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    /// How many times `target` has failed to install in a row (0 for a target
    /// we have never failed on).
    pub fn failures_for(&self, target: &str) -> u32 {
        match &self.failing_commit {
            Some(c) if c == target => self.consecutive_failures,
            _ => 0,
        }
    }

    /// Record a failed apply of `target`. A new target restarts the count.
    pub fn record_failure(&mut self, target: &str) {
        if self.failing_commit.as_deref() == Some(target) {
            self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        } else {
            self.failing_commit = Some(target.to_string());
            self.consecutive_failures = 1;
        }
    }

    /// Record a successful apply — clears the failure memory entirely.
    pub fn record_success(&mut self, target: &str, at: DateTime<Utc>) {
        self.last_applied = Some(at);
        self.last_applied_commit = Some(target.to_string());
        self.failing_commit = None;
        self.consecutive_failures = 0;
    }
}

/// The whole decision, in one pure function so every branch is testable
/// without a git checkout, a network, or a running agent.
///
/// `working_session` is `Some(name)` when an agent is mid-turn.
pub fn decide(
    policy: AutoUpdatePolicy,
    state: &CheckoutState,
    history: &AutoUpdateState,
    working_session: Option<&str>,
) -> Decision {
    if policy == AutoUpdatePolicy::Off {
        return Decision::Disabled;
    }

    // Nothing to pull is checked BEFORE the refusals: a dirty checkout that is
    // already current is not a problem worth alerting anyone about, and that is
    // the normal state of a developer's own machine.
    if state.behind == 0 {
        return Decision::UpToDate;
    }

    if policy == AutoUpdatePolicy::Check {
        return Decision::NotifyOnly {
            behind: state.behind,
            target: state.target.clone(),
        };
    }

    // Order matters: report the condition the user must FIX before the one they
    // merely have to wait out.
    if state.dirty {
        return Decision::Skip { reason: SkipReason::DirtyTree };
    }
    if state.ahead > 0 {
        return Decision::Skip {
            reason: SkipReason::LocalCommits { ahead: state.ahead },
        };
    }
    let failures = history.failures_for(&state.target);
    if failures >= FAILURE_CAP {
        return Decision::Skip {
            reason: SkipReason::RepeatedFailure {
                target: state.target.clone(),
                failures,
            },
        };
    }
    if let Some(session) = working_session {
        return Decision::Skip {
            reason: SkipReason::AgentWorking {
                session: session.to_string(),
            },
        };
    }

    Decision::Apply {
        behind: state.behind,
        target: state.target.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn behind(n: usize) -> CheckoutState {
        CheckoutState {
            behind: n,
            ahead: 0,
            dirty: false,
            target: "abc1234".to_string(),
        }
    }

    #[test]
    fn nothing_to_pull_is_the_quiet_daily_outcome() {
        let d = decide(
            AutoUpdatePolicy::Apply,
            &behind(0),
            &AutoUpdateState::default(),
            None,
        );
        assert_eq!(d, Decision::UpToDate);
    }

    #[test]
    fn an_available_update_is_installed() {
        let d = decide(
            AutoUpdatePolicy::Apply,
            &behind(3),
            &AutoUpdateState::default(),
            None,
        );
        assert_eq!(
            d,
            Decision::Apply {
                behind: 3,
                target: "abc1234".to_string()
            }
        );
    }

    #[test]
    fn check_policy_reports_but_never_installs() {
        let d = decide(
            AutoUpdatePolicy::Check,
            &behind(2),
            &AutoUpdateState::default(),
            None,
        );
        assert!(matches!(d, Decision::NotifyOnly { behind: 2, .. }));
    }

    #[test]
    fn off_policy_does_nothing_at_all() {
        // Not even the "you are behind" notice — off means off.
        let d = decide(
            AutoUpdatePolicy::Off,
            &behind(9),
            &AutoUpdateState::default(),
            None,
        );
        assert_eq!(d, Decision::Disabled);
    }

    #[test]
    fn local_work_is_never_clobbered() {
        let mut dirty = behind(2);
        dirty.dirty = true;
        assert!(matches!(
            decide(AutoUpdatePolicy::Apply, &dirty, &AutoUpdateState::default(), None),
            Decision::Skip { reason: SkipReason::DirtyTree }
        ));

        let mut ahead = behind(2);
        ahead.ahead = 4;
        assert!(matches!(
            decide(AutoUpdatePolicy::Apply, &ahead, &AutoUpdateState::default(), None),
            Decision::Skip { reason: SkipReason::LocalCommits { ahead: 4 } }
        ));
    }

    #[test]
    fn a_dirty_checkout_that_is_current_is_not_an_alert() {
        // A developer's own machine sits dirty all day. Only a dirty checkout
        // that BLOCKS a real update is worth saying anything about.
        let mut s = behind(0);
        s.dirty = true;
        assert_eq!(
            decide(AutoUpdatePolicy::Apply, &s, &AutoUpdateState::default(), None),
            Decision::UpToDate
        );
    }

    #[test]
    fn a_working_agent_defers_the_rebuild() {
        let d = decide(
            AutoUpdatePolicy::Apply,
            &behind(1),
            &AutoUpdateState::default(),
            Some("oracle-Camelia-1"),
        );
        match d {
            Decision::Skip { reason } => {
                assert!(!reason.needs_human(), "waiting a day is not an escalation");
                assert!(reason.describe().contains("oracle-Camelia-1"));
            }
            other => panic!("expected a deferral, got {:?}", other),
        }
    }

    #[test]
    fn the_same_bad_commit_is_not_retried_forever() {
        let mut history = AutoUpdateState::default();
        let state = behind(1);
        for attempt in 1..=FAILURE_CAP {
            assert!(
                matches!(
                    decide(AutoUpdatePolicy::Apply, &state, &history, None),
                    Decision::Apply { .. }
                ),
                "attempt {} must still be tried",
                attempt
            );
            history.record_failure(&state.target);
        }
        // Cap reached — it stops and asks for a human.
        match decide(AutoUpdatePolicy::Apply, &state, &history, None) {
            Decision::Skip { reason } => {
                assert!(reason.needs_human());
                assert!(matches!(reason, SkipReason::RepeatedFailure { failures, .. } if failures == FAILURE_CAP));
            }
            other => panic!("expected the cap to stop it, got {:?}", other),
        }
    }

    #[test]
    fn a_new_commit_on_the_remote_is_a_fresh_attempt() {
        let mut history = AutoUpdateState::default();
        for _ in 0..FAILURE_CAP {
            history.record_failure("abc1234");
        }
        let mut moved_on = behind(1);
        moved_on.target = "def5678".to_string();
        assert!(
            matches!(
                decide(AutoUpdatePolicy::Apply, &moved_on, &history, None),
                Decision::Apply { .. }
            ),
            "a different target must not inherit the old target's failures"
        );
        assert_eq!(history.failures_for("def5678"), 0);
    }

    #[test]
    fn success_clears_the_failure_memory() {
        let mut history = AutoUpdateState::default();
        history.record_failure("abc1234");
        history.record_failure("abc1234");
        assert_eq!(history.failures_for("abc1234"), 2);
        history.record_success("abc1234", Utc::now());
        assert_eq!(history.failures_for("abc1234"), 0);
        assert_eq!(history.last_applied_commit.as_deref(), Some("abc1234"));
    }

    #[test]
    fn state_round_trips_through_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let mut s = AutoUpdateState::default();
        s.record_failure("abc1234");
        s.last_outcome = Some("installed".to_string());
        s.save(tmp.path()).unwrap();

        let back = AutoUpdateState::load(tmp.path());
        assert_eq!(back.failures_for("abc1234"), 1);
        assert_eq!(back.last_outcome.as_deref(), Some("installed"));
    }

    #[test]
    fn a_corrupt_state_file_never_blocks_an_update() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(AutoUpdateState::path(tmp.path()), "{ truncated").unwrap();
        let back = AutoUpdateState::load(tmp.path());
        assert_eq!(back.consecutive_failures, 0, "corrupt reads as no history");
        assert!(matches!(
            decide(AutoUpdatePolicy::Apply, &behind(1), &back, None),
            Decision::Apply { .. }
        ));
    }

    #[test]
    fn policy_parses_what_users_actually_type() {
        assert_eq!(AutoUpdatePolicy::parse("off"), AutoUpdatePolicy::Off);
        assert_eq!(AutoUpdatePolicy::parse("  CHECK "), AutoUpdatePolicy::Check);
        assert_eq!(AutoUpdatePolicy::parse("apply"), AutoUpdatePolicy::Apply);
        // A typo must not silently disable updates.
        assert_eq!(AutoUpdatePolicy::parse("aply"), AutoUpdatePolicy::Apply);
    }
}
