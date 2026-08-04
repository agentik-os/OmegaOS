//! Execution graph types — the persisted SHAPE of a mission (R-GRAPH).
//!
//! [`crate::mission::PlanContract`] already models a mission as a list of tasks
//! plus `depends_on`, which is a DAG and nothing more. That is enough to answer
//! "what must run before what" and nothing else: it cannot say that a branch is
//! taken on a classification string, and it cannot say that a stage loops until
//! it runs dry, because a `depends_on` cycle is unconditionally rejected there.
//! Both of those are real, daily shapes — the router that sends a finding to the
//! right verifier, and the loop-until-dry sweep that keeps fanning out finders
//! until K rounds surface nothing new.
//!
//! This module is the persisted vocabulary for those shapes, and deliberately
//! ONLY the vocabulary: types, validation, and pure lookups. No executor, no
//! risk gate, no ledger write, no process spawn, no network. A separate task
//! owns each of those, and a type file that quietly grew an executor would be a
//! second writer on their surface.
//!
//! What it reuses rather than re-invents, because a second vocabulary for the
//! same concept is how two halves of one system drift apart:
//!
//! - [`crate::mission::TaskAttemptState`] IS the node lifecycle. A node does not
//!   get its own `Pending | Done | Error` enum; it moves through the exact same
//!   machine an attempt does, so a node's state and the ledger's state are the
//!   same word and [`crate::mission::InvalidTransition`] is the same error.
//! - [`crate::mission::RetryPolicy`] and [`crate::mission::VerifierCheck`] are
//!   attached per node, so R-LOOP's bounded retries and R-VERIFY's checks are
//!   carried by the graph itself instead of re-derived by whoever runs it.
//! - [`crate::mission::TaskId`] links a node back to the task contract that
//!   authorized it, when there is one. Optional, because a reduce node is plain
//!   code and never had a contract.
//!
//! Two invariants shape every struct here:
//!
//! 1. AN OLD FILE STILL LOADS AND AN UNKNOWN FIELD STILL SURVIVES. Every added
//!    field carries `#[serde(default)]` and every struct carries a flattened
//!    `extra` map. A graph written by a newer OmegaOS, read here, and written
//!    back keeps the fields this version has never heard of. There is no
//!    destructive migration anywhere in this file: nothing rewrites, drops, or
//!    renames a key it does not own.
//! 2. THE BRANCH IS DATA, NOT A MODEL CALL. A [`Router`] resolves a
//!    classification string through a `BTreeMap` and a default. The same string
//!    always lands on the same node, on every machine, in every replay. A model
//!    may PRODUCE the classification string somewhere else; the branch it takes
//!    is a table lookup, so there is no "it decided to skip the audit" surprise
//!    and a run is reproducible from the persisted graph alone.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

use crate::mission::{InvalidTransition, RetryPolicy, TaskAttemptState, TaskId, VerifierCheck};

/// Schema version of a persisted [`Graph`] / [`GraphState`].
///
/// Same idiom as [`crate::mission::CONTRACT_SCHEMA_VERSION`]: a bare `u32`
/// written into the document and re-supplied by serde when an older file omits
/// it, so a file that predates the field is readable rather than a parse error.
pub const GRAPH_SCHEMA_VERSION: u32 = 1;

fn graph_schema_version() -> u32 {
    GRAPH_SCHEMA_VERSION
}

// ---------------------------------------------------------------------------
// NodeId — the key everything else points at
// ---------------------------------------------------------------------------

/// Identity of one node, unique inside one graph.
///
/// `Ord` is derived on purpose: routers and per-node state are `BTreeMap`s keyed
/// by this type, so iteration order, serialized key order and therefore error
/// messages are stable across runs. A `HashMap` here would make two validations
/// of the same broken graph report a different node first.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for NodeId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

// ---------------------------------------------------------------------------
// NodeKind — what a node IS, which decides what it costs
// ---------------------------------------------------------------------------

