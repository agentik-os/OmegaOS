//! Graph executor — the pure decision core that turns a [`Graph`] plus a
//! [`GraphState`] into "what may run now, and what will never run" (R-GRAPH).
//!
//! [`crate::graph`] owns the persisted VOCABULARY and validates that a graph is
//! structurally sound. It deliberately stops there: it cannot say which nodes are
//! runnable right now, it does not count a retry against a ceiling, and it has no
//! opinion on what happens to the dependents of a node that will never finish.
//! This module owns exactly those three answers and nothing else.
//!
//! WHAT THIS IS NOT, and the boundary is the point: no process spawn, no network,
//! no filesystem, no clock. Every function here is a pure computation over its
//! two arguments, so a whole mission can be replayed off a persisted state file
//! and reach the same decisions on any machine. The CLI, the daemon or a test
//! harness drives it: this core says WHAT to run, the caller runs it and hands
//! the results back through [`advance`].
//!
//! The four contracts it implements:
//!
//! 1. FAN-OUT IS A SET, NOT A SEQUENCE. [`ready_nodes`] returns EVERY node whose
//!    incoming edges are satisfied, so the caller dispatches them concurrently.
//!    A node with no incoming edges is ready from the first call. Returning one
//!    node at a time would silently serialize a diamond, which is the exact
//!    wasted wall clock R-GRAPH exists to stop.
//! 2. BOUNDED RETRIES, THEN ESCALATE (R-LOOP). A failed node consults its own
//!    [`crate::mission::RetryPolicy`]. Attempts are counted in [`GraphState`], so
//!    the ceiling survives a restart: a run resumed from disk cannot hand a
//!    thrashing node a fresh budget. When the cap is spent the node is TERMINALLY
//!    failed and the graph either falls back or reports the stranded work.
//! 3. FAILURE PROPAGATES, IT NEVER HANGS. A terminally failed node with no
//!    fallback strands every dependent that can only be reached through it. Those
//!    dependents are reported as unreachable rather than left queued forever,
//!    because a caller that cannot tell "nothing ready right now" from "nothing
//!    will ever be ready" has no choice but to poll a dead graph.
//! 4. IT CONVERGES. Repeated calls strictly progress or terminate. Every retry
//!    spends a per-node budget that is never refunded, and every back edge spends
//!    a per-edge traversal budget that is never refunded, so no sequence of calls
//!    can cycle forever. See [`advance`] for the argument in full.
//!
//! WHERE THE EXECUTOR'S OWN BOOKKEEPING LIVES. Loop traversal counts, failure
//! reasons and the unreachable set are the executor's state, not the graph
//! vocabulary's, so they are written into the forward-compatible `extra` bag of
//! [`GraphState`] under one namespaced key rather than by adding fields to
//! `graph.rs`. Two reasons, and they point the same way: `graph.rs` is owned by
//! the vocabulary and a second writer on it is how two halves of one system
//! drift apart, and an `extra` key round-trips through an older or newer OmegaOS
//! untouched, so a state file written here still loads there.
//!
//! JOIN SEMANTICS ARE AND, ROUTING IS THE CALLER'S. A node waits on ALL of its
//! incoming edges. A [`crate::graph::Router`] is resolved by whoever produced the
//! classification (`Graph::route`), and taking a branch is expressed by the
//! caller reporting the untaken node rather than by this core guessing. Encoding
//! OR-joins here would mean inventing a second branch mechanism beside the
//! deterministic table `graph.rs` already ships.

use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

use crate::graph::{Graph, GraphError, GraphState, Node, NodeId};
use crate::mission::{InvalidTransition, TaskAttemptState};

// ---------------------------------------------------------------------------
// Fallback declaration
// ---------------------------------------------------------------------------

/// Key under which a node declares its fallback, inside [`Node::extra`].
///
/// Declared through the forward-compatible `extra` bag rather than as a new
/// field on [`Node`] for the same reason the bookkeeping below lives in
/// `GraphState::extra`: the fallback is an EXECUTION concern and `graph.rs` is
/// the vocabulary, so this module does not become a second writer there. Because
/// `extra` is `#[serde(flatten)]`, a declared fallback persists in the graph
/// document exactly like a first-class field and survives a round trip through
/// any other OmegaOS version.
pub const FALLBACK_KEY: &str = "fallback";

/// Declare `fallback` as the node taken when `node` exhausts its retries.
///
/// A free function rather than a `Node` method because `Node` belongs to
/// `graph.rs`; adding an inherent method there would put this module's semantics
/// inside the vocabulary file.
pub fn with_fallback(mut node: Node, fallback: impl Into<NodeId>) -> Node {
    let id: NodeId = fallback.into();
    node.extra
        .insert(FALLBACK_KEY.to_string(), Value::String(id.0));
    node
}

/// The fallback declared by `node`, if any. A non-string value is treated as no
/// declaration rather than an error: this reads data that may have been written
/// by another version, and a panic or a hard failure over one malformed optional
/// key would take a whole mission down.
pub fn fallback_of(node: &Node) -> Option<NodeId> {
    node.extra
        .get(FALLBACK_KEY)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(NodeId::new)
}

// ---------------------------------------------------------------------------
// Results the caller hands back
// ---------------------------------------------------------------------------

/// What one dispatched node reported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeResult {
    Succeeded,
    /// Failed with a human-readable reason. The reason is recorded in the state
    /// so a later [`ExecutionOutcome::Failed`] can name the cause instead of
    /// telling the operator only that something, somewhere, went wrong.
    Failed {
        reason: String,
    },
}

/// One node's result, as handed to [`advance`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeReport {
    pub node: NodeId,
    pub result: NodeResult,
}

