//! Task (a): typed decomposition DAG.
//!
//! Converts the kit breakdown pattern (explorer -> architect -> parallel
//! coders -> critic, each coder owning a vertical slice) into typed nodes:
//! every node declares its input type, output type, judge, and receipt.
//! NOTHING runs until the whole DAG typechecks, and nodes run in DAG order.
//!
//! Sources: kit `SUBAGENT_SWARM_LOGIC.md` orchestration pattern (read-only
//! ref) + Trash `design/skills/design-spec.md` (contract-before-code) +
//! N3 `SUBAGENT-CONTRACT.md` acceptance shapes. Date: 2026-10-31.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

/// The six typed nodes. Coders are parallel siblings; each owns one
/// vertical slice (feature/surface/contract frontier, never a horizontal layer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    Explorer,
    Architect,
    SurfaceCoder,
    GraphCoder,
    WorkbenchCoder,
    Critic,
}

impl NodeKind {
    /// All nodes in canonical declaration order (NOT execution order).
    pub const ALL: [NodeKind; 6] = [
        NodeKind::Explorer,
        NodeKind::Architect,
        NodeKind::SurfaceCoder,
        NodeKind::GraphCoder,
        NodeKind::WorkbenchCoder,
        NodeKind::Critic,
    ];

    /// Declared input artifact type. Must equal the predecessor's output
    /// type along every DAG edge or [`typecheck`] fails before any run.
    pub fn input_type(self) -> &'static str {
        match self {
            NodeKind::Explorer => "RepoSnapshot",
            NodeKind::Architect => "Inventory",
            NodeKind::SurfaceCoder | NodeKind::GraphCoder | NodeKind::WorkbenchCoder => {
                "Contracts"
            }
            NodeKind::Critic => "AssembledBuild",
        }
    }

    /// Declared output artifact type.
    pub fn output_type(self) -> &'static str {
        match self {
            NodeKind::Explorer => "Inventory",
            NodeKind::Architect => "Contracts",
            NodeKind::SurfaceCoder | NodeKind::GraphCoder | NodeKind::WorkbenchCoder => {
                "SliceBuild"
            }
            NodeKind::Critic => "Verdict",
        }
    }

    /// Vertical slice owned by this node (dirs it may write; disjoint).
    pub fn owns(self) -> &'static [&'static str] {
        match self {
            NodeKind::Explorer => &[],
            NodeKind::Architect => &["src/types/", "src/store/"],
            NodeKind::SurfaceCoder => &["src/components/layout/", "src/components/ui/"],
            NodeKind::GraphCoder => &["src/components/graph/", "src/components/dag/"],
            NodeKind::WorkbenchCoder => &["src/components/workbench/", "src/components/editors/"],
            NodeKind::Critic => &["qa/verdicts/"],
        }
    }
}

/// Judge gate attached to each node. A node passes only its own judge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Judge {
    InventoryComplete,
    ContractGate,
    Typecheck,
    VisualQa,
}

impl NodeKind {
    /// The judge that must pass for this node. Never self-graded: the judge
    /// is named BEFORE the node runs (runbook PLAN-before-EXECUTE).
    pub fn judge(self) -> Judge {
        match self {
            NodeKind::Explorer => Judge::InventoryComplete,
            NodeKind::Architect => Judge::ContractGate,
            NodeKind::SurfaceCoder | NodeKind::GraphCoder | NodeKind::WorkbenchCoder => {
                Judge::Typecheck
            }
            NodeKind::Critic => Judge::VisualQa,
        }
    }
}

/// Per-node receipt: evidence the judge ran and what it decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeReceipt {
    pub node: NodeKind,
    pub input_type: &'static str,
    pub output_type: &'static str,
    pub judge: Judge,
    pub verdict: Verdict,
    /// 1-based position in the executed order.
    pub seq: usize,
}

/// Judge verdict. Only `Pass` advances the DAG.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail(String),
}

/// DAG edges: (upstream, downstream). Coders fan out from Architect and
/// rejoin at Critic. Critic consumes the assembled build, not raw slices:
/// `SurfaceCoder.output = SliceBuild`, `Critic.input = AssembledBuild` —
/// the assembly step is typed by [`ASSEMBLY`] below, not by an edge.
pub const EDGES: [(NodeKind, NodeKind); 7] = [
    (NodeKind::Explorer, NodeKind::Architect),
    (NodeKind::Architect, NodeKind::SurfaceCoder),
    (NodeKind::Architect, NodeKind::GraphCoder),
    (NodeKind::Architect, NodeKind::WorkbenchCoder),
    (NodeKind::SurfaceCoder, NodeKind::Critic),
    (NodeKind::GraphCoder, NodeKind::Critic),
    (NodeKind::WorkbenchCoder, NodeKind::Critic),
];