/// The role a node plays in the graph.
///
/// The split that matters is [`NodeKind::Reduce`] versus everything else: a
/// flatten, a dedupe, a filter or a rank between a fan-out and a synthesis is
/// plain code and costs zero model tokens (R-GRAPH). Naming it as its own kind
/// is what stops the next author from burning an agent on an edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// One bounded model call: one input in, one output out.
    Agent,
    /// Pure code between stages. No model call, no tokens.
    Reduce,
    /// Hosts a [`Router`]: classifies, then branches through the table.
    Router,
    /// Sits on an edge and tries to REFUTE a finding before it moves downstream.
    Verifier,
    /// Merges the fan-in into the answer that leaves the graph.
    Synthesis,
}

// ---------------------------------------------------------------------------
// Node
// ---------------------------------------------------------------------------

fn default_node_state() -> TaskAttemptState {
    TaskAttemptState::Queued
}

/// One node: one bounded job, one input, one output.
///
/// `state` is a [`TaskAttemptState`] rather than a graph-local enum so a node
/// and the attempt ledger speak one vocabulary; see the module doc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    /// Bounded retries for this node (R-LOOP). Defaulted so a graph written
    /// before this field existed loads with the mission default rather than
    /// failing to parse.
    #[serde(default)]
    pub retry: RetryPolicy,
    /// The checks that must pass before this node's output is accepted
    /// (R-VERIFY). Empty is legal: a reduce node has nothing to verify.
    #[serde(default)]
    pub checks: Vec<VerifierCheck>,
    #[serde(default = "default_node_state")]
    pub state: TaskAttemptState,
    /// The task contract that authorized this node, when one exists. A reduce
    /// node is plain code and has none, so this is optional rather than a
    /// synthetic id nobody can resolve.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<TaskId>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Node {
    /// A queued node of `kind`, with the mission's default retry policy.
    pub fn new(id: impl Into<NodeId>, kind: NodeKind) -> Self {
        Self {
            id: id.into(),
            kind,
            retry: RetryPolicy::default(),
            checks: Vec::new(),
            state: default_node_state(),
            task_id: None,
            extra: Map::new(),
        }
    }

    pub fn with_task(mut self, task_id: TaskId) -> Self {
        self.task_id = Some(task_id);
        self
    }

    pub fn with_checks(mut self, checks: Vec<VerifierCheck>) -> Self {
        self.checks = checks;
        self
    }

    pub fn with_retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    /// Move this node's lifecycle, through the mission state machine.
    ///
    /// Delegating instead of re-implementing is the whole point: an illegal move
    /// here fails with the same [`InvalidTransition`] the ledger raises, so a
    /// caller has one error type to handle rather than two that disagree.
    pub fn transition(&mut self, next: TaskAttemptState) -> Result<(), InvalidTransition> {
        self.state = self.state.transition(next)?;
        Ok(())
    }
}

impl From<NodeId> for Node {
    fn from(id: NodeId) -> Self {
        Self::new(id, NodeKind::Agent)
    }
}

// ---------------------------------------------------------------------------
// Edge
// ---------------------------------------------------------------------------

/// An edge exists ONLY where data actually moves from `from` to `to`.
///
/// This is the "and then" test made structural (R-GRAPH): if the next node does
/// not READ the previous node's output, there is no edge and the wait it implies
/// is pure latency. Encoding sequencing-that-is-not-dataflow as an edge is the
/// single most common way a graph gets needlessly serial.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Edge {
    pub fn new(from: impl Into<NodeId>, to: impl Into<NodeId>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            extra: Map::new(),
        }
    }

    fn same_ends(&self, from: &NodeId, to: &NodeId) -> bool {
        self.from == *from && self.to == *to
    }
}

// ---------------------------------------------------------------------------
// Router — the branch, as deterministic data
// ---------------------------------------------------------------------------

