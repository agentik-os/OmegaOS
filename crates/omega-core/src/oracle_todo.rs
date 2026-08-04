//! Oracle todo ledger — the durable half of `oracle-<key>.progress.json`.
//!
//! [`crate::progress::ProgressInfo`] is the READ projection of that file: it
//! answers "how far along is this oracle" for the TUI bars, `omega list` and
//! patrol. It deliberately never modelled the task LIST, so every producer that
//! wanted to change one task re-derived the whole document by hand — the
//! Telegram bot with its own JSON merge, `cmd_progress` with another, the L4
//! close-gate in `cmd_done` re-parsing `tasks[].s` as raw `serde_json::Value`
//! because there was no type to parse it into. Three hand-rolled readings of
//! one schema is how the two sides drift, and the operator watched exactly that
//! drift: an oracle that re-stated its plan after a compaction and silently
//! reset finished tasks back to `todo`, another that claimed done_clean while a
//! worker was still running, and a third that reported a task `done` that a
//! later turn had quietly walked back.
//!
//! This module is the WRITE model those producers were missing, plus the
//! closure semantics as pure functions the CLI can feed live sessions into.
//! What it deliberately does NOT do: invent a new file. It reads and writes the
//! same `oracle-<key>.progress.json`, with the same `{tasks:[{t,s}], done,
//! total, ts}` keys, resolved by the same path rule as `ProgressInfo::read`
//! (session name minus ONE leading `oracle-`). Every key it does not itself
//! own — the bot's `chat` / `thread` / `msgId` / `bot` / `project` / `oracle` /
//! `mission`, and anything a future producer adds — is carried through
//! verbatim, because this file has four writers and a write that clobbers the
//! bot's `msgId` costs the operator the live progress card.
//!
//! The invariants, each one paid for by an observed failure:
//!
//! 1. RE-STATING A PLAN NEVER RESETS FINISHED WORK. After a compaction an
//!    oracle re-sends its plan from a fresh context. `set_plan` matches on the
//!    title and keeps the status and evidence it already had.
//! 2. `done` IS A ONE-WAY DOOR. A transition out of `Done` back into an
//!    unfinished state is rejected with a typed error. That is the silent
//!    regression: a plan that reads 5/5 in one turn and 3/5 in the next, with
//!    nothing in between explaining why.
//! 3. AT MOST ONE ITEM IS `Doing`. A ledger with three tasks in progress is a
//!    ledger nobody is reading, and the resume-after-compaction read-back has
//!    no single answer to "what was I doing".
//! 4. A CLEAN CLOSE IS REFUSED WHILE WORKERS RUN, and an honest non-clean
//!    status never is. `pending` / `failed` / `blocked` are exactly what a
//!    stuck oracle must be able to emit, and refusing those would leave it with
//!    no legal way to report at all.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt;
use std::path::{Path, PathBuf};

use crate::done::{DoneStatus, OracleDoneSignal};
use crate::oracle_lifecycle::LiveWorkers;

// ---------------------------------------------------------------------------
// TodoStatus — the on-disk `s` value, as a validated state machine
// ---------------------------------------------------------------------------

/// Status of one plan item. The serde names are the EXACT lowercase strings
/// already on disk (`"todo" | "doing" | "done" | "fail"`) — the L4 gate in
/// `cmd_done` matches those literals, the Telegram card renders from them, and
/// renaming any of them silently breaks both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TodoStatus {
    Todo,
    Doing,
    Done,
    Fail,
}

impl TodoStatus {
    /// The on-disk string. Kept next to the serde attribute so a rename of one
    /// without the other fails a test rather than a production read.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::Doing => "doing",
            Self::Done => "done",
            Self::Fail => "fail",
        }
    }

    /// The marker the Telegram progress card already uses per status.
    pub fn marker(self) -> char {
        match self {
            Self::Todo => '☐',
            Self::Doing => '▸',
            Self::Done => '✓',
            Self::Fail => '✗',
        }
    }

    /// An item that will not change on its own. Mirrors
    /// [`crate::mission::TaskAttemptState::is_terminal`] in spirit: `Fail` is
    /// terminal for the CURRENT attempt, which is why a bounded retry out of it
    /// is still legal below.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Done | Self::Fail)
    }

    /// Legal moves, in the style of [`crate::mission::TaskAttemptState`].
    ///
    /// The permissive edges are the ones a real mission needs: `Todo -> Done`
    /// for an item that turned out to be already satisfied (forcing a
    /// ceremonial `Doing` in between would be bookkeeping theatre), `Doing ->
    /// Todo` for an honest de-escalation when an oracle picks up a different
    /// item first, and `Fail -> Doing` / `Fail -> Todo` for the bounded retry
    /// R-LOOP mandates.
    ///
    /// The strict edge is the whole point: NOTHING leaves `Done` for an
    /// unfinished state. An oracle that re-states its plan after a compaction,
    /// or a worker that re-reports an item it already closed, must not be able
    /// to walk a finished task backwards without anyone noticing. `Done ->
    /// Fail` IS allowed, and is not a hole in that rule: it lands in a terminal
    /// state, not an unfinished one, and it is the honest direction — an item
    /// whose verification later collapsed has to have somewhere to go, or the
    /// only way to report it is to lie about it.
    ///
    /// The identity move is allowed on every status, and that is deliberate
    /// rather than an oversight: `upsert` is the only way to attach evidence to
    /// an item, so re-asserting a status to record its proof has to be legal.
    /// It also makes the whole API idempotent, which matters because a
    /// resumed oracle replays its own last few statements routinely.
    pub fn can_transition_to(self, next: Self) -> bool {
        use TodoStatus::*;
        if self == next {
            return true;
        }
        matches!(
            (self, next),
            (Todo, Doing | Done)
                | (Doing, Done | Fail | Todo)
                | (Fail, Doing | Todo)
                | (Done, Fail)
        )
    }

    /// Validated move, mirroring `TaskAttemptState::transition`. The error
    /// carries no item title (the enum does not know one); `OracleTodo::upsert`
    /// produces the same error WITH the title, which is what an operator
    /// actually needs to read.
    pub fn transition(self, next: Self) -> Result<Self, InvalidTodoTransition> {
        if self.can_transition_to(next) {
            Ok(next)
        } else {
            Err(InvalidTodoTransition {
                item: None,
                from: self,
                to: next,
            })
        }
    }
}