/// The slice-assembly step: three `SliceBuild`s become one `AssembledBuild`.
/// Typed explicitly so the coder->critic boundary still typechecks.
pub struct Assembly;
impl Assembly {
    pub const INPUT_TYPE: &'static str = "SliceBuild";
    pub const OUTPUT_TYPE: &'static str = "AssembledBuild";
}

/// What can go wrong before or during a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DagError {
    /// Edge (upstream -> downstream) with mismatched artifact types.
    TypeMismatch {
        from: NodeKind,
        from_out: &'static str,
        to: NodeKind,
        to_in: &'static str,
    },
    /// Assembly boundary broken (coder output or critic input changed).
    AssemblyMismatch { coder_out: &'static str, critic_in: &'static str },
    /// Proposed execution order violates DAG dependencies.
    OrderViolation { node: NodeKind, missing_dep: NodeKind },
    /// Dependency cycle (no valid topological order exists).
    Cycle,
    /// A judge failed; nothing downstream ran.
    JudgeFailed { node: NodeKind, reason: String },
}

/// Check every edge: upstream output type == downstream input type, plus the
/// assembly boundary. MUST be called (and succeed) before any node runs.
pub fn typecheck() -> Result<(), DagError> {
    typecheck_edges(&EDGES)
}

fn typecheck_edges(edges: &[(NodeKind, NodeKind)]) -> Result<(), DagError> {
    for (from, to) in edges.iter() {
        // Coder -> Critic edges pass through Assembly; check each half.
        if *to == NodeKind::Critic
            && (*from == NodeKind::SurfaceCoder
                || *from == NodeKind::GraphCoder
                || *from == NodeKind::WorkbenchCoder)
        {
            if from.output_type() != Assembly::INPUT_TYPE {
                return Err(DagError::TypeMismatch {
                    from: *from,
                    from_out: from.output_type(),
                    to: *to,
                    to_in: Assembly::INPUT_TYPE,
                });
            }
            continue;
        }
        if from.output_type() != to.input_type() {
            return Err(DagError::TypeMismatch {
                from: *from,
                from_out: from.output_type(),
                to: *to,
                to_in: to.input_type(),
            });
        }
    }
    // Assembly boundary: coder slices in, assembled build out to Critic.
    if NodeKind::SurfaceCoder.output_type() != Assembly::INPUT_TYPE
        || Assembly::OUTPUT_TYPE != NodeKind::Critic.input_type()
    {
        return Err(DagError::AssemblyMismatch {
            coder_out: NodeKind::SurfaceCoder.output_type(),
            critic_in: NodeKind::Critic.input_type(),
        });
    }
    Ok(())
}

/// Topological order of the DAG (Kahn's algorithm). Deterministic: ties
/// broken by canonical declaration order. `Err(Cycle)` if none exists.
pub fn topo_order() -> Result<Vec<NodeKind>, DagError> {
    topo_order_edges(&EDGES)
}

fn topo_order_edges(edges: &[(NodeKind, NodeKind)]) -> Result<Vec<NodeKind>, DagError> {
    let mut indegree: HashMap<NodeKind, usize> = HashMap::new();
    let mut outgoing: HashMap<NodeKind, Vec<NodeKind>> = HashMap::new();
    for n in NodeKind::ALL {
        indegree.insert(n, 0);
        outgoing.insert(n, Vec::new());
    }
    for (from, to) in edges {
        if let Some(v) = outgoing.get_mut(from) { v.push(*to) }
        *indegree.get_mut(to).expect("known node") += 1;
    }
    let mut ready: VecDeque<NodeKind> = NodeKind::ALL
        .iter()
        .filter(|n| indegree[*n] == 0)
        .copied()
        .collect();
    let mut order = Vec::with_capacity(NodeKind::ALL.len());
    while let Some(n) = ready.pop_front() {
        order.push(n);
        let mut next: Vec<NodeKind> = outgoing[&n].clone();
        next.sort_by_key(|m| NodeKind::ALL.iter().position(|a| a == m).unwrap_or(99));
        for m in next {
            let d = indegree.get_mut(&m).expect("known node");
            *d -= 1;
            if *d == 0 {
                ready.push_back(m);
            }
        }
    }
    if order.len() == NodeKind::ALL.len() {
        Ok(order)
    } else {
        Err(DagError::Cycle)
    }
}