/// A deterministic branch: a classification string in, one node out.
///
/// `on` names WHAT is being classified (a field name, a probe name) purely as
/// documentation for the reader and the report; it plays no part in resolution.
/// Resolution is [`Router::resolve`] and nothing else: an exact lookup in
/// `routes`, then `default`. No model call, no fuzzy match, no ordering
/// sensitivity — the same classification always resolves to the same node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Router {
    /// What is classified, for humans. Not used to resolve.
    #[serde(default)]
    pub on: String,
    /// Exact classification -> destination. `BTreeMap` so the persisted key
    /// order, and therefore any digest taken over it, is stable.
    #[serde(default)]
    pub routes: BTreeMap<String, NodeId>,
    /// Where an unmatched classification goes. `None` means an unmatched
    /// classification has NOWHERE to go, which is a legal graph: the caller is
    /// then required to handle the miss instead of silently drifting into a
    /// branch nobody chose.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<NodeId>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Router {
    pub fn new(on: impl Into<String>) -> Self {
        Self {
            on: on.into(),
            routes: BTreeMap::new(),
            default: None,
            extra: Map::new(),
        }
    }

    pub fn with_route(mut self, case: impl Into<String>, target: impl Into<NodeId>) -> Self {
        self.routes.insert(case.into(), target.into());
        self
    }

    pub fn with_default(mut self, target: impl Into<NodeId>) -> Self {
        self.default = Some(target.into());
        self
    }

    /// Resolve a classification. Exact match first, then the default.
    pub fn resolve(&self, classification: &str) -> Option<&NodeId> {
        self.routes.get(classification).or(self.default.as_ref())
    }

    /// Every node this router can reach, deduplicated and ordered.
    fn targets(&self) -> impl Iterator<Item = (&str, &NodeId)> {
        self.routes
            .iter()
            .map(|(case, target)| (case.as_str(), target))
    }
}

// ---------------------------------------------------------------------------
// LoopBound — what makes a cycle legal
// ---------------------------------------------------------------------------

/// An explicit, finite bound on one back edge.
///
/// A cycle is not forbidden here: loop-until-dry is a real and useful pattern.
/// What is forbidden is an UNBOUNDED one, because a cycle with no ceiling is a
/// machine that pays to rediscover the same dead ends until the budget dies
/// (R-LOOP, R-BUDGET). A bound declared on a back edge is the proof of
/// convergence: cut the bounded edges and what remains must be acyclic, so every
/// possible loop has to pass through a counter that runs out.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoopBound {
    pub from: NodeId,
    pub to: NodeId,
    /// Hard ceiling on traversals of this edge. Must be at least 1: a bound of 0
    /// does not describe a convergent loop, it describes an edge that can never
    /// be taken, which is almost always a defaulted field rather than an intent.
    pub max_iterations: u32,
    /// Stop after this many consecutive traversals that surfaced nothing new.
    /// Optional, and never a substitute for `max_iterations`: a dry-run counter
    /// only converges if the thing being deduplicated is deduplicated against
    /// everything SEEN, which this file cannot enforce.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_after_dry_rounds: Option<u32>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl LoopBound {
    pub fn new(from: impl Into<NodeId>, to: impl Into<NodeId>, max_iterations: u32) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            max_iterations,
            stop_after_dry_rounds: None,
            extra: Map::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// GraphError
// ---------------------------------------------------------------------------

/// Every way a graph can be structurally wrong. A typed enum, never a panic:
/// this validates data that arrives from disk and from other machines, and a
/// panic there takes down a daemon over a bad file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    UnsupportedSchema {
        version: u32,
    },
    EmptyNodeId,
    DuplicateNodeId(String),
    /// An edge endpoint that is not a node in this graph.
    DanglingEdge {
        from: String,
        to: String,
        missing: String,
    },
    /// A router attached to a node id that does not exist.
    UnknownRouterHost(String),
    /// A route pointing at a node that does not exist.
    UnknownRouterRoute {
        router: String,
        case: String,
        target: String,
    },
    /// A router default pointing at a node that does not exist.
    UnknownRouterDefault {
        router: String,
        target: String,
    },
    /// A loop bound declared on an edge the graph does not have.
    UnknownLoopBoundEdge {
        from: String,
        to: String,
    },
    /// A loop bound that cannot converge to a useful traversal count.
    NonConvergentLoopBound {
        from: String,
        to: String,
    },
    /// A cycle that survives after every bounded back edge is cut.
    UnboundedCycle {
        nodes: Vec<String>,
    },
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use GraphError::*;
        match self {
            UnsupportedSchema { version } => {
                write!(f, "unsupported graph schema version: {version}")
            }
            EmptyNodeId => write!(f, "node id must not be empty"),
            DuplicateNodeId(id) => write!(f, "duplicate node id: {id}"),
            DanglingEdge { from, to, missing } => {
                write!(f, "edge {from} -> {to} references unknown node {missing}")
            }
            UnknownRouterHost(id) => write!(f, "router attached to unknown node {id}"),
            UnknownRouterRoute {
                router,
                case,
                target,
            } => write!(
                f,
                "router {router} route {case:?} points at unknown node {target}"
            ),
            UnknownRouterDefault { router, target } => {
                write!(f, "router {router} default points at unknown node {target}")
            }
            UnknownLoopBoundEdge { from, to } => {
                write!(f, "loop bound declared on missing edge {from} -> {to}")
            }
            NonConvergentLoopBound { from, to } => write!(
                f,
                "loop bound on edge {from} -> {to} has max_iterations 0 and cannot converge"
            ),
            UnboundedCycle { nodes } => write!(
                f,
                "graph contains an unbounded cycle through: {}",
                nodes.join(", ")
            ),
        }
    }
}