impl NodeReport {
    pub fn succeeded(node: impl Into<NodeId>) -> Self {
        Self {
            node: node.into(),
            result: NodeResult::Succeeded,
        }
    }

    pub fn failed(node: impl Into<NodeId>, reason: impl Into<String>) -> Self {
        Self {
            node: node.into(),
            result: NodeResult::Failed {
                reason: reason.into(),
            },
        }
    }

    fn is_failure(&self) -> bool {
        matches!(self.result, NodeResult::Failed { .. })
    }
}

// ---------------------------------------------------------------------------
// Outcome
// ---------------------------------------------------------------------------

/// Where the graph stands after a step.
///
/// The variants exist to make ONE distinction impossible to miss: `Progressing`
/// means work is available now, `Blocked` means no work is available and none
/// ever will be for the listed nodes. A caller that cannot tell those apart can
/// only poll forever.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionOutcome {
    /// These nodes may be dispatched now, concurrently.
    Progressing { ready: Vec<NodeId> },
    /// Nothing is runnable and these nodes will never become runnable, because
    /// every path to them runs through a terminally failed node.
    Blocked { unreachable: Vec<NodeId> },
    /// Every node has settled and nothing failed unrecovered.
    Complete,
    /// Everything that could run has run, and this node failed terminally
    /// without stranding anyone. Reported after `Blocked` because `Blocked`
    /// carries strictly more information when both apply; the reason for any
    /// failed node stays retrievable through [`failure_reason`].
    Failed { node: NodeId, reason: String },
}

/// The full result of one [`advance`] call: the outcome plus what the step
/// actually did, so a caller can log a run without diffing two state snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepOutcome {
    pub outcome: ExecutionOutcome,
    /// Reports that were applied to the state this step.
    pub applied: Vec<NodeId>,
    /// Nodes that failed but still had budget, and will be offered again.
    pub retrying: Vec<NodeId>,
    /// Nodes that failed with their retry budget spent. Terminal.
    pub exhausted: Vec<NodeId>,
    /// Fallback nodes unlocked by an exhausted principal this step.
    pub fallbacks: Vec<NodeId>,
    /// Back edges traversed this step, as `(from, to)`.
    pub loops_taken: Vec<(NodeId, NodeId)>,
    /// Nodes newly proven unreachable this step.
    pub newly_unreachable: Vec<NodeId>,
}

impl StepOutcome {
    /// The ready set, or an empty slice when the graph is not progressing.
    pub fn ready(&self) -> &[NodeId] {
        match &self.outcome {
            ExecutionOutcome::Progressing { ready } => ready,
            _ => &[],
        }
    }

    pub fn is_terminal(&self) -> bool {
        !matches!(self.outcome, ExecutionOutcome::Progressing { .. })
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Every way a step can be rejected. A typed enum and never a panic: this core
/// consumes a graph and a state that arrived from disk or from another machine,
/// and a panic there takes down a daemon over one bad file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutorError {
    /// The graph itself is structurally invalid. Re-checked on every step rather
    /// than trusted once, because the caller may have mutated it between calls
    /// and a dangling edge silently strands every node behind it.
    InvalidGraph(GraphError),
    /// A report naming a node this graph does not contain.
    UnknownNode(String),
    /// The same node reported twice in one step. Rejected rather than
    /// last-one-wins: two results for one attempt means the caller lost track of
    /// a dispatch, and silently picking one hides that.
    DuplicateReport(String),
    /// A lifecycle move the mission state machine forbids, surfaced verbatim so
    /// the executor and the ledger report the same error for the same mistake.
    IllegalTransition(InvalidTransition),
}

impl fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGraph(err) => write!(f, "invalid graph: {err}"),
            Self::UnknownNode(id) => write!(f, "result reported for unknown node {id}"),
            Self::DuplicateReport(id) => write!(f, "node {id} reported twice in one step"),
            Self::IllegalTransition(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for ExecutorError {}

impl From<GraphError> for ExecutorError {
    fn from(err: GraphError) -> Self {
        Self::InvalidGraph(err)
    }
}

impl From<InvalidTransition> for ExecutorError {
    fn from(err: InvalidTransition) -> Self {
        Self::IllegalTransition(err)
    }
}

// ---------------------------------------------------------------------------
// Executor bookkeeping inside GraphState::extra
// ---------------------------------------------------------------------------

/// One namespaced key rather than three top-level ones, so this module cannot
/// collide with another writer's `extra` entries.
const EXEC_KEY: &str = "graph_executor";
const LOOPS_KEY: &str = "loop_traversals";
const FAILURES_KEY: &str = "failures";
const UNREACHABLE_KEY: &str = "unreachable";

fn edge_key(from: &NodeId, to: &NodeId) -> String {
    format!("{from}->{to}")
}

fn bag(state: &GraphState) -> Option<&Map<String, Value>> {
    state.extra.get(EXEC_KEY).and_then(Value::as_object)
}

fn bag_mut(state: &mut GraphState) -> &mut Map<String, Value> {
    state
        .extra
        .entry(EXEC_KEY.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    // A non-object value written by something else is replaced rather than
    // fought over: this is our namespace, and refusing to proceed over a
    // malformed key would strand a whole run.
    let slot = state.extra.get_mut(EXEC_KEY).expect("just inserted");
    if !slot.is_object() {
        *slot = Value::Object(Map::new());
    }
    slot.as_object_mut().expect("object")
}

fn sub_map<'a>(state: &'a GraphState, key: &str) -> Option<&'a Map<String, Value>> {
    bag(state)?.get(key).and_then(Value::as_object)
}

fn sub_map_mut<'a>(state: &'a mut GraphState, key: &str) -> &'a mut Map<String, Value> {
    let bag = bag_mut(state);
    let slot = bag
        .entry(key.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    if !slot.is_object() {
        *slot = Value::Object(Map::new());
    }
    slot.as_object_mut().expect("object")
}