/// Verify a proposed execution order respects all DAG dependencies:
/// every node's upstream deps must appear earlier in the order.
pub fn check_order(order: &[NodeKind]) -> Result<(), DagError> {
    check_order_edges(order, &EDGES)
}

fn check_order_edges(order: &[NodeKind], edges: &[(NodeKind, NodeKind)]) -> Result<(), DagError> {
    let pos: HashMap<NodeKind, usize> = order
        .iter()
        .enumerate()
        .map(|(i, n)| (*n, i))
        .collect();
    if pos.len() != NodeKind::ALL.len() {
        // Missing or duplicated node: find an unsatisfied dependency to report.
        for n in NodeKind::ALL {
            if !pos.contains_key(&n) {
                return Err(DagError::OrderViolation { node: n, missing_dep: n });
            }
        }
    }
    for (from, to) in edges {
        let (pf, pt) = (pos[from], pos[to]);
        if pf >= pt {
            return Err(DagError::OrderViolation { node: *to, missing_dep: *from });
        }
    }
    Ok(())
}

/// Execution log used by tests to prove nothing ran before typecheck.
pub static EXEC_LOG: Mutex<Vec<NodeKind>> = Mutex::new(Vec::new());

/// Run the DAG in the proposed order:
/// 1. typecheck the whole DAG (fail = nothing runs, log stays empty);
/// 2. verify the proposed order is topological (fail = nothing runs);
/// 3. execute each node through its judge, emitting a receipt per node.
pub fn run(order: &[NodeKind]) -> Result<Vec<NodeReceipt>, DagError> {
    run_with_sink(order, None)
}

/// Same as run(), but execution events go to `sink` when provided instead of
/// the shared EXEC_LOG. Tests pass a local Vec: fully thread-isolated, so
/// parallel `cargo test` threads cannot pollute each other (QC fix 2026-10-31:
/// the shared static log made `typecheck_before_run_mismatch_runs_nothing`
/// flaky under parallel execution — it passed solo and serial, failed parallel).
pub fn run_with_sink(
    order: &[NodeKind],
    sink: Option<&Mutex<Vec<NodeKind>>>,
) -> Result<Vec<NodeReceipt>, DagError> {
    typecheck()?;
    check_order(order)?;
    let log: &Mutex<Vec<NodeKind>> = sink.unwrap_or(&EXEC_LOG);
    log.lock().expect("log").clear();
    let mut receipts = Vec::with_capacity(order.len());
    for (i, node) in order.iter().enumerate() {
        log.lock().expect("log").push(*node);
        let verdict = judge_node(*node);
        if let Verdict::Fail(reason) = &verdict {
            return Err(DagError::JudgeFailed { node: *node, reason: reason.clone() });
        }
        receipts.push(NodeReceipt {
            node: *node,
            input_type: node.input_type(),
            output_type: node.output_type(),
            judge: node.judge(),
            verdict,
            seq: i + 1,
        });
    }
    Ok(receipts)
}

/// The judge for a node: asserts the node's own contract (non-empty owned
/// slice set where required, judge pre-declared). Deterministic Pass here;
/// real judges (tsc/pytest/screenshots) plug in behind this boundary.
fn judge_node(node: NodeKind) -> Verdict {
    // Every node must declare a receipt schema: input, output, judge.
    if node.input_type().is_empty() || node.output_type().is_empty() {
        return Verdict::Fail(format!("{:?} declares an empty artifact type", node));
    }
    // Vertical-slice law: parallel coders must own disjoint non-empty slices.
    if matches!(
        node,
        NodeKind::SurfaceCoder | NodeKind::GraphCoder | NodeKind::WorkbenchCoder
    ) && node.owns().is_empty()
    {
        return Verdict::Fail(format!("{:?} owns no vertical slice", node));
    }
    Verdict::Pass
}