impl std::error::Error for GraphError {}

// ---------------------------------------------------------------------------
// Graph
// ---------------------------------------------------------------------------

/// The persisted shape of a mission.
///
/// Every field carries `#[serde(default)]` and the struct carries `extra`, so an
/// OLD file loads (missing fields default) and a NEW file round-trips through
/// this version without losing the keys it does not know (invariant 1 in the
/// module doc). No field here is ever rewritten or dropped on load.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Graph {
    #[serde(default = "graph_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub edges: Vec<Edge>,
    /// Routers keyed by the node that hosts them. Keying by host rather than
    /// carrying a router inline on [`Node`] keeps the branch table addressable
    /// on its own, and keeps `Node` the same shape whether or not it branches.
    #[serde(default)]
    pub routers: BTreeMap<NodeId, Router>,
    #[serde(default)]
    pub loop_bounds: Vec<LoopBound>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            schema_version: GRAPH_SCHEMA_VERSION,
            nodes: Vec::new(),
            edges: Vec::new(),
            routers: BTreeMap::new(),
            loop_bounds: Vec::new(),
            extra: Map::new(),
        }
    }
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_node(mut self, node: Node) -> Self {
        self.nodes.push(node);
        self
    }

    pub fn with_edge(mut self, from: impl Into<NodeId>, to: impl Into<NodeId>) -> Self {
        self.edges.push(Edge::new(from, to));
        self
    }

    pub fn with_router(mut self, host: impl Into<NodeId>, router: Router) -> Self {
        self.routers.insert(host.into(), router);
        self
    }

    pub fn with_loop_bound(mut self, bound: LoopBound) -> Self {
        self.loop_bounds.push(bound);
        self
    }

    pub fn node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.iter().find(|node| node.id == *id)
    }

    /// Resolve a classification at `host`. `None` means either no router lives
    /// there or the classification matched no route and there is no default.
    pub fn route(&self, host: &NodeId, classification: &str) -> Option<&NodeId> {
        self.routers.get(host)?.resolve(classification)
    }

    /// Reject every structurally invalid graph, with the reason.
    ///
    /// The order is deliberate: identity first (a duplicate id makes every later
    /// lookup ambiguous), then references, then reachability of the cycle rule,
    /// so the first error reported is the most upstream cause rather than a
    /// symptom of it.
    pub fn validate(&self) -> Result<(), GraphError> {
        if self.schema_version != GRAPH_SCHEMA_VERSION {
            return Err(GraphError::UnsupportedSchema {
                version: self.schema_version,
            });
        }

        let mut ids: BTreeSet<&str> = BTreeSet::new();
        for node in &self.nodes {
            if node.id.0.trim().is_empty() {
                return Err(GraphError::EmptyNodeId);
            }
            if !ids.insert(node.id.as_str()) {
                return Err(GraphError::DuplicateNodeId(node.id.0.clone()));
            }
        }

        for edge in &self.edges {
            for endpoint in [&edge.from, &edge.to] {
                if !ids.contains(endpoint.as_str()) {
                    return Err(GraphError::DanglingEdge {
                        from: edge.from.0.clone(),
                        to: edge.to.0.clone(),
                        missing: endpoint.0.clone(),
                    });
                }
            }
        }

        for (host, router) in &self.routers {
            if !ids.contains(host.as_str()) {
                return Err(GraphError::UnknownRouterHost(host.0.clone()));
            }
            for (case, target) in router.targets() {
                if !ids.contains(target.as_str()) {
                    return Err(GraphError::UnknownRouterRoute {
                        router: host.0.clone(),
                        case: case.to_string(),
                        target: target.0.clone(),
                    });
                }
            }
            if let Some(target) = &router.default {
                if !ids.contains(target.as_str()) {
                    return Err(GraphError::UnknownRouterDefault {
                        router: host.0.clone(),
                        target: target.0.clone(),
                    });
                }
            }
        }

        for bound in &self.loop_bounds {
            if !self
                .edges
                .iter()
                .any(|edge| edge.same_ends(&bound.from, &bound.to))
            {
                return Err(GraphError::UnknownLoopBoundEdge {
                    from: bound.from.0.clone(),
                    to: bound.to.0.clone(),
                });
            }
            if bound.max_iterations == 0 {
                return Err(GraphError::NonConvergentLoopBound {
                    from: bound.from.0.clone(),
                    to: bound.to.0.clone(),
                });
            }
        }

        self.check_cycles()
    }

    /// Cut every bounded back edge, then require what remains to be acyclic.
    ///
    /// Kahn's algorithm rather than a recursive walk: it needs no stack depth
    /// proportional to the graph, and the nodes it fails to emit ARE the ones
    /// caught in or fed by the cycle, which is the evidence the error carries.
    fn check_cycles(&self) -> Result<(), GraphError> {
        let bounded: BTreeSet<(&str, &str)> = self
            .loop_bounds
            .iter()
            .map(|bound| (bound.from.as_str(), bound.to.as_str()))
            .collect();

        let mut outgoing: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        let mut indegree: BTreeMap<&str, usize> = BTreeMap::new();
        for node in &self.nodes {
            outgoing.entry(node.id.as_str()).or_default();
            indegree.entry(node.id.as_str()).or_insert(0);
        }
        for edge in &self.edges {
            let pair = (edge.from.as_str(), edge.to.as_str());
            if bounded.contains(&pair) {
                continue;
            }
            outgoing.entry(pair.0).or_default().push(pair.1);
            *indegree.entry(pair.1).or_insert(0) += 1;
        }

        let mut queue: VecDeque<&str> = indegree
            .iter()
            .filter(|(_, degree)| **degree == 0)
            .map(|(id, _)| *id)
            .collect();
        let mut emitted = 0usize;
        while let Some(id) = queue.pop_front() {
            emitted += 1;
            for next in outgoing.get(id).into_iter().flatten() {
                let degree = indegree.entry(next).or_insert(0);
                *degree = degree.saturating_sub(1);
                if *degree == 0 {
                    queue.push_back(next);
                }
            }
        }

        if emitted == indegree.len() {
            return Ok(());
        }

        // Whatever still has an indegree is unreachable in topological order,
        // which is exactly the cycle plus what hangs off it.
        let nodes: Vec<String> = indegree
            .iter()
            .filter(|(_, degree)| **degree > 0)
            .map(|(id, _)| (*id).to_string())
            .collect();
        Err(GraphError::UnboundedCycle { nodes })
    }
}