/// A rejected status change, shaped after [`crate::mission::InvalidTransition`]
/// (whose constructor is private to that module, so this is a sibling type
/// rather than a reuse). It names the item because the operator-facing message
/// "done -> todo rejected" is useless without knowing WHICH task tried it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidTodoTransition {
    pub item: Option<String>,
    pub from: TodoStatus,
    pub to: TodoStatus,
}

impl fmt::Display for InvalidTodoTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.item {
            Some(item) => write!(
                f,
                "invalid todo transition for {:?}: {} -> {}",
                item,
                self.from.as_str(),
                self.to.as_str()
            ),
            None => write!(
                f,
                "invalid todo transition: {} -> {}",
                self.from.as_str(),
                self.to.as_str()
            ),
        }
    }
}

impl std::error::Error for InvalidTodoTransition {}

// ---------------------------------------------------------------------------
// TodoItem — one `{t, s}` entry, plus the fields this module adds
// ---------------------------------------------------------------------------

/// One plan item. `t` and `s` are the existing on-disk keys and keep their
/// exact names; `evidence` and `updated_at` are new and OPTIONAL, so a file
/// written by the current Telegram bot still parses here and a file written
/// here still parses everywhere else.
///
/// `extra` catches per-task keys this module does not know about. Without it a
/// future producer's field would be silently dropped the first time an oracle
/// touched its own plan, which is the class of data loss that is impossible to
/// notice until the field is needed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TodoItem {
    #[serde(rename = "t")]
    pub title: String,
    #[serde(rename = "s")]
    pub status: TodoStatus,
    /// What proves this item is in the status it claims: a file:line, a command
    /// and its exit code, a commit. Optional because the existing producers
    /// write none, and R-CITE is a quality bar rather than a parse requirement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl TodoItem {
    /// A fresh, untouched item.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            status: TodoStatus::Todo,
            evidence: None,
            updated_at: None,
            extra: Map::new(),
        }
    }

    /// Titles are matched case-insensitively everywhere in this module. An
    /// oracle re-stating its plan from a fresh context after a compaction
    /// rarely reproduces its own capitalisation, and a case-sensitive match
    /// would silently create a SECOND item and reset the first one's progress —
    /// invariant 1 defeated by a capital letter.
    fn matches(&self, title: &str) -> bool {
        self.title.eq_ignore_ascii_case(title)
    }
}

// ---------------------------------------------------------------------------
// OracleTodo — the whole plan, round-tripped through the shared file
// ---------------------------------------------------------------------------

/// The plan an oracle is working through, backed by
/// `oracle-<key>.progress.json`.
///
/// `done` / `total` / `ts` are declared as real fields rather than left to
/// `extra` on purpose: they are DERIVED, recomputed from `tasks` on every save,
/// and if they lived in the flattened map they would be written twice into the
/// same JSON object. Declaring them keeps `extra` for genuinely foreign keys
/// only.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OracleTodo {
    #[serde(default)]
    pub tasks: Vec<TodoItem>,
    /// Legacy derived counters. Read back by the TUI, `omega list`, patrol and
    /// the Telegram card; recomputed by [`OracleTodo::save`] so those consumers
    /// never see a count that disagrees with the task list they sit next to.
    #[serde(default)]
    done: u32,
    #[serde(default)]
    total: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ts: Option<DateTime<Utc>>,
    /// Every top-level key this module does not own: the bot's `chat`,
    /// `thread`, `msgId`, `bot`, `project`, `oracle`, `mission`, and whatever
    /// the next producer adds. Written back verbatim.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
    /// The session this plan belongs to. Never serialized — it is the lookup
    /// key, not file content, and the file already carries the bot's `oracle`
    /// field for that purpose.
    #[serde(skip)]
    session: String,
}

impl OracleTodo {
    /// An empty plan for `session`. This is what [`OracleTodo::load`] returns
    /// when no file exists yet.
    pub fn new(session: &str) -> Self {
        Self {
            session: session.to_string(),
            ..Default::default()
        }
    }

    /// The session name this plan was loaded for.
    pub fn session(&self) -> &str {
        &self.session
    }

    /// Same path rule as [`crate::progress::ProgressInfo::read`] and
    /// `cmd_progress`: the file key is the session name minus ONE leading
    /// `oracle-`, so a worker session keeps its full name inside the key and
    /// resolves `oracle-<worker>.progress.json`. Re-deriving this rule anywhere
    /// else is how the writer and the reader end up one prefix apart, which is
    /// the bug `ProgressInfo::read` documents at length.
    pub fn path(state_dir: &Path, session: &str) -> PathBuf {
        let key = session.strip_prefix("oracle-").unwrap_or(session);
        state_dir.join(format!("oracle-{}.progress.json", key))
    }