/// Disjoint-ownership check across the three parallel coders.
pub fn coder_slices_disjoint() -> bool {
    let mut seen: Vec<&str> = Vec::new();
    for c in [
        NodeKind::SurfaceCoder,
        NodeKind::GraphCoder,
        NodeKind::WorkbenchCoder,
    ] {
        for dir in c.owns() {
            if seen.contains(dir) {
                return false;
            }
            seen.push(dir);
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clear_log() {
        EXEC_LOG.lock().expect("log").clear();
    }

    #[test]
    fn node_types_declared_per_node() {
        // Every node: input type, output type, judge — all non-empty/named.
        for n in NodeKind::ALL {
            assert!(!n.input_type().is_empty(), "{:?} input", n);
            assert!(!n.output_type().is_empty(), "{:?} output", n);
            let _ = n.judge();
            assert!(!n.owns().is_empty() || n == NodeKind::Explorer);
        }
    }

    #[test]
    fn typecheck_passes_on_sealed_dag() {
        assert_eq!(typecheck(), Ok(()));
    }

    #[test]
    fn typecheck_before_run_mismatch_runs_nothing() {
        // QC fix 2026-10-31: use a thread-local sink so parallel test threads
        // sharing EXEC_LOG cannot pollute this assertion. A rejected run must
        // execute nothing into OUR sink.
        let local: Mutex<Vec<NodeKind>> = Mutex::new(Vec::new());
        // A DAG with one broken edge must fail typecheck...
        let bad = [
            (NodeKind::Explorer, NodeKind::Architect),
            (NodeKind::Architect, NodeKind::Critic), // Contracts vs AssembledBuild
        ];
        let r = typecheck_edges(&bad);
        assert!(matches!(r, Err(DagError::TypeMismatch { .. })), "got {:?}", r);
        // ...and the real run gate refuses too: prove via check on a bad order.
        let reversed = [
            NodeKind::Critic,
            NodeKind::WorkbenchCoder,
            NodeKind::GraphCoder,
            NodeKind::SurfaceCoder,
            NodeKind::Architect,
            NodeKind::Explorer,
        ];
        // run() typechecks the SEALED dag (valid) then rejects the reversed
        // ORDER — nothing from this call may execute into our sink.
        let r2 = run_with_sink(&reversed, Some(&local));
        assert!(r2.is_err(), "reversed order must be rejected");
        assert!(
            local.lock().expect("log").is_empty(),
            "rejected run must execute nothing, got {:?}",
            local.lock().expect("log")
        );
    }

    #[test]
    fn topo_order_valid_and_critic_last_explorer_first() {
        let order = topo_order().expect("acyclic");
        assert_eq!(order.len(), 6);
        assert_eq!(order[0], NodeKind::Explorer);
        assert_eq!(order[5], NodeKind::Critic);
        check_order(&order).expect("topo order must satisfy deps");
        // Architect before all coders.
        let pa = order.iter().position(|n| *n == NodeKind::Architect).unwrap();
        for c in [
            NodeKind::SurfaceCoder,
            NodeKind::GraphCoder,
            NodeKind::WorkbenchCoder,
        ] {
            assert!(pa < order.iter().position(|n| *n == c).unwrap());
        }
    }

    #[test]
    fn shuffled_input_rejected() {
        // Any order with Critic before a coder, or a coder before Architect,
        // is rejected — try several shuffles, all must fail.
        let shuffles = [
            [
                NodeKind::Explorer,
                NodeKind::Critic,
                NodeKind::Architect,
                NodeKind::SurfaceCoder,
                NodeKind::GraphCoder,
                NodeKind::WorkbenchCoder,
            ],
            [
                NodeKind::Architect,
                NodeKind::Explorer,
                NodeKind::SurfaceCoder,
                NodeKind::GraphCoder,
                NodeKind::WorkbenchCoder,
                NodeKind::Critic,
            ],
            [
                NodeKind::Explorer,
                NodeKind::Architect,
                NodeKind::SurfaceCoder,
                NodeKind::GraphCoder,
                NodeKind::Critic,
                NodeKind::WorkbenchCoder,
            ],
        ];
        for s in shuffles {
            assert!(check_order(&s).is_err(), "shuffle accepted: {:?}", s);
            assert!(run(&s).is_err(), "run accepted shuffle: {:?}", s);
        }
    }

    #[test]
    fn run_emits_receipt_per_node_in_order() {
        clear_log();
        let order = topo_order().expect("acyclic");
        let receipts = run(&order).expect("valid run");
        assert_eq!(receipts.len(), 6);
        for (i, r) in receipts.iter().enumerate() {
            assert_eq!(r.seq, i + 1);
            assert_eq!(r.verdict, Verdict::Pass);
            assert_eq!(r.input_type, r.node.input_type());
            assert_eq!(r.output_type, r.node.output_type());
            assert_eq!(r.judge, r.node.judge());
        }
        assert_eq!(*EXEC_LOG.lock().expect("log"), order);
    }

    #[test]
    fn coder_slices_disjoint_vertical() {
        assert!(coder_slices_disjoint());
    }

    #[test]
    fn cycle_detected() {
        let cyclic = [(NodeKind::Explorer, NodeKind::Explorer)];
        assert_eq!(topo_order_edges(&cyclic), Err(DagError::Cycle));
    }
}