// ---------------------------------------------------------------------------
// GraphState — the mutable half
// ---------------------------------------------------------------------------

/// Runtime state of one node: where it is, and how many attempts it has spent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeRunState {
    #[serde(default = "default_node_state")]
    pub state: TaskAttemptState,
    /// Attempts already spent. Compared against the node's
    /// [`RetryPolicy::max_attempts`] by whoever runs the graph; this file only
    /// records it, because counting is the executor's job and a type file that
    /// enforced the ceiling would be a second authority on R-LOOP.
    #[serde(default)]
    pub attempts: u32,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for NodeRunState {
    fn default() -> Self {
        Self {
            state: default_node_state(),
            attempts: 0,
            extra: Map::new(),
        }
    }
}

impl NodeRunState {
    /// Move this node's lifecycle through the mission state machine, for the
    /// same reason [`Node::transition`] does.
    pub fn transition(&mut self, next: TaskAttemptState) -> Result<(), InvalidTransition> {
        self.state = self.state.transition(next)?;
        Ok(())
    }
}

/// Per-node execution state, persisted beside the [`Graph`] rather than inside
/// it: the graph is the immutable shape, this is what moves. Same forward
/// compatibility contract as [`Graph`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphState {
    #[serde(default = "graph_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub nodes: BTreeMap<NodeId, NodeRunState>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for GraphState {
    fn default() -> Self {
        Self {
            schema_version: GRAPH_SCHEMA_VERSION,
            nodes: BTreeMap::new(),
            extra: Map::new(),
        }
    }
}