    /// Load the plan. A MISSING file is an empty plan, never an error: that is
    /// the post-compaction resume path, where an oracle asks what it was doing
    /// before it has ever written anything. A file that exists but does not
    /// parse IS an error, because silently starting from empty there would let
    /// the next save overwrite a real plan with nothing.
    pub fn load(state_dir: &Path, session: &str) -> Result<Self> {
        let path = Self::path(state_dir, session);
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::new(session)),
            // Any OTHER io error (a permission problem, a directory where the
            // file should be) is real and must surface: it is not the same
            // thing as "this oracle has not written a plan yet".
            Err(e) => {
                return Err(
                    anyhow::Error::from(e).context(format!("cannot read {}", path.display()))
                )
            }
        };
        let mut todo: Self = serde_json::from_str(&content).map_err(|e| {
            anyhow::anyhow!("{} is not a readable progress file: {}", path.display(), e)
        })?;
        todo.session = session.to_string();
        Ok(todo)
    }

    /// Replace the task list with `titles`, in the given order.
    ///
    /// INVARIANT 1 lives here. A title that already exists KEEPS its status,
    /// its evidence, its timestamp and its unknown keys; only genuinely new
    /// titles start at `Todo`. An oracle that re-states its plan after a
    /// compaction is describing the same mission, not restarting it, and the
    /// operator watched a re-statement reset four finished tasks to `todo` and
    /// then report 0/5 on a mission that was nearly over.
    ///
    /// Titles dropped from the new plan are dropped from the ledger: a plan
    /// that no longer contains an item is the oracle saying that item is not
    /// part of the mission, and keeping a ghost `done` item would inflate the
    /// counts a legacy consumer reads.
    pub fn set_plan<I, S>(&mut self, titles: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let previous = std::mem::take(&mut self.tasks);
        for title in titles {
            let title = title.as_ref();
            match previous.iter().find(|p| p.matches(title)) {
                // Preserve everything the previous entry knew, but adopt the
                // NEW spelling of the title: the oracle just told us how it
                // refers to this item now.
                Some(prev) => self.tasks.push(TodoItem {
                    title: title.to_string(),
                    ..prev.clone()
                }),
                None => self.tasks.push(TodoItem::new(title)),
            }
        }
        self.enforce_single_doing();
    }

    /// Move `title` to `status`, attaching `evidence` when supplied.
    ///
    /// An UNKNOWN title is created at the requested status. Creation is not a
    /// transition — there is no prior state to leave — so this is the one path
    /// that can mint an item directly as `Done` or `Fail`. That matters for a
    /// worker reporting an item its oracle never planned: refusing it would
    /// mean the honest outcome of real work has nowhere to go.
    ///
    /// An EXISTING title is validated by [`TodoStatus::can_transition_to`] and
    /// the plan is left untouched on rejection.
    pub fn upsert(
        &mut self,
        title: &str,
        status: TodoStatus,
        evidence: Option<&str>,
    ) -> Result<(), InvalidTodoTransition> {
        match self.tasks.iter().position(|t| t.matches(title)) {
            Some(idx) => {
                let from = self.tasks[idx].status;
                if !from.can_transition_to(status) {
                    return Err(InvalidTodoTransition {
                        item: Some(self.tasks[idx].title.clone()),
                        from,
                        to: status,
                    });
                }
                let item = &mut self.tasks[idx];
                item.status = status;
                // Evidence is additive: `None` means "no new proof", never
                // "erase the proof I already had". A retry that re-asserts a
                // status without re-citing its artifact must not blank the
                // citation the previous attempt earned (R-CITE).
                if let Some(ev) = evidence {
                    item.evidence = Some(ev.to_string());
                }
                item.updated_at = Some(Utc::now());
            }
            None => {
                self.tasks.push(TodoItem {
                    title: title.to_string(),
                    status,
                    evidence: evidence.map(|e| e.to_string()),
                    updated_at: Some(Utc::now()),
                    extra: Map::new(),
                });
            }
        }
        if status == TodoStatus::Doing {
            // Demote by INDEX, not by timestamp. `upsert` knows exactly which
            // item the caller just moved, so it must not re-derive that from
            // `updated_at`: two items stamped inside the same clock tick would
            // tie, and the tie-break would silently demote the item the caller
            // had just started. `enforce_single_doing` keeps the timestamp rule
            // for the case where nobody knows the intent (a file another
            // producer wrote with two `doing` entries).
            let moved = self
                .tasks
                .iter()
                .position(|t| t.matches(title))
                .expect("upsert just inserted or updated this title");
            self.demote_other_doing(moved);
        }
        Ok(())
    }

    /// INVARIANT 3: collapse the plan to at most one `Doing` item.
    ///
    /// This is the REPAIR path, for a plan whose `doing` items were not set
    /// through `upsert` — a file written by another producer, or one hand-edited
    /// during an incident. The winner is the most recently updated `Doing` item,
    /// falling back to the first in plan order when no timestamp distinguishes
    /// them, because plan order is the only signal left at that point.
    pub fn enforce_single_doing(&mut self) {
        let winner = self
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.status == TodoStatus::Doing)
            .max_by_key(|(idx, t)| (t.updated_at, std::cmp::Reverse(*idx)))
            .map(|(idx, _)| idx);
        if let Some(winner) = winner {
            self.demote_other_doing(winner);
        }
    }

    /// Send every `Doing` item except `keep` back to `Todo` — never to a
    /// terminal status, because nothing was finished or failed by being set
    /// aside.
    fn demote_other_doing(&mut self, keep: usize) {
        for (idx, task) in self.tasks.iter_mut().enumerate() {
            if idx != keep && task.status == TodoStatus::Doing {
                task.status = TodoStatus::Todo;
            }
        }
    }

    /// The item currently in progress, if any. This is the answer to "what was
    /// I doing" after a compaction, and invariant 3 is what makes it a single
    /// answer rather than a list.
    pub fn current(&self) -> Option<&TodoItem> {
        self.tasks.iter().find(|t| t.status == TodoStatus::Doing)
    }

    /// `(done, total)` computed from the tasks themselves, which is what
    /// [`OracleTodo::save`] writes into the legacy `done` / `total` keys.
    pub fn counts(&self) -> (u32, u32) {
        let done = self
            .tasks
            .iter()
            .filter(|t| t.status == TodoStatus::Done)
            .count() as u32;
        (done, self.tasks.len() as u32)
    }

    pub fn has_failures(&self) -> bool {
        self.tasks.iter().any(|t| t.status == TodoStatus::Fail)
    }

    /// Titles of the failed items — what a refusal has to name.
    pub fn failed(&self) -> Vec<&TodoItem> {
        self.tasks
            .iter()
            .filter(|t| t.status == TodoStatus::Fail)
            .collect()
    }

    /// Everything still owed: `todo` plus the one `doing`.
    pub fn unfinished(&self) -> Vec<&TodoItem> {
        self.tasks
            .iter()
            .filter(|t| !t.status.is_terminal())
            .collect()
    }

    /// L4: done means 100%, and a failed item is not a finished one. An EMPTY
    /// plan is never complete — `total == 0` is the absence of a plan, and the
    /// close-gate in `cmd_done` already treats it as un-acceptable rather than
    /// as vacuously true.
    pub fn is_complete(&self) -> bool {
        let (done, total) = self.counts();
        total > 0 && done == total && !self.has_failures()
    }

    /// Persist the plan, recomputing `done` / `total` / `ts` so every existing
    /// consumer keeps reading numbers that agree with the task list.
    ///
    /// Two properties matter more than the write itself. It MERGES: the file is
    /// re-read immediately before writing, so a key the Telegram bot added
    /// since this plan was loaded survives (four processes write this file; a
    /// blind serialize would drop the bot's `msgId` and kill the live card).
    /// And it is ATOMIC: tmp file plus rename, the idiom `cmd_progress`,
    /// `done.rs` and `oracle_lifecycle.rs` already use, so a reader never
    /// catches a half-written plan.
    pub fn save(&self, state_dir: &Path, session: &str) -> Result<()> {
        std::fs::create_dir_all(state_dir)?;
        let path = Self::path(state_dir, session);
        let key = session.strip_prefix("oracle-").unwrap_or(session);
        let tmp = state_dir.join(format!(".oracle-{}.progress.json.tmp", key));

        // Start from what is on disk RIGHT NOW so a concurrent producer's keys
        // are preserved, then fill in anything we remember that disk no longer
        // has. Disk wins on a conflict: it is the fresher of the two.
        let mut root: Map<String, Value> = std::fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str::<Value>(&c).ok())
            .and_then(|v| match v {
                Value::Object(m) => Some(m),
                _ => None,
            })
            .unwrap_or_default();
        for (k, v) in &self.extra {
            root.entry(k.clone()).or_insert_with(|| v.clone());
        }
        // The keys this module owns are authoritative and overwrite whatever
        // was there.
        let (done, total) = self.counts();
        root.insert("tasks".to_string(), serde_json::to_value(&self.tasks)?);
        root.insert("done".to_string(), Value::from(done));
        root.insert("total".to_string(), Value::from(total));
        root.insert("ts".to_string(), serde_json::to_value(Utc::now())?);

        let content = serde_json::to_string_pretty(&Value::Object(root))?;
        std::fs::write(&tmp, &content)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    /// The checklist an oracle reads back to itself after a compaction, with
    /// the same markers the Telegram progress card uses. Stable by
    /// construction: one line per item, in plan order, no timestamps and no
    /// counts inside the lines, so two renders of an unchanged plan compare
    /// equal.
    pub fn render_checklist(&self) -> String {
        if self.tasks.is_empty() {
            return "(no plan)".to_string();
        }
        let mut out = String::with_capacity(self.tasks.len() * 40);
        for task in &self.tasks {
            out.push(task.status.marker());
            out.push(' ');
            out.push_str(&task.title);
            if let Some(ev) = &task.evidence {
                out.push_str(" (");
                out.push_str(ev);
                out.push(')');
            }
            out.push('\n');
        }
        let (done, total) = self.counts();
        out.push_str(&format!("{}/{}", done, total));
        out
    }
}