/// Traversals already spent on one back edge.
pub fn loop_traversals(state: &GraphState, from: &NodeId, to: &NodeId) -> u32 {
    sub_map(state, LOOPS_KEY)
        .and_then(|map| map.get(&edge_key(from, to)))
        .and_then(Value::as_u64)
        .unwrap_or(0)
        .min(u64::from(u32::MAX)) as u32
}

fn record_traversal(state: &mut GraphState, from: &NodeId, to: &NodeId) -> u32 {
    let next = loop_traversals(state, from, to).saturating_add(1);
    sub_map_mut(state, LOOPS_KEY).insert(edge_key(from, to), Value::from(next));
    next
}

/// The recorded reason a node failed, if it ever did.
pub fn failure_reason(state: &GraphState, id: &NodeId) -> Option<String> {
    sub_map(state, FAILURES_KEY)?
        .get(id.as_str())
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn record_failure(state: &mut GraphState, id: &NodeId, reason: &str) {
    sub_map_mut(state, FAILURES_KEY).insert(id.0.clone(), Value::from(reason));
}

/// Nodes already proven unreachable in this run.
pub fn unreachable_nodes(state: &GraphState) -> Vec<NodeId> {
    bag(state)
        .and_then(|bag| bag.get(UNREACHABLE_KEY))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(NodeId::new)
                .collect()
        })
        .unwrap_or_default()
}

fn record_unreachable(state: &mut GraphState, ids: &BTreeSet<NodeId>) {
    if ids.is_empty() {
        return;
    }
    let mut all: BTreeSet<NodeId> = unreachable_nodes(state).into_iter().collect();
    all.extend(ids.iter().cloned());
    let encoded: Vec<Value> = all.iter().map(|id| Value::from(id.0.clone())).collect();
    bag_mut(state).insert(UNREACHABLE_KEY.to_string(), Value::Array(encoded));
}

// ---------------------------------------------------------------------------
// Lifecycle driving
// ---------------------------------------------------------------------------

/// Walk a node through the mission attempt machine to a target state.
///
/// The machine in [`crate::mission`] has no shortcut from `Queued` to `Accepted`,
/// and reimplementing a looser one here would let a node's recorded lifecycle
/// disagree with the ledger's. So the executor walks the real path and skips any
/// step already taken, which makes it idempotent when a state was restored
/// mid-path from disk.
fn drive(
    state: &mut GraphState,
    id: &NodeId,
    path: &[TaskAttemptState],
) -> Result<(), ExecutorError> {
    for step in path {
        let current = state.state_of(id).unwrap_or(TaskAttemptState::Queued);
        if current == *step {
            continue;
        }
        state.transition(id, *step)?;
    }
    Ok(())
}

const ACCEPT_PATH: [TaskAttemptState; 4] = [
    TaskAttemptState::Running,
    TaskAttemptState::CandidateDone,
    TaskAttemptState::Verifying,
    TaskAttemptState::Accepted,
];

const RETRY_PATH: [TaskAttemptState; 4] = [
    TaskAttemptState::Running,
    TaskAttemptState::CandidateDone,
    TaskAttemptState::Verifying,
    TaskAttemptState::CorrectionRequired,
];

const FAIL_PATH: [TaskAttemptState; 4] = [
    TaskAttemptState::Running,
    TaskAttemptState::CandidateDone,
    TaskAttemptState::Verifying,
    TaskAttemptState::Failed,
];

/// Re-seed a node for a NEW loop iteration.
///
/// Deliberately a re-seed and not a transition: `Accepted` is terminal in the
/// mission machine, so walking backwards from it is illegal and should stay
/// illegal. A second pass through a loop body is a fresh attempt series at those
/// nodes, which the ledger models as a new run of the node, not as an undone
/// acceptance. `attempts` is NOT cleared, so a node that burned failures in
/// round one cannot buy a full new budget by going round again: that is the R-LOOP
/// ceiling holding across iterations rather than resetting with them.
fn reseed_for_iteration(state: &mut GraphState, id: &NodeId) {
    let entry = state.nodes.entry(id.clone()).or_default();
    entry.state = TaskAttemptState::Queued;
}

// ---------------------------------------------------------------------------
// Structural queries
// ---------------------------------------------------------------------------

fn is_back_edge(graph: &Graph, from: &NodeId, to: &NodeId) -> bool {
    graph
        .loop_bounds
        .iter()
        .any(|bound| bound.from == *from && bound.to == *to)
}

fn state_of(state: &GraphState, id: &NodeId) -> TaskAttemptState {
    state.state_of(id).unwrap_or(TaskAttemptState::Queued)
}

/// The node this one substitutes for, when it was declared as a fallback.
fn principal_of(graph: &Graph, id: &NodeId) -> Option<NodeId> {
    graph
        .nodes
        .iter()
        .find(|node| fallback_of(node).as_ref() == Some(id))
        .map(|node| node.id.clone())
}

/// Has this node failed terminally with nothing left to cover for it?
///
/// A declared fallback that has not yet had its turn counts as cover. Treating a
/// failure as unrecovered the instant it happens would strand the very
/// dependents the fallback exists to serve, one step before the fallback is even
/// dispatched: the failure is only final once the fallback is final too.
fn is_unrecovered_failure(graph: &Graph, state: &GraphState, id: &NodeId) -> bool {
    if state_of(state, id) != TaskAttemptState::Failed {
        return false;
    }
    match graph.node(id).and_then(fallback_of) {
        // A fallback naming a node this graph does not contain covers nothing.
        Some(fallback) if graph.node(&fallback).is_some() => matches!(
            state_of(state, &fallback),
            TaskAttemptState::Failed | TaskAttemptState::Cancelled
        ),
        _ => true,
    }
}