impl GraphState {
    pub fn new() -> Self {
        Self::default()
    }

    /// A state map seeded from a graph: every node queued, zero attempts.
    pub fn for_graph(graph: &Graph) -> Self {
        let mut state = Self::new();
        for node in &graph.nodes {
            state.nodes.insert(
                node.id.clone(),
                NodeRunState {
                    state: node.state,
                    attempts: 0,
                    extra: Map::new(),
                },
            );
        }
        state
    }

    pub fn state_of(&self, id: &NodeId) -> Option<TaskAttemptState> {
        self.nodes.get(id).map(|node| node.state)
    }

    pub fn attempts_of(&self, id: &NodeId) -> u32 {
        self.nodes.get(id).map(|node| node.attempts).unwrap_or(0)
    }

    /// Count one more attempt at `id`, creating the entry if the node was never
    /// seen before. `saturating_add` because a wrapped counter would silently
    /// hand a thrashing node a fresh budget.
    pub fn record_attempt(&mut self, id: &NodeId) -> u32 {
        let entry = self.nodes.entry(id.clone()).or_default();
        entry.attempts = entry.attempts.saturating_add(1);
        entry.attempts
    }

    /// Move `id` through the mission state machine.
    pub fn transition(
        &mut self,
        id: &NodeId,
        next: TaskAttemptState,
    ) -> Result<TaskAttemptState, InvalidTransition> {
        let entry = self.nodes.entry(id.clone()).or_default();
        entry.transition(next)?;
        Ok(entry.state)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// fan-out -> reduce -> synthesis, the workhorse diamond, with a router in
    /// front of the two finders.
    fn valid_graph() -> Graph {
        Graph::new()
            .with_node(Node::new("classify", NodeKind::Router))
            .with_node(Node::new("find_bugs", NodeKind::Agent))
            .with_node(Node::new("find_perf", NodeKind::Agent))
            .with_node(Node::new("dedupe", NodeKind::Reduce))
            .with_node(Node::new("synthesize", NodeKind::Synthesis))
            .with_edge("classify", "find_bugs")
            .with_edge("classify", "find_perf")
            .with_edge("find_bugs", "dedupe")
            .with_edge("find_perf", "dedupe")
            .with_edge("dedupe", "synthesize")
            .with_router(
                "classify",
                Router::new("finding_kind")
                    .with_route("bug", "find_bugs")
                    .with_route("perf", "find_perf")
                    .with_default("find_bugs"),
            )
    }

    #[test]
    fn valid_graph_validates_clean() {
        let graph = valid_graph();
        assert_eq!(graph.validate(), Ok(()));
        assert_eq!(graph.schema_version, GRAPH_SCHEMA_VERSION);
    }

    #[test]
    fn unknown_future_field_survives_round_trip() {
        // A graph written by a LATER OmegaOS: fields this version has never
        // heard of, at the graph level, on a node, on an edge, on a router, and
        // in the state document.
        let json = serde_json::json!({
            "schema_version": GRAPH_SCHEMA_VERSION,
            // Every known field is spelled out so the equality below is exact:
            // a defaulted field that serde re-emits is not a dropped key, and
            // omitting them here would test the wrong thing. The old-file case
            // (fields absent entirely) is `old_file_without_added_fields_loads`.
            "nodes": [{
                "id": "a",
                "kind": "agent",
                "retry": {"max_attempts": 3, "backoff_secs": 5},
                "checks": [],
                "state": "queued",
                "budget_tokens": 50_000,
            }],
            "edges": [{"from": "a", "to": "a", "carries": "findings"}],
            "routers": {
                "a": {"on": "verdict", "routes": {}, "weighting": "uniform"}
            },
            "loop_bounds": [{
                "from": "a",
                "to": "a",
                "max_iterations": 3,
                "dry_streak_policy": "strict",
            }],
            "provenance": {"authored_by": "oracle-OmegaOS-9"},
        });

        let graph: Graph =
            serde_json::from_value(json.clone()).expect("old reader parses new file");
        assert_eq!(graph.validate(), Ok(()));

        let round_tripped = serde_json::to_value(&graph).expect("serializes");
        assert_eq!(round_tripped, json, "no key may be dropped on round trip");

        // And the unknown keys are reachable, not merely preserved by accident.
        assert_eq!(
            graph.nodes[0].extra.get("budget_tokens"),
            Some(&Value::from(50_000))
        );
        assert!(graph.extra.contains_key("provenance"));
        assert!(graph.edges[0].extra.contains_key("carries"));
        assert!(graph.routers[&NodeId::new("a")]
            .extra
            .contains_key("weighting"));

        // Same contract for the state document.
        let state_json = serde_json::json!({
            "schema_version": GRAPH_SCHEMA_VERSION,
            "nodes": {"a": {"state": "running", "attempts": 2, "last_worker": "w-1"}},
            "run_id": "run-7",
        });
        let state: GraphState = serde_json::from_value(state_json.clone()).expect("state parses");
        assert_eq!(
            serde_json::to_value(&state).expect("serializes"),
            state_json
        );
    }

    #[test]
    fn old_file_without_added_fields_loads() {
        // No schema_version, no retry, no checks, no state, no routers: the
        // shape a graph had before those fields existed.
        let graph: Graph = serde_json::from_value(serde_json::json!({
            "nodes": [{"id": "a", "kind": "agent"}, {"id": "b", "kind": "reduce"}],
            "edges": [{"from": "a", "to": "b"}],
        }))
        .expect("old file parses");

        assert_eq!(graph.schema_version, GRAPH_SCHEMA_VERSION);
        assert_eq!(graph.nodes[0].retry, RetryPolicy::default());
        assert_eq!(graph.nodes[0].state, TaskAttemptState::Queued);
        assert!(graph.nodes[0].checks.is_empty());
        assert_eq!(graph.validate(), Ok(()));
    }

    #[test]
    fn duplicate_node_id_rejected() {
        let graph = Graph::new()
            .with_node(Node::new("a", NodeKind::Agent))
            .with_node(Node::new("a", NodeKind::Reduce));

        assert_eq!(
            graph.validate(),
            Err(GraphError::DuplicateNodeId("a".to_string()))
        );
    }

    #[test]
    fn dangling_edge_rejected() {
        let graph = Graph::new()
            .with_node(Node::new("a", NodeKind::Agent))
            .with_edge("a", "ghost");

        assert_eq!(
            graph.validate(),
            Err(GraphError::DanglingEdge {
                from: "a".to_string(),
                to: "ghost".to_string(),
                missing: "ghost".to_string(),
            })
        );
    }

    #[test]
    fn router_route_to_missing_node_rejected() {
        let graph = Graph::new()
            .with_node(Node::new("classify", NodeKind::Router))
            .with_node(Node::new("known", NodeKind::Agent))
            .with_router(
                "classify",
                Router::new("verdict")
                    .with_route("keep", "known")
                    .with_route("escalate", "ghost"),
            );

        assert_eq!(
            graph.validate(),
            Err(GraphError::UnknownRouterRoute {
                router: "classify".to_string(),
                case: "escalate".to_string(),
                target: "ghost".to_string(),
            })
        );
    }

    #[test]
    fn unbounded_cycle_rejected() {
        let graph = Graph::new()
            .with_node(Node::new("find", NodeKind::Agent))
            .with_node(Node::new("verify", NodeKind::Verifier))
            .with_edge("find", "verify")
            .with_edge("verify", "find");

        match graph.validate() {
            Err(GraphError::UnboundedCycle { nodes }) => {
                assert_eq!(nodes, vec!["find".to_string(), "verify".to_string()]);
            }
            other => panic!("expected an unbounded cycle, got {other:?}"),
        }
    }

    #[test]
    fn bounded_cycle_accepted() {
        // Loop-until-dry: the back edge carries an explicit finite ceiling, so
        // the loop is provably convergent and therefore legal.
        let graph = Graph::new()
            .with_node(Node::new("find", NodeKind::Agent))
            .with_node(Node::new("verify", NodeKind::Verifier))
            .with_edge("find", "verify")
            .with_edge("verify", "find")
            .with_loop_bound(LoopBound::new("verify", "find", 3));

        assert_eq!(graph.validate(), Ok(()));
    }

    #[test]
    fn zero_iteration_bound_does_not_legalize_a_cycle() {
        let graph = Graph::new()
            .with_node(Node::new("find", NodeKind::Agent))
            .with_node(Node::new("verify", NodeKind::Verifier))
            .with_edge("find", "verify")
            .with_edge("verify", "find")
            .with_loop_bound(LoopBound::new("verify", "find", 0));

        assert_eq!(
            graph.validate(),
            Err(GraphError::NonConvergentLoopBound {
                from: "verify".to_string(),
                to: "find".to_string(),
            })
        );
    }

    #[test]
    fn loop_bound_on_a_missing_edge_rejected() {
        let graph = Graph::new()
            .with_node(Node::new("a", NodeKind::Agent))
            .with_node(Node::new("b", NodeKind::Agent))
            .with_edge("a", "b")
            .with_loop_bound(LoopBound::new("b", "a", 2));

        assert_eq!(
            graph.validate(),
            Err(GraphError::UnknownLoopBoundEdge {
                from: "b".to_string(),
                to: "a".to_string(),
            })
        );
    }

    #[test]
    fn router_resolves_deterministically_and_falls_back_to_default() {
        let graph = valid_graph();
        let host = NodeId::new("classify");

        // Same classification, same node, every time.
        for _ in 0..8 {
            assert_eq!(graph.route(&host, "perf"), Some(&NodeId::new("find_perf")));
        }
        assert_eq!(graph.route(&host, "bug"), Some(&NodeId::new("find_bugs")));

        // Unknown classification falls back to the declared default.
        assert_eq!(
            graph.route(&host, "never_seen"),
            Some(&NodeId::new("find_bugs"))
        );

        // A router with no default reports the miss instead of guessing.
        let strict = Router::new("verdict").with_route("keep", "known");
        assert_eq!(strict.resolve("keep"), Some(&NodeId::new("known")));
        assert_eq!(strict.resolve("drop"), None);

        // A node with no router is a miss, not a panic.
        assert_eq!(graph.route(&NodeId::new("dedupe"), "bug"), None);
    }

    #[test]
    fn node_state_reuses_the_mission_attempt_machine() {
        let mut node = Node::new("a", NodeKind::Agent);
        assert_eq!(node.state, TaskAttemptState::Queued);
        node.transition(TaskAttemptState::Running).expect("legal");
        assert!(node.transition(TaskAttemptState::Accepted).is_err());
        assert_eq!(node.state, TaskAttemptState::Running);
    }

    #[test]
    fn graph_state_tracks_attempts_per_node() {
        let graph = valid_graph();
        let mut state = GraphState::for_graph(&graph);
        let id = NodeId::new("find_bugs");

        assert_eq!(state.state_of(&id), Some(TaskAttemptState::Queued));
        assert_eq!(state.record_attempt(&id), 1);
        assert_eq!(state.record_attempt(&id), 2);
        assert_eq!(state.attempts_of(&id), 2);
        assert_eq!(
            state.transition(&id, TaskAttemptState::Running),
            Ok(TaskAttemptState::Running)
        );
        assert!(state.transition(&id, TaskAttemptState::Accepted).is_err());
        assert_eq!(state.attempts_of(&NodeId::new("dedupe")), 0);
    }
}