// ---------------------------------------------------------------------------
// Closure semantics — pure functions, no session IO
// ---------------------------------------------------------------------------

/// Why a close was refused. Every variant NAMES what is in the way, because a
/// refusal an operator cannot act on is just an error message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClosureRefusal {
    /// Workers of this oracle are still running. The zombie-worker guard: an
    /// oracle that closes over live workers leaves sessions nobody will ever
    /// reap, because the reaper keys off the oracle's signal.
    WorkersRunning(Vec<String>),
    /// L4: the plan is not 100%.
    PlanIncomplete {
        done: u32,
        total: u32,
        unfinished: Vec<String>,
    },
    /// The plan carries failed items. `done_clean` over a failed task is the
    /// dishonest-signal case; the honest move is `failed` (or a retry).
    PlanFailed(Vec<String>),
    /// No plan at all. `total == 0` proves no work was tracked, and the CLI
    /// gate already fails closed on it rather than accepting a vacuous 0/0.
    NoPlan,
}

impl fmt::Display for ClosureRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkersRunning(w) => write!(
                f,
                "done_clean refused: {} worker(s) still running: {}",
                w.len(),
                w.join(", ")
            ),
            Self::PlanIncomplete {
                done,
                total,
                unfinished,
            } => write!(
                f,
                "done_clean refused: plan {}/{}, still owed: {}",
                done,
                total,
                unfinished.join(", ")
            ),
            Self::PlanFailed(items) => write!(
                f,
                "done_clean refused: {} failed item(s): {}",
                items.len(),
                items.join(", ")
            ),
            Self::NoPlan => write!(
                f,
                "done_clean refused: no mission plan recorded, acceptance impossible"
            ),
        }
    }
}