/// Is the dependency carried by `from -> to` satisfied?
///
/// Three cases, and the middle one is the whole fallback mechanism: an edge from
/// a failed principal to ITS fallback fires precisely BECAUSE the principal
/// failed, which is what lets a fallback be wired downstream of the node it
/// replaces without deadlocking on it.
fn edge_satisfied(graph: &Graph, state: &GraphState, from: &NodeId, to: &NodeId) -> bool {
    match state_of(state, from) {
        TaskAttemptState::Accepted => true,
        TaskAttemptState::Failed => match graph.node(from).and_then(fallback_of) {
            Some(fallback) if fallback == *to => true,
            Some(fallback) => state_of(state, &fallback) == TaskAttemptState::Accepted,
            None => false,
        },
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// ready_nodes
// ---------------------------------------------------------------------------

/// Every node that may be dispatched right now: the FAN-OUT set.
///
/// A node is ready when it is waiting to run (`Queued`, or `CorrectionRequired`
/// after a retryable failure) and every one of its incoming edges is satisfied.
/// Two edges are deliberately not counted:
///
/// - A declared back edge (a [`crate::graph::LoopBound`]) is ignored, otherwise
///   nothing inside a legal bounded cycle could ever start: every node in the
///   cycle would be waiting on the next one round.
/// - A fallback node stays gated until its principal has terminally failed, so a
///   standalone fallback with no incoming edges does not fire eagerly beside the
///   node it exists to replace.
///
/// Order follows the graph's own node declaration order, so two runs over the
/// same graph dispatch in the same order and a log diff means something.
/// Returns a plain `Vec` rather than a `Result`: this is a lookup over data that
/// [`Graph::validate`] has already had its chance to reject, and it treats an
/// unknown node as `Queued` rather than failing.
pub fn ready_nodes(graph: &Graph, state: &GraphState) -> Vec<NodeId> {
    let mut ready = Vec::new();
    for node in &graph.nodes {
        let id = &node.id;
        if !matches!(
            state_of(state, id),
            TaskAttemptState::Queued | TaskAttemptState::CorrectionRequired
        ) {
            continue;
        }
        if let Some(principal) = principal_of(graph, id) {
            if state_of(state, &principal) != TaskAttemptState::Failed {
                continue;
            }
        }
        let satisfied = graph
            .edges
            .iter()
            .filter(|edge| edge.to == *id)
            .filter(|edge| !is_back_edge(graph, &edge.from, &edge.to))
            .all(|edge| edge_satisfied(graph, state, &edge.from, id));
        if satisfied {
            ready.push(id.clone());
        }
    }
    ready
}

// ---------------------------------------------------------------------------
// advance
// ---------------------------------------------------------------------------

/// Apply the reported results, then say what the graph can do next.
///
/// WHY IT CONVERGES, which is the contract that matters most here. Three
/// monotone counters bound every possible sequence of calls:
///
/// 1. A node's failures are counted in [`GraphState`] and never refunded, so a
///    node can be offered at most `RetryPolicy::max_attempts` times before it is
///    terminally failed (R-LOOP).
/// 2. A back edge's traversals are counted per edge and never refunded, so once
///    a loop's `max_iterations` is spent the edge is dead and what remains is the
///    acyclic graph [`Graph::validate`] already proved acyclic.
/// 3. Every other node moves only forward through the mission attempt machine
///    towards a terminal state.
///
/// So each call either applies at least one result, spends at least one budget,
/// or returns a terminal outcome. Calling it repeatedly with no results is safe
/// and idempotent: it re-reports the same decision instead of mutating.
///
/// PRECEDENCE of the outcome, and it is deliberate: `Progressing` first (work
/// beats analysis), then `Blocked` (stranded nodes are the most actionable
/// terminal fact and name what died), then `Failed`, then `Complete`.
pub fn advance(
    graph: &Graph,
    state: &mut GraphState,
    results: &[NodeReport],
) -> Result<StepOutcome, ExecutorError> {
    graph.validate()?;

    let known: BTreeSet<&str> = graph.nodes.iter().map(|node| node.id.as_str()).collect();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for report in results {
        if !known.contains(report.node.as_str()) {
            return Err(ExecutorError::UnknownNode(report.node.0.clone()));
        }
        if !seen.insert(report.node.as_str()) {
            return Err(ExecutorError::DuplicateReport(report.node.0.clone()));
        }
    }

    let mut applied = Vec::new();
    let mut retrying = Vec::new();
    let mut exhausted = Vec::new();

    for report in results {
        let id = &report.node;
        match &report.result {
            NodeResult::Succeeded => {
                drive(state, id, &ACCEPT_PATH)?;
            }
            NodeResult::Failed { reason } => {
                // The attempt is counted BEFORE the ceiling is tested, so the
                // budget is spent by the failure that just happened rather than
                // by the one after it. A policy of `max_attempts: 3` therefore
                // runs the node exactly three times, never four.
                let spent = state.record_attempt(id);
                let cap = graph
                    .node(id)
                    .map(|node| node.retry.max_attempts)
                    .unwrap_or_default();
                record_failure(state, id, reason);
                if spent < cap {
                    drive(state, id, &RETRY_PATH)?;
                    retrying.push(id.clone());
                } else {
                    drive(state, id, &FAIL_PATH)?;
                    exhausted.push(id.clone());
                }
            }
        }
        applied.push(id.clone());
    }

    // Fallbacks unlocked by an exhaustion in THIS step, reported so a caller can
    // log the substitution rather than infer it from a changed ready set.
    let mut fallbacks = Vec::new();
    for id in &exhausted {
        if let Some(fallback) = graph.node(id).and_then(fallback_of) {
            if graph.node(&fallback).is_some() {
                fallbacks.push(fallback);
            }
        }
    }

    let loops_taken = take_back_edges(graph, state, results);
    let newly_unreachable = strand_dependents(graph, state);

    let outcome = classify(graph, state);

    Ok(StepOutcome {
        outcome,
        applied,
        retrying,
        exhausted,
        fallbacks,
        loops_taken,
        newly_unreachable,
    })
}

/// Spend a traversal on every back edge whose source just completed, and re-seed
/// the loop body so the next round can run.
///
/// Only edges whose source was reported succeeded IN THIS STEP are considered.
/// Testing "source is Accepted" instead would re-take the same back edge on
/// every later call until the budget drained, which turns an idempotent no-op
/// call into silent budget burn.
fn take_back_edges(
    graph: &Graph,
    state: &mut GraphState,
    results: &[NodeReport],
) -> Vec<(NodeId, NodeId)> {
    let completed: BTreeSet<&str> = results
        .iter()
        .filter(|report| !report.is_failure())
        .map(|report| report.node.as_str())
        .collect();

    let mut taken = Vec::new();
    for bound in &graph.loop_bounds {
        if !completed.contains(bound.from.as_str()) {
            continue;
        }
        if state_of(state, &bound.from) != TaskAttemptState::Accepted {
            continue;
        }
        if loop_traversals(state, &bound.from, &bound.to) >= bound.max_iterations {
            continue;
        }
        record_traversal(state, &bound.from, &bound.to);
        for id in loop_body(graph, &bound.from, &bound.to) {
            reseed_for_iteration(state, &id);
        }
        taken.push((bound.from.clone(), bound.to.clone()));
    }
    taken
}

/// The nodes that belong to one loop: everything forward-reachable from `to`
/// that is also backward-reachable from `from`, over ordinary (non back) edges.
///
/// That intersection is exactly the body the back edge sends round again.
/// Re-seeding the whole graph instead would throw away work outside the loop,
/// and re-seeding only `to` would leave the rest of the body stuck `Accepted`
/// and the loop would run once and stop.
fn loop_body(graph: &Graph, from: &NodeId, to: &NodeId) -> BTreeSet<NodeId> {
    let forward = reachable(graph, to, Direction::Forward);
    let backward = reachable(graph, from, Direction::Backward);
    forward.intersection(&backward).cloned().collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Forward,
    Backward,
}

/// Breadth-first reachability over non back edges, including the seed.
///
/// Iterative on purpose: a recursive walk costs stack depth proportional to the
/// graph, and this runs against graphs that arrive from disk.
fn reachable(graph: &Graph, seed: &NodeId, direction: Direction) -> BTreeSet<NodeId> {
    let mut adjacency: BTreeMap<&str, Vec<&NodeId>> = BTreeMap::new();
    for edge in &graph.edges {
        if is_back_edge(graph, &edge.from, &edge.to) {
            continue;
        }
        match direction {
            Direction::Forward => adjacency
                .entry(edge.from.as_str())
                .or_default()
                .push(&edge.to),
            Direction::Backward => adjacency
                .entry(edge.to.as_str())
                .or_default()
                .push(&edge.from),
        }
    }

    let mut seen: BTreeSet<NodeId> = BTreeSet::new();
    seen.insert(seed.clone());
    let mut queue: VecDeque<NodeId> = VecDeque::from([seed.clone()]);
    while let Some(current) = queue.pop_front() {
        for next in adjacency.get(current.as_str()).into_iter().flatten() {
            if seen.insert((*next).clone()) {
                queue.push_back((*next).clone());
            }
        }
    }
    seen
}

/// Cancel and record every node that can now only be reached through a
/// terminally failed node.
///
/// This is contract 3: failure PROPAGATES rather than hanging. Under AND-join a
/// node with even one dead incoming edge can never become ready, so leaving it
/// `Queued` would mean a caller polling a graph that is already over. The walk
/// stops at nodes that already succeeded, because their output exists and
/// nothing behind them is stranded through that path.
fn strand_dependents(graph: &Graph, state: &mut GraphState) -> Vec<NodeId> {
    let mut stranded: BTreeSet<NodeId> = BTreeSet::new();
    let already: BTreeSet<NodeId> = unreachable_nodes(state).into_iter().collect();

    for node in &graph.nodes {
        if !is_unrecovered_failure(graph, state, &node.id) {
            continue;
        }
        let skip = fallback_of(node);
        let mut queue: VecDeque<NodeId> = VecDeque::new();
        for edge in &graph.edges {
            if edge.from != node.id || is_back_edge(graph, &edge.from, &edge.to) {
                continue;
            }
            if skip.as_ref() == Some(&edge.to) {
                continue;
            }
            queue.push_back(edge.to.clone());
        }
        while let Some(current) = queue.pop_front() {
            match state_of(state, &current) {
                // Its output exists, or it is already terminal on its own
                // account: nothing beyond it is stranded through this path.
                TaskAttemptState::Accepted
                | TaskAttemptState::Failed
                | TaskAttemptState::Cancelled => continue,
                _ => {}
            }
            if !stranded.insert(current.clone()) {
                continue;
            }
            for edge in &graph.edges {
                if edge.from == current && !is_back_edge(graph, &edge.from, &edge.to) {
                    queue.push_back(edge.to.clone());
                }
            }
        }
    }

    for id in &stranded {
        // Cancelled is the mission machine's word for "this will not run". A
        // transition that the machine refuses is swallowed rather than raised:
        // the node is unreachable either way, and failing the whole step over
        // the bookkeeping would hide the far more important fact.
        let _ = state.transition(id, TaskAttemptState::Cancelled);
    }
    record_unreachable(state, &stranded);

    stranded
        .into_iter()
        .filter(|id| !already.contains(id))
        .collect()
}

/// Decide the outcome, in the precedence documented on [`advance`].
fn classify(graph: &Graph, state: &GraphState) -> ExecutionOutcome {
    let ready = ready_nodes(graph, state);
    if !ready.is_empty() {
        return ExecutionOutcome::Progressing { ready };
    }

    // Anything recorded unreachable, plus anything still unsettled with nothing
    // ready. The second half is the safety net: an unsettled node that cannot be
    // dispatched is blocked by definition, and reporting it as `Complete` would
    // be a false green.
    let mut blocked: BTreeSet<NodeId> = unreachable_nodes(state).into_iter().collect();
    for node in &graph.nodes {
        if !state_of(state, &node.id).is_terminal() {
            blocked.insert(node.id.clone());
        }
    }
    blocked.retain(|id| graph.node(id).is_some());
    if !blocked.is_empty() {
        return ExecutionOutcome::Blocked {
            unreachable: blocked.into_iter().collect(),
        };
    }

    if let Some(node) = graph
        .nodes
        .iter()
        .find(|node| is_unrecovered_failure(graph, state, &node.id))
    {
        return ExecutionOutcome::Failed {
            node: node.id.clone(),
            reason: failure_reason(state, &node.id)
                .unwrap_or_else(|| "node failed without a recorded reason".to_string()),
        };
    }

    ExecutionOutcome::Complete
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{LoopBound, NodeKind};
    use crate::mission::RetryPolicy;

    fn chain() -> Graph {
        Graph::new()
            .with_node(Node::new("a", NodeKind::Agent))
            .with_node(Node::new("b", NodeKind::Agent))
            .with_node(Node::new("c", NodeKind::Synthesis))
            .with_edge("a", "b")
            .with_edge("b", "c")
    }

    /// a fans out to b and c, both feed the reduce d.
    fn diamond() -> Graph {
        Graph::new()
            .with_node(Node::new("a", NodeKind::Agent))
            .with_node(Node::new("b", NodeKind::Agent))
            .with_node(Node::new("c", NodeKind::Agent))
            .with_node(Node::new("d", NodeKind::Reduce))
            .with_edge("a", "b")
            .with_edge("a", "c")
            .with_edge("b", "d")
            .with_edge("c", "d")
    }

    fn ids(values: &[&str]) -> Vec<NodeId> {
        values.iter().map(|value| NodeId::new(*value)).collect()
    }

    fn step(graph: &Graph, state: &mut GraphState, reports: &[NodeReport]) -> StepOutcome {
        advance(graph, state, reports).expect("step is legal")
    }

    #[test]
    fn linear_chain_advances_one_node_at_a_time() {
        let graph = chain();
        let mut state = GraphState::for_graph(&graph);

        let first = step(&graph, &mut state, &[]);
        assert_eq!(first.ready(), ids(&["a"]).as_slice());

        let second = step(&graph, &mut state, &[NodeReport::succeeded("a")]);
        assert_eq!(second.ready(), ids(&["b"]).as_slice());
        assert_eq!(second.applied, ids(&["a"]));

        let third = step(&graph, &mut state, &[NodeReport::succeeded("b")]);
        assert_eq!(third.ready(), ids(&["c"]).as_slice());

        let last = step(&graph, &mut state, &[NodeReport::succeeded("c")]);
        assert_eq!(last.outcome, ExecutionOutcome::Complete);
    }

    #[test]
    fn diamond_fans_out_to_two_ready_nodes_then_converges() {
        let graph = diamond();
        let mut state = GraphState::for_graph(&graph);

        assert_eq!(
            step(&graph, &mut state, &[]).ready(),
            ids(&["a"]).as_slice()
        );

        // THE fan-out: both branches are offered in one step, not one per call.
        let fanned = step(&graph, &mut state, &[NodeReport::succeeded("a")]);
        assert_eq!(fanned.ready(), ids(&["b", "c"]).as_slice());

        // One branch back: the join still waits on the other, so d is NOT
        // offered. The unreported branch stays ready, because the ready set is
        // idempotent by design: this core says what MAY run, and tracking what
        // is already in flight belongs to the caller that dispatched it.
        let half = step(&graph, &mut state, &[NodeReport::succeeded("b")]);
        assert_eq!(half.ready(), ids(&["c"]).as_slice());
        assert!(
            !half.ready().contains(&NodeId::new("d")),
            "join must wait on every input"
        );

        let joined = step(&graph, &mut state, &[NodeReport::succeeded("c")]);
        assert_eq!(joined.ready(), ids(&["d"]).as_slice());

        let done = step(&graph, &mut state, &[NodeReport::succeeded("d")]);
        assert_eq!(done.outcome, ExecutionOutcome::Complete);
    }

    #[test]
    fn failing_node_retries_exactly_the_policy_and_no_more() {
        let policy = RetryPolicy {
            max_attempts: 3,
            backoff_secs: 0,
        };
        let graph = Graph::new()
            .with_node(Node::new("flaky", NodeKind::Agent).with_retry(policy))
            .with_node(Node::new("after", NodeKind::Synthesis))
            .with_edge("flaky", "after");
        let mut state = GraphState::for_graph(&graph);

        let mut dispatches = 0;
        let mut guard = 0;
        loop {
            guard += 1;
            assert!(guard < 20, "advance must converge");
            let outcome = step(&graph, &mut state, &[]);
            match outcome.outcome {
                ExecutionOutcome::Progressing { ref ready } => {
                    assert_eq!(ready.as_slice(), ids(&["flaky"]).as_slice());
                    dispatches += 1;
                    step(&graph, &mut state, &[NodeReport::failed("flaky", "boom")]);
                }
                _ => break,
            }
        }

        assert_eq!(dispatches, 3, "exactly max_attempts runs, never a fourth");
        assert_eq!(state.attempts_of(&NodeId::new("flaky")), 3);
        assert_eq!(
            state.state_of(&NodeId::new("flaky")),
            Some(TaskAttemptState::Failed)
        );
        assert_eq!(
            failure_reason(&state, &NodeId::new("flaky")).as_deref(),
            Some("boom")
        );
    }

    #[test]
    fn exhausted_node_takes_its_fallback() {
        let policy = RetryPolicy {
            max_attempts: 1,
            backoff_secs: 0,
        };
        let graph = Graph::new()
            .with_node(with_fallback(
                Node::new("primary", NodeKind::Agent).with_retry(policy),
                "backup",
            ))
            .with_node(Node::new("backup", NodeKind::Agent))
            .with_node(Node::new("consume", NodeKind::Synthesis))
            .with_edge("primary", "backup")
            .with_edge("primary", "consume");
        let mut state = GraphState::for_graph(&graph);

        // The fallback is GATED: it does not fire beside its principal.
        assert_eq!(
            step(&graph, &mut state, &[]).ready(),
            ids(&["primary"]).as_slice()
        );

        let failed = step(
            &graph,
            &mut state,
            &[NodeReport::failed("primary", "no capacity")],
        );
        assert_eq!(failed.exhausted, ids(&["primary"]));
        assert_eq!(failed.fallbacks, ids(&["backup"]));
        assert_eq!(failed.ready(), ids(&["backup"]).as_slice());

        // The fallback's success satisfies the dependents of the node it replaced.
        let recovered = step(&graph, &mut state, &[NodeReport::succeeded("backup")]);
        assert_eq!(recovered.ready(), ids(&["consume"]).as_slice());

        let done = step(&graph, &mut state, &[NodeReport::succeeded("consume")]);
        assert_eq!(done.outcome, ExecutionOutcome::Complete);
    }

    #[test]
    fn a_fallback_that_also_dies_stops_covering_and_the_failure_escalates() {
        // Cover is not unlimited: once the fallback is terminal too there is
        // nothing left to wait for, so the dependents must be reported rather
        // than left queued behind two dead nodes.
        let policy = RetryPolicy {
            max_attempts: 1,
            backoff_secs: 0,
        };
        let graph = Graph::new()
            .with_node(with_fallback(
                Node::new("primary", NodeKind::Agent).with_retry(policy.clone()),
                "backup",
            ))
            .with_node(Node::new("backup", NodeKind::Agent).with_retry(policy))
            .with_node(Node::new("consume", NodeKind::Synthesis))
            .with_edge("primary", "backup")
            .with_edge("primary", "consume");
        let mut state = GraphState::for_graph(&graph);

        step(&graph, &mut state, &[]);
        let first = step(&graph, &mut state, &[NodeReport::failed("primary", "down")]);
        assert!(
            first.newly_unreachable.is_empty(),
            "nothing is stranded while the fallback still has its turn"
        );

        let second = step(
            &graph,
            &mut state,
            &[NodeReport::failed("backup", "down too")],
        );
        assert_eq!(second.newly_unreachable, ids(&["consume"]));
        assert_eq!(
            second.outcome,
            ExecutionOutcome::Blocked {
                unreachable: ids(&["consume"]),
            }
        );
    }

    #[test]
    fn exhausted_node_without_fallback_strands_its_dependents() {
        let policy = RetryPolicy {
            max_attempts: 1,
            backoff_secs: 0,
        };
        let graph = Graph::new()
            .with_node(Node::new("root", NodeKind::Agent).with_retry(policy))
            .with_node(Node::new("mid", NodeKind::Agent))
            .with_node(Node::new("leaf", NodeKind::Synthesis))
            .with_edge("root", "mid")
            .with_edge("mid", "leaf");
        let mut state = GraphState::for_graph(&graph);

        step(&graph, &mut state, &[]);
        let dead = step(&graph, &mut state, &[NodeReport::failed("root", "crashed")]);

        // It reports the stranded set instead of hanging on a node that will
        // never complete: that is the whole point of the variant.
        assert_eq!(dead.newly_unreachable, ids(&["leaf", "mid"]));
        assert_eq!(
            dead.outcome,
            ExecutionOutcome::Blocked {
                unreachable: ids(&["leaf", "mid"]),
            }
        );
        assert!(dead.ready().is_empty());

        // And it stays terminal: a second call neither hangs nor changes verdict.
        let again = step(&graph, &mut state, &[]);
        assert_eq!(again.outcome, dead.outcome);
        assert!(again.newly_unreachable.is_empty());
    }

    #[test]
    fn terminal_failure_with_no_dependents_reports_failed() {
        let policy = RetryPolicy {
            max_attempts: 1,
            backoff_secs: 0,
        };
        let graph = Graph::new().with_node(Node::new("solo", NodeKind::Agent).with_retry(policy));
        let mut state = GraphState::for_graph(&graph);

        step(&graph, &mut state, &[]);
        let outcome = step(&graph, &mut state, &[NodeReport::failed("solo", "nope")]);

        assert_eq!(
            outcome.outcome,
            ExecutionOutcome::Failed {
                node: NodeId::new("solo"),
                reason: "nope".to_string(),
            }
        );
    }

    #[test]
    fn bounded_cycle_terminates() {
        // Loop-until-dry: find -> verify, verify loops back to find twice.
        let graph = Graph::new()
            .with_node(Node::new("find", NodeKind::Agent))
            .with_node(Node::new("verify", NodeKind::Verifier))
            .with_edge("find", "verify")
            .with_edge("verify", "find")
            .with_loop_bound(LoopBound::new("verify", "find", 2));
        let mut state = GraphState::for_graph(&graph);

        let mut rounds = 0;
        let mut guard = 0;
        let outcome = loop {
            guard += 1;
            assert!(guard < 50, "a bounded cycle must converge");
            let outcome = step(&graph, &mut state, &[]);
            let ready = match outcome.outcome {
                ExecutionOutcome::Progressing { ref ready } => ready.clone(),
                other => break other,
            };
            if ready.contains(&NodeId::new("find")) {
                rounds += 1;
            }
            let reports: Vec<NodeReport> = ready
                .iter()
                .map(|id| NodeReport::succeeded(id.0.as_str()))
                .collect();
            step(&graph, &mut state, &reports);
        };

        assert_eq!(outcome, ExecutionOutcome::Complete);
        // The body ran once, then once per traversal of the bounded back edge.
        assert_eq!(rounds, 3);
        assert_eq!(
            loop_traversals(&state, &NodeId::new("verify"), &NodeId::new("find")),
            2,
            "the traversal budget is spent, never refunded"
        );
    }

    #[test]
    fn advance_on_a_complete_graph_reports_complete_and_stays_idempotent() {
        let graph = chain();
        let mut state = GraphState::for_graph(&graph);
        for id in ["a", "b", "c"] {
            step(&graph, &mut state, &[]);
            step(&graph, &mut state, &[NodeReport::succeeded(id)]);
        }

        let before = state.clone();
        let first = step(&graph, &mut state, &[]);
        assert_eq!(first.outcome, ExecutionOutcome::Complete);
        assert!(first.applied.is_empty());

        let second = step(&graph, &mut state, &[]);
        assert_eq!(second.outcome, ExecutionOutcome::Complete);
        assert_eq!(state, before, "a no-op step must not mutate the state");
    }

    #[test]
    fn a_clean_graph_never_yields_the_same_node_twice() {
        let graph = diamond();
        assert_eq!(graph.validate(), Ok(()));
        let mut state = GraphState::for_graph(&graph);

        let mut dispatched: Vec<NodeId> = Vec::new();
        let mut guard = 0;
        loop {
            guard += 1;
            assert!(guard < 50, "advance must converge");
            let outcome = step(&graph, &mut state, &[]);
            let ready = match outcome.outcome {
                ExecutionOutcome::Progressing { ref ready } => ready.clone(),
                _ => break,
            };
            for id in &ready {
                assert!(
                    !dispatched.contains(id),
                    "node {id} was offered twice in an acyclic graph"
                );
            }
            dispatched.extend(ready.iter().cloned());
            let reports: Vec<NodeReport> = ready
                .iter()
                .map(|id| NodeReport::succeeded(id.0.as_str()))
                .collect();
            step(&graph, &mut state, &reports);
        }

        assert_eq!(dispatched, ids(&["a", "b", "c", "d"]));
    }

    #[test]
    fn malformed_input_is_a_typed_error_never_a_panic() {
        let graph = chain();
        let mut state = GraphState::for_graph(&graph);

        assert_eq!(
            advance(&graph, &mut state, &[NodeReport::succeeded("ghost")]),
            Err(ExecutorError::UnknownNode("ghost".to_string()))
        );
        assert_eq!(
            advance(
                &graph,
                &mut state,
                &[NodeReport::succeeded("a"), NodeReport::failed("a", "x")],
            ),
            Err(ExecutorError::DuplicateReport("a".to_string()))
        );

        let broken = Graph::new()
            .with_node(Node::new("a", NodeKind::Agent))
            .with_edge("a", "ghost");
        let mut broken_state = GraphState::for_graph(&broken);
        assert!(matches!(
            advance(&broken, &mut broken_state, &[]),
            Err(ExecutorError::InvalidGraph(GraphError::DanglingEdge { .. }))
        ));
    }

    #[test]
    fn executor_bookkeeping_round_trips_through_the_state_document() {
        // The executor's counters live in GraphState::extra, so they must
        // survive a save and a reload like any other persisted field.
        let policy = RetryPolicy {
            max_attempts: 1,
            backoff_secs: 0,
        };
        let graph = Graph::new()
            .with_node(Node::new("root", NodeKind::Agent).with_retry(policy))
            .with_node(Node::new("leaf", NodeKind::Synthesis))
            .with_edge("root", "leaf");
        let mut state = GraphState::for_graph(&graph);
        step(&graph, &mut state, &[]);
        step(&graph, &mut state, &[NodeReport::failed("root", "boom")]);

        let encoded = serde_json::to_value(&state).expect("serializes");
        let restored: GraphState = serde_json::from_value(encoded).expect("parses");
        assert_eq!(restored, state);
        assert_eq!(
            failure_reason(&restored, &NodeId::new("root")).as_deref(),
            Some("boom")
        );
        assert_eq!(unreachable_nodes(&restored), ids(&["leaf"]));
    }

    #[test]
    fn fallback_declaration_survives_a_graph_round_trip() {
        let node = with_fallback(Node::new("primary", NodeKind::Agent), "backup");
        assert_eq!(fallback_of(&node), Some(NodeId::new("backup")));

        let graph = Graph::new()
            .with_node(node)
            .with_node(Node::new("backup", NodeKind::Agent));
        let encoded = serde_json::to_value(&graph).expect("serializes");
        let restored: Graph = serde_json::from_value(encoded).expect("parses");
        assert_eq!(
            fallback_of(&restored.nodes[0]),
            Some(NodeId::new("backup")),
            "a fallback declared in extra must persist like a real field"
        );
        assert_eq!(restored.validate(), Ok(()));
    }
}