impl std::error::Error for ClosureRefusal {}

/// What the caller should actually DO to close. Pure data: this module decides,
/// the CLI executes (kills the sessions, releases the claims).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClosurePlan {
    /// Finished worker sessions that close together with their oracle. Empty
    /// when nothing closes.
    pub cascade_workers: Vec<String>,
    /// Scope claims to release: the oracle plus every cascaded worker.
    pub scopes_to_release: Vec<String>,
    /// The oracle was ALREADY closed. Nothing to do, and nothing to re-kill.
    pub already_closed: bool,
}

/// Decide whether this oracle may close, and with what fallout.
///
/// Fed by the CLI from live data (`live_workers_of_oracle` over the real
/// session list) but itself pure, so the decision is testable without a
/// session manager, a daemon or a spawned process.
///
/// The order of the checks is deliberate. Running workers are refused FIRST,
/// before any plan arithmetic, because that is a fact about the live system
/// with an obvious operator action ("wait, or kill them"), whereas an
/// incomplete plan is a bookkeeping fact. The `cmd_done` gate resolves the same
/// two conditions in the opposite order only because it DOWNGRADES on the plan
/// rather than refusing; both agree on what is and is not a legal `done_clean`.
pub fn evaluate_closure(
    todo: &OracleTodo,
    live: &LiveWorkers,
    existing_signal: Option<&OracleDoneSignal>,
    requested: DoneStatus,
) -> Result<ClosurePlan, ClosureRefusal> {
    // IDEMPOTENCE. A second close of an already-closed oracle is a no-op, not
    // an error and not a second cascade. `omega done` is re-run routinely (a
    // resumed session that lost its context, patrol upgrading a gate_pending
    // signal), and a cascade computed twice would kill sessions that a LATER
    // dispatch had already spawned under recycled worker names.
    if existing_signal.is_some_and(|sig| sig.is_closeable() && signal_is_for(sig, todo.session())) {
        return Ok(ClosurePlan {
            already_closed: true,
            ..Default::default()
        });
    }

    // An honest non-clean status is never refused. `pending` / `failed` /
    // `blocked` are precisely what a blocked oracle must be able to emit, and
    // gating them behind the same conditions as `done_clean` would leave a
    // stuck mission with no legal way to report at all. Nothing closes on such
    // a signal either: the oracle stays alive, so its workers keep their
    // sessions and their scope claims.
    if requested != DoneStatus::DoneClean {
        return Ok(ClosurePlan::default());
    }

    if !live.running.is_empty() {
        return Err(ClosureRefusal::WorkersRunning(live.running.clone()));
    }

    let (done, total) = todo.counts();
    if total == 0 {
        return Err(ClosureRefusal::NoPlan);
    }
    if todo.has_failures() {
        return Err(ClosureRefusal::PlanFailed(
            todo.failed().iter().map(|t| t.title.clone()).collect(),
        ));
    }
    if done != total {
        return Err(ClosureRefusal::PlanIncomplete {
            done,
            total,
            unfinished: todo.unfinished().iter().map(|t| t.title.clone()).collect(),
        });
    }

    let cascade = live.terminal.clone();
    let mut scopes = Vec::with_capacity(cascade.len() + 1);
    scopes.push(todo.session().to_string());
    scopes.extend(cascade.iter().cloned());
    Ok(ClosurePlan {
        cascade_workers: cascade,
        scopes_to_release: scopes,
        already_closed: false,
    })
}

/// True when `sig` belongs to `session`. The signal stores the oracle KEY
/// (`OracleDoneSignal::new` is called with the name minus its `oracle-`
/// prefix) while callers hold the full session name, so both sides are
/// normalized the same way before comparing. Without this an unrelated
/// oracle's signal handed in by mistake would report `already_closed` for a
/// mission that never closed.
fn signal_is_for(sig: &OracleDoneSignal, session: &str) -> bool {
    let key = |s: &str| s.strip_prefix("oracle-").unwrap_or(s).to_string();
    key(&sig.oracle) == key(session)
}

/// The honest status for a requested one, given the plan.
///
/// Downgrades only, never upgrades: a `done_clean` over an incomplete plan
/// becomes `Pending`, over a failed plan becomes `Failed`, and every non-clean
/// status is returned untouched. This is the "signal honnête" rule — the report
/// the operator reads must not say a mission finished when its own ledger says
/// it did not, and no arithmetic here may ever turn a worker's honest `failed`
/// into something nicer.
pub fn honest_status(requested: DoneStatus, todo: &OracleTodo) -> DoneStatus {
    if requested != DoneStatus::DoneClean {
        return requested;
    }
    if todo.has_failures() {
        return DoneStatus::Failed;
    }
    if !todo.is_complete() {
        return DoneStatus::Pending;
    }
    DoneStatus::DoneClean
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::ProgressInfo;

    /// A real merged on-disk file: the Telegram bot's card fields alongside
    /// `cmd_progress`'s tasks/done/total/ts. Same shape as the fixture
    /// `progress::tests` uses, because it is the same file.
    const REAL_MERGED: &str = r#"{
        "chat": 8626440209,
        "thread": 63,
        "msgId": 1234,
        "bot": "agent",
        "project": "academy",
        "oracle": "oracle-academy-2",
        "mission": "fix the login flow",
        "tasks": [{"t": "analyze", "s": "done"}, {"t": "fix", "s": "todo"}],
        "done": 1,
        "total": 2,
        "ts": "2026-06-10T08:00:00Z"
    }"#;

    fn read_json(path: &Path) -> Map<String, Value> {
        let content = std::fs::read_to_string(path).expect("file must exist");
        match serde_json::from_str::<Value>(&content).expect("valid json") {
            Value::Object(m) => m,
            other => panic!("expected an object, got {other}"),
        }
    }

    fn oracle_signal(oracle: &str, status: DoneStatus) -> OracleDoneSignal {
        OracleDoneSignal::new(oracle, "academy", status, "mission")
    }

    /// The whole backward-compatibility contract in one test: every key this
    /// module does not own comes back byte-for-byte in value, and a legacy
    /// per-task key survives too.
    #[test]
    fn every_foreign_key_survives_a_load_save_round_trip() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        let path = dir.join("oracle-academy-2.progress.json");
        // Add an unknown PER-TASK key on top of the real fixture: a future
        // producer's field must survive an oracle touching its own plan.
        let mut seed: Value = serde_json::from_str(REAL_MERGED).unwrap();
        seed["tasks"][0]["owner"] = Value::from("worker-1");
        std::fs::write(&path, serde_json::to_string_pretty(&seed).unwrap()).unwrap();

        let before = read_json(&path);
        let todo = OracleTodo::load(dir, "oracle-academy-2").unwrap();
        todo.save(dir, "oracle-academy-2").unwrap();
        let after = read_json(&path);

        for key in [
            "chat", "thread", "msgId", "bot", "project", "oracle", "mission",
        ] {
            assert_eq!(
                before.get(key),
                after.get(key),
                "foreign key {key} must be preserved verbatim"
            );
        }
        assert_eq!(
            after["tasks"][0]["owner"],
            Value::from("worker-1"),
            "unknown per-task keys must survive too"
        );
        // And the schema this module DOES own is still the legacy one.
        assert_eq!(after["tasks"][0]["t"], Value::from("analyze"));
        assert_eq!(after["tasks"][0]["s"], Value::from("done"));
        assert_eq!(after["done"], Value::from(1));
        assert_eq!(after["total"], Value::from(2));
    }

    /// The on-disk `s` vocabulary is a CONTRACT with three consumers that do
    /// not compile against this crate: the L4 gate in `cmd_done` matches the
    /// literals `"fail"` / `"todo"` / `"doing"`, the Telegram bot renders from
    /// them, and the shell alert greps them. A rename here would break all
    /// three silently, so the strings are pinned in both directions, and
    /// `as_str` is pinned against serde so the two cannot drift apart.
    #[test]
    fn the_on_disk_status_vocabulary_is_pinned() {
        for (status, on_disk) in [
            (TodoStatus::Todo, "todo"),
            (TodoStatus::Doing, "doing"),
            (TodoStatus::Done, "done"),
            (TodoStatus::Fail, "fail"),
        ] {
            assert_eq!(serde_json::to_value(status).unwrap(), Value::from(on_disk));
            assert_eq!(
                serde_json::from_value::<TodoStatus>(Value::from(on_disk)).unwrap(),
                status
            );
            assert_eq!(
                status.as_str(),
                on_disk,
                "as_str must not drift from the serde attribute"
            );
        }
        // And the task keys themselves are `t` / `s`, not the Rust names.
        let item = TodoItem::new("x");
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["t"], Value::from("x"));
        assert_eq!(json["s"], Value::from("todo"));
        assert!(
            json.get("title").is_none() && json.get("status").is_none(),
            "the Rust field names must never reach the file"
        );
        // The two NEW fields are omitted when absent, so a file written here is
        // byte-shaped like one written by the existing producers.
        assert!(json.get("evidence").is_none() && json.get("updated_at").is_none());
    }

    /// `ts` is read by `ProgressInfo` (chrono) and sits in a document the bot's
    /// JS parses. It must stay an RFC3339 UTC string, not a serde integer or a
    /// chrono struct.
    #[test]
    fn the_timestamp_stays_an_rfc3339_utc_string() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        let mut todo = OracleTodo::new("oracle-academy");
        todo.set_plan(["a"]);
        todo.save(dir, "oracle-academy").unwrap();

        let root = read_json(&OracleTodo::path(dir, "oracle-academy"));
        let ts = root["ts"].as_str().expect("ts must be a string");
        assert!(ts.ends_with('Z'), "expected a UTC RFC3339 stamp, got {ts}");
        DateTime::parse_from_rfc3339(ts).expect("must parse as RFC3339");
    }

    /// The post-compaction resume path: an oracle asks what it was doing before
    /// anything was ever written. That must be an empty plan, not an error.
    #[test]
    fn a_missing_progress_file_loads_an_empty_plan() {
        let tmp = tempfile::TempDir::new().unwrap();
        let todo = OracleTodo::load(tmp.path(), "oracle-brand-new").expect("must not error");
        assert!(todo.tasks.is_empty());
        assert_eq!(todo.counts(), (0, 0));
        assert!(todo.current().is_none());
        assert_eq!(todo.render_checklist(), "(no plan)");
        // A corrupt file, by contrast, IS an error: starting from empty there
        // would let the next save overwrite a real plan with nothing.
        std::fs::write(
            OracleTodo::path(tmp.path(), "oracle-corrupt"),
            "{not json at all",
        )
        .unwrap();
        assert!(OracleTodo::load(tmp.path(), "oracle-corrupt").is_err());
    }

    /// INVARIANT 1, the single most important resume property.
    #[test]
    fn restating_the_plan_preserves_finished_work() {
        let mut todo = OracleTodo::new("oracle-academy");
        todo.set_plan(["analyze", "fix", "verify"]);
        todo.upsert("analyze", TodoStatus::Done, Some("src/a.rs:12"))
            .unwrap();
        todo.upsert("fix", TodoStatus::Doing, None).unwrap();

        // Same mission, re-stated from a fresh context after a compaction:
        // different capitalisation, one extra item, one dropped.
        todo.set_plan(["Analyze", "fix", "verify", "report"]);

        let analyze = &todo.tasks[0];
        assert_eq!(
            analyze.status,
            TodoStatus::Done,
            "finished work must survive"
        );
        assert_eq!(analyze.evidence.as_deref(), Some("src/a.rs:12"));
        assert_eq!(analyze.title, "Analyze", "the new spelling is adopted");
        assert_eq!(todo.tasks[1].status, TodoStatus::Doing);
        assert_eq!(
            todo.tasks[3].status,
            TodoStatus::Todo,
            "new item starts todo"
        );
        assert_eq!(todo.counts(), (1, 4));
    }

    /// INVARIANT 3.
    #[test]
    fn moving_a_second_item_to_doing_demotes_the_first() {
        let mut todo = OracleTodo::new("oracle-academy");
        todo.set_plan(["a", "b", "c"]);
        todo.upsert("a", TodoStatus::Doing, None).unwrap();
        assert_eq!(todo.current().map(|t| t.title.as_str()), Some("a"));

        todo.upsert("b", TodoStatus::Doing, None).unwrap();
        assert_eq!(
            todo.tasks
                .iter()
                .filter(|t| t.status == TodoStatus::Doing)
                .count(),
            1,
            "at most one item may be in progress"
        );
        assert_eq!(todo.current().map(|t| t.title.as_str()), Some("b"));
        assert_eq!(
            todo.tasks[0].status,
            TodoStatus::Todo,
            "the demoted item goes back to todo, never to a terminal status"
        );
    }

    /// INVARIANT 2, and the bounded retry that must still be legal beside it.
    #[test]
    fn done_cannot_regress_but_a_failure_can_be_retried() {
        let mut todo = OracleTodo::new("oracle-academy");
        todo.set_plan(["a", "b"]);
        todo.upsert("a", TodoStatus::Done, None).unwrap();

        let err = todo
            .upsert("a", TodoStatus::Todo, None)
            .expect_err("done -> todo must be rejected");
        assert_eq!(err.from, TodoStatus::Done);
        assert_eq!(err.to, TodoStatus::Todo);
        assert_eq!(err.item.as_deref(), Some("a"));
        assert!(err.to_string().contains("done -> todo"));
        assert_eq!(
            todo.tasks[0].status,
            TodoStatus::Done,
            "a rejected transition must leave the plan untouched"
        );
        assert!(todo.upsert("a", TodoStatus::Doing, None).is_err());

        // doing -> fail -> doing is the bounded retry R-LOOP mandates.
        todo.upsert("b", TodoStatus::Doing, None).unwrap();
        todo.upsert("b", TodoStatus::Fail, Some("cargo test: 1 failed"))
            .unwrap();
        todo.upsert("b", TodoStatus::Doing, None).unwrap();
        assert_eq!(todo.tasks[1].status, TodoStatus::Doing);
        assert_eq!(
            todo.tasks[1].evidence.as_deref(),
            Some("cargo test: 1 failed"),
            "evidence is additive; a re-assert must not blank it"
        );
    }

    #[test]
    fn closure_refuses_done_clean_while_workers_run_and_names_them() {
        let mut todo = OracleTodo::new("oracle-academy");
        todo.set_plan(["a"]);
        todo.upsert("a", TodoStatus::Done, None).unwrap();
        // The plan is COMPLETE, so the only possible refusal is the worker one.
        let live = LiveWorkers {
            terminal: vec!["academy-worker-1".to_string()],
            running: vec!["academy-worker-2".to_string()],
        };
        let err = evaluate_closure(&todo, &live, None, DoneStatus::DoneClean)
            .expect_err("must refuse while a worker runs");
        assert_eq!(
            err,
            ClosureRefusal::WorkersRunning(vec!["academy-worker-2".to_string()])
        );
        assert!(err.to_string().contains("academy-worker-2"), "must name it");
    }

    #[test]
    fn an_honest_non_clean_status_is_never_refused() {
        let todo = OracleTodo::new("oracle-academy"); // no plan at all
        let live = LiveWorkers {
            terminal: vec![],
            running: vec!["academy-worker-2".to_string()],
        };
        for status in [DoneStatus::Blocked, DoneStatus::Pending, DoneStatus::Failed] {
            let plan = evaluate_closure(&todo, &live, None, status)
                .unwrap_or_else(|e| panic!("{status:?} must be allowed, got {e}"));
            assert!(
                plan.cascade_workers.is_empty() && plan.scopes_to_release.is_empty(),
                "a non-clean signal closes nothing: the oracle stays alive"
            );
            assert!(!plan.already_closed);
        }
    }

    #[test]
    fn closure_is_idempotent_against_an_existing_closeable_signal() {
        let mut todo = OracleTodo::new("oracle-academy");
        todo.set_plan(["a"]);
        todo.upsert("a", TodoStatus::Done, None).unwrap();
        let live = LiveWorkers {
            terminal: vec!["academy-worker-1".to_string()],
            running: vec![],
        };
        // The signal stores the KEY; the plan holds the full session name.
        let signal = oracle_signal("academy", DoneStatus::DoneClean);
        assert!(signal.is_closeable());

        let plan = evaluate_closure(&todo, &live, Some(&signal), DoneStatus::DoneClean).unwrap();
        assert!(plan.already_closed);
        assert!(
            plan.cascade_workers.is_empty(),
            "a second close must not re-kill anything"
        );
        assert!(plan.scopes_to_release.is_empty());

        // A signal belonging to a DIFFERENT oracle proves nothing about this
        // one, and the first close must still be computed normally.
        let other = oracle_signal("someone-else", DoneStatus::DoneClean);
        let plan = evaluate_closure(&todo, &live, Some(&other), DoneStatus::DoneClean).unwrap();
        assert!(!plan.already_closed);
        assert_eq!(plan.cascade_workers, vec!["academy-worker-1".to_string()]);

        // A non-closeable signal (pending) is not a close either.
        let pending = oracle_signal("academy", DoneStatus::Pending);
        let plan = evaluate_closure(&todo, &live, Some(&pending), DoneStatus::DoneClean).unwrap();
        assert!(!plan.already_closed);
    }

    #[test]
    fn a_clean_close_cascades_finished_workers_and_releases_their_scopes() {
        let mut todo = OracleTodo::new("oracle-academy");
        todo.set_plan(["a"]);
        todo.upsert("a", TodoStatus::Done, None).unwrap();
        let live = LiveWorkers {
            terminal: vec![
                "academy-worker-1".to_string(),
                "academy-worker-2".to_string(),
            ],
            running: vec![],
        };
        let plan = evaluate_closure(&todo, &live, None, DoneStatus::DoneClean).unwrap();
        assert_eq!(plan.cascade_workers, live.terminal);
        assert_eq!(
            plan.scopes_to_release,
            vec![
                "oracle-academy".to_string(),
                "academy-worker-1".to_string(),
                "academy-worker-2".to_string()
            ],
            "the oracle's own claim is released alongside every cascaded worker"
        );
    }

    /// The L4 gate, refusing for the same reasons `cmd_done` downgrades.
    #[test]
    fn closure_refuses_done_clean_on_an_absent_incomplete_or_failed_plan() {
        let live = LiveWorkers::default();

        let empty = OracleTodo::new("oracle-academy");
        assert_eq!(
            evaluate_closure(&empty, &live, None, DoneStatus::DoneClean).unwrap_err(),
            ClosureRefusal::NoPlan
        );

        let mut incomplete = OracleTodo::new("oracle-academy");
        incomplete.set_plan(["a", "b"]);
        incomplete.upsert("a", TodoStatus::Done, None).unwrap();
        assert_eq!(
            evaluate_closure(&incomplete, &live, None, DoneStatus::DoneClean).unwrap_err(),
            ClosureRefusal::PlanIncomplete {
                done: 1,
                total: 2,
                unfinished: vec!["b".to_string()],
            }
        );

        let mut failed = OracleTodo::new("oracle-academy");
        failed.set_plan(["a", "b"]);
        failed.upsert("a", TodoStatus::Done, None).unwrap();
        failed.upsert("b", TodoStatus::Doing, None).unwrap();
        failed.upsert("b", TodoStatus::Fail, None).unwrap();
        assert_eq!(
            evaluate_closure(&failed, &live, None, DoneStatus::DoneClean).unwrap_err(),
            ClosureRefusal::PlanFailed(vec!["b".to_string()])
        );
    }

    #[test]
    fn honest_status_downgrades_and_never_upgrades() {
        let mut incomplete = OracleTodo::new("oracle-academy");
        incomplete.set_plan(["a", "b"]);
        incomplete.upsert("a", TodoStatus::Done, None).unwrap();
        assert_eq!(
            honest_status(DoneStatus::DoneClean, &incomplete),
            DoneStatus::Pending
        );

        let mut failed = OracleTodo::new("oracle-academy");
        failed.set_plan(["a"]);
        failed.upsert("a", TodoStatus::Doing, None).unwrap();
        failed.upsert("a", TodoStatus::Fail, None).unwrap();
        assert_eq!(
            honest_status(DoneStatus::DoneClean, &failed),
            DoneStatus::Failed
        );

        let mut complete = OracleTodo::new("oracle-academy");
        complete.set_plan(["a"]);
        complete.upsert("a", TodoStatus::Done, None).unwrap();
        assert_eq!(
            honest_status(DoneStatus::DoneClean, &complete),
            DoneStatus::DoneClean
        );

        // Never an upgrade: a complete plan does not turn an honest failure,
        // a pending or a block into a clean close.
        for status in [DoneStatus::Failed, DoneStatus::Pending, DoneStatus::Blocked] {
            assert_eq!(honest_status(status, &complete), status);
            assert_eq!(honest_status(status, &failed), status);
        }
    }

    /// The counters a legacy consumer reads must agree with the task list this
    /// module wrote next to them. Verified through `ProgressInfo`, the actual
    /// reader the TUI, `omega list` and patrol use.
    #[test]
    fn counts_match_what_a_legacy_consumer_reads_back() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        let mut todo = OracleTodo::new("oracle-academy-2");
        todo.set_plan(["analyze", "fix", "verify"]);
        todo.upsert("analyze", TodoStatus::Done, None).unwrap();
        todo.upsert("fix", TodoStatus::Doing, None).unwrap();
        todo.save(dir, "oracle-academy-2").unwrap();

        assert_eq!(todo.counts(), (1, 3));
        let info = ProgressInfo::read(dir, "oracle-academy-2").expect("legacy read must parse");
        assert_eq!(info.todos_completed, 1);
        assert_eq!(info.todos_total, 3);
        assert!(info.last_updated.is_some(), "ts must be stamped on save");
        assert!((info.percentage() - 33.333332).abs() < 0.01);

        // And the plan round-trips through the same file.
        let reloaded = OracleTodo::load(dir, "oracle-academy-2").unwrap();
        assert_eq!(reloaded.counts(), (1, 3));
        assert_eq!(reloaded.current().map(|t| t.title.as_str()), Some("fix"));
        assert_eq!(
            reloaded
                .unfinished()
                .iter()
                .map(|t| t.title.as_str())
                .collect::<Vec<_>>(),
            vec!["fix", "verify"]
        );
        assert_eq!(
            reloaded.render_checklist(),
            "✓ analyze\n▸ fix\n☐ verify\n1/3"
        );
    }
}
