import { create } from 'zustand';

export type PageId =
  | 'platform'
  | 'dispatch'
  | 'tools-mcp'
  | 'research-lab'
  | 'research-hub'
  | 'meta-hackathon'
  | 'ide-workbench'
  | 'swarm-legacy'
  | 'contact'
  | 'sign-up'
  | 'downloads'
  | 'cloud-app';

export type CloudWorkspaceTab =
  | 'code'
  | 'swarm'
  | 'mind'
  | 'research'
  | 'tools'
  | 'settings';

export type SimulationTabId =
  | 'flare-dag'
  | 'orbital-os'
  | 'ares-runbook'
  | 'galaxy-3d'
  | 'xyflow-swarm';

export type FlareColorBy =
  | 'Clusters'
  | 'Activity'
  | 'Hotspots'
  | 'Risk'
  | 'Tests'
  | 'Instability'
  | 'Unread'
  | 'Cycles';

export type FlareDrawAs = 'Canvas' | 'Wheel' | 'Districts';

export interface CloudModelEndpoint {
  id: string;
  name: string;
  provider: string;
  protocol: string;
  latency: string;
  rate: string;
  kvMode: string;
  status: 'online' | 'standby' | 'key_required';
}

export interface UploadedDocItem {
  id: string;
  name: string;
  kind: 'PDF' | 'MD' | 'CODE' | 'YAML';
  size: string;
  sections: number;
  status: 'Verified' | 'Indexed' | 'Linked';
  summary: string;
}

export interface SwarmTaskPacket {
  id: string;
  title: string;
  agent: 'Coordinator' | 'Builder-01' | 'Researcher-02' | 'Verifier-03' | 'Memory' | 'Critic';
  targetFile: string;
  status: 'completed' | 'running' | 'waiting' | 'queued';
  dependencies: string[];
  tokens: string;
  duration: string;
}

export interface ChatTurn {
  id: string;
  sender: 'user' | 'assistant' | 'system';
  authorLabel: string;
  timestamp: string;
  text: string;
  modelUsed?: string;
  bullets?: string[];
  codePatch?: {
    file: string;
    added: number;
    removed: number;
    snippet: string;
  };
}

export interface RunbookNodeData {
  id: 'scope' | 'plan' | 'execute' | 'test' | 'review' | 'receipt';
  label: string;
  subtitle: string;
  status: 'complete' | 'running' | 'pending' | 'review_required';
  duration: string;
  sqlHash: string;
  agent: string;
  details: string;
  checks: string[];
  codeSnippet: string;
}

export const CLOUD_MODELS: CloudModelEndpoint[] = [
  {
    id: 'atoma-n3-70b',
    name: 'Atoma-N3-70B-SpecOps',
    provider: 'Atoma Cloud · Hyperbolic KV',
    protocol: 'O(n log n) KV · Receipts',
    latency: '310ms TTFT',
    rate: '94.2 tok/s',
    kvMode: 'Hyperbolic O(n log n)',
    status: 'online',
  },
  {
    id: 'nemotron-omlx',
    name: 'Nemotron-3.5-Lightning-30B-MLX',
    provider: 'Local oMLX · 127.0.0.1:8000',
    protocol: 'OpenAI-compatible + AtomaDB RAG',
    latency: '140ms TTFT',
    rate: '118.4 tok/s',
    kvMode: 'Unified Apple Silicon',
    status: 'online',
  },
  {
    id: 'beam-501b',
    name: 'Beam-1.0-501B-A23B',
    provider: 'Reflection AI Cloud',
    protocol: 'https://api.reflection.ai/openai/v1',
    latency: '420ms TTFT',
    rate: '78.6 tok/s',
    kvMode: 'Distributed Sparse MoE',
    status: 'online',
  },
  {
    id: 'claude-sonnet',
    name: 'Claude-4.5-Sonnet-SpecOps',
    provider: 'Anthropic Cloud Lane',
    protocol: 'MCP + Seatbelt Hole Staging',
    latency: '380ms TTFT',
    rate: '82.0 tok/s',
    kvMode: '200K Context Pin',
    status: 'online',
  },
  {
    id: 'muse-spark',
    name: 'Muse-Spark-1.3-Contributor',
    provider: 'Meta Muse API',
    protocol: 'META_API_KEY Witness Bus',
    latency: '290ms TTFT',
    rate: '104.5 tok/s',
    kvMode: 'Shared Bus Sequenced',
    status: 'standby',
  },
];

export const INITIAL_UPLOADED_DOCS: UploadedDocItem[] = [
  {
    id: 'doc-1',
    name: 'context-retention-paper.md',
    kind: 'MD',
    size: '48 KB',
    sections: 6,
    status: 'Verified',
    summary:
      'Evaluates objective retention R = |O_T ∩ O_0| / |O_0| across compaction and retrieval regimes.',
  },
  {
    id: 'doc-2',
    name: 'design-contract.md',
    kind: 'MD',
    size: '19 KB',
    sections: 4,
    status: 'Verified',
    summary:
      'Specifies deterministic event IDs, deduplication invariants, and mandatory human-review gates.',
  },
  {
    id: 'doc-3',
    name: 'verified-reasoning-linear-systems.pdf',
    kind: 'PDF',
    size: '1.4 MB',
    sections: 12,
    status: 'Indexed',
    summary:
      'Proof 3.1: Convergence holds when λ < 1. Bounds multi-agent holonomy drift across worktrees.',
  },
  {
    id: 'doc-4',
    name: 'acceptance-contract.rs',
    kind: 'CODE',
    size: '12 KB',
    sections: 3,
    status: 'Linked',
    summary:
      'Rust integration test contract verifying RunbookEvent deduplication and human review gate.',
  },
];

export const INITIAL_SWARM_TASKS: SwarmTaskPacket[] = [
  {
    id: 'pkt-01',
    title: 'Validate runbook proposal & scope boundaries',
    agent: 'Coordinator',
    targetFile: 'docs/spec.md',
    status: 'completed',
    dependencies: [],
    tokens: '3.2K',
    duration: '1.1s',
  },
  {
    id: 'pkt-02',
    title: 'Decompose plan & assign worktree packets',
    agent: 'Coordinator',
    targetFile: 'docs/plan.md',
    status: 'completed',
    dependencies: ['pkt-01'],
    tokens: '4.1K',
    duration: '1.4s',
  },
  {
    id: 'pkt-03',
    title: 'Retrieve 12 sources & index N³ citations',
    agent: 'Researcher-02',
    targetFile: 'manuscripts/context-retention-paper.md',
    status: 'completed',
    dependencies: ['pkt-02'],
    tokens: '8.7K',
    duration: '2.8s',
  },
  {
    id: 'pkt-04',
    title: 'Implement stable event IDs & deduplication in runbook.rs',
    agent: 'Builder-01',
    targetFile: 'src/lib/runbook.rs',
    status: 'running',
    dependencies: ['pkt-02', 'pkt-03'],
    tokens: '12.4K',
    duration: '3.4s',
  },
  {
    id: 'pkt-05',
    title: 'Verify human-review gate invariant & event dedup tests',
    agent: 'Verifier-03',
    targetFile: 'tests/acceptance.rs',
    status: 'waiting',
    dependencies: ['pkt-04'],
    tokens: '5.1K',
    duration: '1.6s',
  },
  {
    id: 'pkt-06',
    title: 'Adversarial review & N³ SQLite receipt commit',
    agent: 'Critic',
    targetFile: 'src/lib/receipt.rs',
    status: 'queued',
    dependencies: ['pkt-05'],
    tokens: '2.9K',
    duration: '0.9s',
  },
];

export const RUNBOOK_NODES: Record<RunbookNodeData['id'], RunbookNodeData> = {
  scope: {
    id: 'scope',
    label: 'Scope',
    subtitle: 'Define the question and context.',
    status: 'complete',
    duration: '0.42s',
    sqlHash: 'n3_8f92a1c',
    agent: 'researcher-05',
    details: 'Ingests repository AST, N³ SQLite prior decisions, and explicit boundary constraints before planning.',
    checks: ['Repository boundary verified', 'Context window pinned (34.2k tokens)', 'Zero external network egress'],
    codeSnippet: `# runbook.yaml — Step 01: Scope
mission: evaluate-runbook-loop
context:
  repository: atoma-engineering/ares-core
  sqlite_db: .ares/n3_traceability.db
  constraints:
    - deterministic_receipts: true
    - max_agent_depth: 4
    - human_gate: before_commit`,
  },
  plan: {
    id: 'plan',
    label: 'Plan',
    subtitle: 'Create the runbook and define steps.',
    status: 'complete',
    duration: '1.18s',
    sqlHash: 'n3_4b19e0d',
    agent: 'planner-01',
    details: 'Synthesizes a typed DAG execution contract with explicit MCP tool permissions and rollback checkpoints.',
    checks: ['DAG acyclic verification passed', 'MCP capability contract matched', 'Pre-registration claims logged'],
    codeSnippet: `-- N³ SQLite Decision Record (Step 02: Plan)
INSERT INTO n3_decisions (run_id, step, intent, rationale, selected_plan)
VALUES (
  'run_2026_10_31_09',
  'plan',
  'Verify DAG dependency parser stability under concurrent edits',
  'Isolate shared/insights.ts and shared/parser.ts hotspots',
  'dag_contract_v4'
);`,
  },
  execute: {
    id: 'execute',
    label: 'Execute',
    subtitle: 'Run with tools and agents.',
    status: 'running',
    duration: '2.84s',
    sqlHash: 'n3_7c31f8b',
    agent: 'coder-02 · executor-03',
    details: 'Executes sandboxed tool calls across Rust/C++ WebGPU workers with real-time trace capture.',
    checks: ['Tool calls completed (14/14)', 'WebGPU compute shader pass verified', 'AST diff written to shadow worktree'],
    codeSnippet: `// agents/coder-02/src/main.rs — Step 03: Execute
pub async fn run_agent(config: AgentConfig) -> Result<Receipt> {
  let (tx, mut rx) = mpsc::channel(32);
  let executor = CodeExecutor::new(&config).await?;
  while let Some(task) = rx.recv().await {
    let outcome = executor.execute_sandboxed(task).await?;
    n3_sqlite::record_execution(&outcome).await?;
  }
  Ok(executor.seal_receipt())
}`,
  },
  test: {
    id: 'test',
    label: 'Test',
    subtitle: 'Validate results and check claims.',
    status: 'complete',
    duration: '1.62s',
    sqlHash: 'n3_2e55d9a',
    agent: 'verifier-04',
    details: 'Runs deterministic unit, property, and snapshot suites against the pre-registered hypothesis.',
    checks: ['14/14 unit & integration specs passed', 'Zero memory leaks in WebGPU buffer pool', 'Latency within <1.5s budget'],
    codeSnippet: `# Verification harness output — Step 04: Test
$ cargo test --workspace --release
running 14 tests
test dag::parser::tests::resolves_cyclic_imports ... ok
test n3::sqlite::tests::verifies_merkle_receipt ... ok
test webgpu::kernel::tests::dispatch_workgroups ... ok
test result: ok. 14 passed; 0 failed; finished in 1.62s`,
  },
  review: {
    id: 'review',
    label: 'Human review',
    subtitle: 'Review, assess and decide.',
    status: 'review_required',
    duration: 'Gate active',
    sqlHash: 'n3_9a04c2e',
    agent: 'critic-06 + Human Builder',
    details: 'Pauses autonomous promotion so the solo builder can inspect the AST diff, risk score, and evidence trail.',
    checks: ['7 changed files ordered by risk', 'Critic-06 static analysis score: 94/100', 'Awaiting explicit builder signature'],
    codeSnippet: `// Acceptance Gate — Step 05: Human Review
{
  "gate": "explicit_human_review",
  "status": "awaiting_signature",
  "risk_hotspots": ["shared/insights.ts (cx96)", "shared/parser.ts (cx120)"],
  "critic_score": 0.94,
  "promote_to_main": false
}`,
  },
  receipt: {
    id: 'receipt',
    label: 'Receipt',
    subtitle: 'Record evidence and close the loop.',
    status: 'complete',
    duration: '0.19s',
    sqlHash: 'n3_f018b77',
    agent: 'coordinator-12',
    details: 'Seals the immutable N³ SQLite receipt linking intent, tool inputs, diffs, test logs, and human sign-off.',
    checks: ['SHA-256 artifact digest committed', 'N³ lineage graph updated', 'Reproducible runbook exported'],
    codeSnippet: `-- N³ Immutable Evidence Receipt — Step 06: Receipt
SELECT r.receipt_hash, d.intent, a.status, r.created_at
FROM n3_receipts r
JOIN n3_decisions d ON d.run_id = r.run_id
JOIN n3_acceptance a ON a.run_id = r.run_id
WHERE r.run_id = 'run_2026_10_31_09';
-- -> sha256:9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08`,
  },
};

interface AtomaState {
  activePage: PageId;
  setActivePage: (page: PageId) => void;

  // Authenticated Cloud Workspace state
  isAuthenticated: boolean;
  setIsAuthenticated: (auth: boolean) => void;
  userName: string;
  setUserName: (name: string) => void;
  userHandle: string;
  setUserHandle: (handle: string) => void;
  workspaceName: string;
  setWorkspaceName: (name: string) => void;
  cloudTab: CloudWorkspaceTab;
  setCloudTab: (tab: CloudWorkspaceTab) => void;
  selectedCloudModel: string;
  setSelectedCloudModel: (modelId: string) => void;

  // Research Hub -> Workspace Scaffolding -> Swarm Decomposition state
  projectIdea: string;
  setProjectIdea: (idea: string) => void;
  uploadedDocs: UploadedDocItem[];
  addUploadedDoc: (doc: UploadedDocItem) => void;
  workspaceScaffolded: boolean;
  scaffoldWorkspaceFromIdea: (idea: string) => void;
  researchChatTurns: ChatTurn[];
  addResearchChatTurn: (prompt: string) => void;
  ideChatTurns: ChatTurn[];
  addIdeChatTurn: (prompt: string) => void;
  swarmTasks: SwarmTaskPacket[];
  decomposeAndAssignToSwarm: () => void;
  swarmDecomposedBanner: boolean;
  setSwarmDecomposedBanner: (val: boolean) => void;

  activeSimTab: SimulationTabId;
  setActiveSimTab: (tab: SimulationTabId) => void;

  commandPaletteOpen: boolean;
  setCommandPaletteOpen: (open: boolean) => void;

  referenceModalOpen: boolean;
  setReferenceModalOpen: (open: boolean) => void;

  flareColorBy: FlareColorBy;
  setFlareColorBy: (mode: FlareColorBy) => void;
  flareDrawAs: FlareDrawAs;
  setFlareDrawAs: (mode: FlareDrawAs) => void;
  flareSelectedNode: string;
  setFlareSelectedNode: (nodeId: string) => void;
  flareHoveredNode: string | null;
  setFlareHoveredNode: (nodeId: string | null) => void;
  flareBannerDismissed: boolean;
  setFlareBannerDismissed: (dismissed: boolean) => void;
  flareZoom: number;
  setFlareZoom: (zoom: number) => void;

  orbitalRunning: boolean;
  setOrbitalRunning: (running: boolean) => void;
  orbitalSelectedNode: string;
  setOrbitalSelectedNode: (nodeId: string) => void;
  orbitalStepIndex: number;
  advanceOrbitalStep: () => void;

  runbookActiveStep: RunbookNodeData['id'];
  setRunbookActiveStep: (id: RunbookNodeData['id']) => void;
  runbookExecuting: boolean;
  triggerRunbookLoop: () => void;
  runbookCompletedIds: RunbookNodeData['id'][];

  graphColorScheme: 'Galaxy ✨' | 'Nebula ✨' | 'Obsidian Vault' | 'Neural Cortex';
  setGraphColorScheme: (scheme: 'Galaxy ✨' | 'Nebula ✨' | 'Obsidian Vault' | 'Neural Cortex') => void;
  graphSizeMode: 'PageRank' | 'Inbound links' | 'Betweenness';
  setGraphSizeMode: (mode: 'PageRank' | 'Inbound links' | 'Betweenness') => void;
  graphNodeSize: number;
  setGraphNodeSize: (val: number) => void;
  graphShowLinks: boolean;
  setGraphShowLinks: (show: boolean) => void;
  graphLinkThickness: number;
  setGraphLinkThickness: (val: number) => void;
  graphLinkOpacity: number;
  setGraphLinkOpacity: (val: number) => void;
  graphRepulsion: number;
  setGraphRepulsion: (val: number) => void;
  graphLinkLength: number;
  setGraphLinkLength: (val: number) => void;
  graphPullToCenter: number;
  setGraphPullToCenter: (val: number) => void;
  graphCloudSeed: number;
  reformGraphCloud: () => void;
}

export const useAtomaStore = create<AtomaState>((set, get) => ({
  activePage: 'platform',
  setActivePage: (activePage) => set({ activePage }),

  isAuthenticated: true,
  setIsAuthenticated: (isAuthenticated) => set({ isAuthenticated }),
  userName: 'Stewart Barteau',
  setUserName: (userName) => set({ userName }),
  userHandle: 'SB',
  setUserHandle: (userHandle) => set({ userHandle }),
  workspaceName: 'N³ runbook study',
  setWorkspaceName: (workspaceName) => set({ workspaceName }),
  cloudTab: 'code',
  setCloudTab: (cloudTab) => set({ cloudTab }),
  selectedCloudModel: 'atoma-n3-70b',
  setSelectedCloudModel: (selectedCloudModel) => set({ selectedCloudModel }),

  projectIdea:
    'Refine the N³ runbook validator and context retention loop without weakening the human-review gate. Ensure stable event IDs and deduplication before memory ingestion.',
  setProjectIdea: (projectIdea) => set({ projectIdea }),
  uploadedDocs: INITIAL_UPLOADED_DOCS,
  addUploadedDoc: (doc) =>
    set((state) => ({ uploadedDocs: [doc, ...state.uploadedDocs] })),
  workspaceScaffolded: true,
  scaffoldWorkspaceFromIdea: (idea) => {
    const model =
      CLOUD_MODELS.find((m) => m.id === get().selectedCloudModel)?.name ||
      'Atoma-N3-70B-SpecOps';
    set((state) => ({
      projectIdea: idea || state.projectIdea,
      workspaceScaffolded: true,
      cloudTab: 'research',
      researchChatTurns: [
        ...state.researchChatTurns,
        {
          id: `turn-${Date.now()}-u`,
          sender: 'user',
          authorLabel: 'You',
          timestamp: 'Just now',
          text: `Scaffold workspace and pull reference evidence for: "${idea}"`,
        },
        {
          id: `turn-${Date.now()}-a`,
          sender: 'assistant',
          authorLabel: 'Research Assistant',
          timestamp: 'Just now',
          modelUsed: model,
          text: `I've scaffolded the research worktree and indexed ${state.uploadedDocs.length} reference documents into the N³ relational graph. Before we decompose this into swarm packets, let's verify the three core invariants:`,
          bullets: [
            '1. Objective retention R = |O_T ∩ O_0| / |O_0| pinned across context compaction boundaries.',
            '2. Deterministic event deduplication key (source + event_type) enforced prior to N³ SQLite ingestion.',
            '3. Consequential file merges remain gated behind an explicit human-review signature.',
          ],
        },
      ],
    }));
  },

  researchChatTurns: [
    {
      id: 'res-1',
      sender: 'user',
      authorLabel: 'You',
      timestamp: '10:24 AM',
      text: 'What would falsify this claim?',
    },
    {
      id: 'res-2',
      sender: 'assistant',
      authorLabel: 'Assistant',
      timestamp: '10:24 AM',
      modelUsed: 'Atoma-N3-70B-SpecOps',
      text: 'A useful test separates retained objectives from fluent output. To falsify the claim, we should show cases where the agent produces fluent, coherent output while failing to retain key objectives from earlier in the workflow.',
      bullets: [
        'Use a fixed set of tasks and initial objectives (O_0) across conditions with identical evaluation criteria.',
        'Compare both with and without context compaction (or different retrieval strategies) on the same workload.',
        'Inspect final outputs and intermediate traces for missing, altered or replaced objectives.',
      ],
    },
  ],

  addResearchChatTurn: (prompt) => {
    const model =
      CLOUD_MODELS.find((m) => m.id === get().selectedCloudModel)?.name ||
      'Atoma-N3-70B-SpecOps';
    set((state) => ({
      researchChatTurns: [
        ...state.researchChatTurns,
        {
          id: `r-u-${Date.now()}`,
          sender: 'user',
          authorLabel: 'You',
          timestamp: 'Now',
          text: prompt,
        },
        {
          id: `r-a-${Date.now()}`,
          sender: 'assistant',
          authorLabel: 'Assistant',
          timestamp: 'Now',
          modelUsed: model,
          text: `Grounded against context-retention-paper.md (§2, Eq. 1) and design-contract.md (§2.4): I have updated the runbook specification and pre-registration criteria. Ready to decompose into 6 dependency-ordered swarm packets whenever you approve.`,
          bullets: [
            'Packet 01–02: Coordinator validates scope & emits worktree DAG.',
            'Packet 03: Researcher-02 links 12 manuscript & code citations.',
            'Packet 04–06: Builder-01 patches src/lib/runbook.rs while Verifier-03 & Critic enforce the human-review gate.',
          ],
        },
      ],
    }));
  },

  ideChatTurns: [
    {
      id: 'ide-1',
      sender: 'user',
      authorLabel: 'You',
      timestamp: '10:24 AM',
      text: 'Refine the runbook validator without weakening the review gate. Ensure stable event IDs and add deduplication before memory ingestion.',
    },
    {
      id: 'ide-2',
      sender: 'assistant',
      authorLabel: 'Builder-01',
      timestamp: '10:24 AM',
      modelUsed: 'Atoma-N3-70B-SpecOps',
      text: "I'll update the validator, add stable event IDs and deduplication, and keep the human-review gate intact. I'll run tests and share a patch for review.",
      codePatch: {
        file: 'runbook.rs',
        added: 42,
        removed: 3,
        snippet: `36 + // Preserve human-review gate: required for all runbooks
37 + if !runbook.requires_human_review() {
38 +     errors.push(ValidationError::new(
39 +         "runbook must include a human review gate"
40 +     ));
41 + }`,
      },
    },
    {
      id: 'ide-3',
      sender: 'assistant',
      authorLabel: 'Builder-01',
      timestamp: '10:26 AM',
      modelUsed: 'Atoma-N3-70B-SpecOps',
      text: 'Tests are passing. Event IDs are now stable and deduped before ingestion. The human-review gate remains required.',
    },
  ],

  addIdeChatTurn: (prompt) => {
    const model =
      CLOUD_MODELS.find((m) => m.id === get().selectedCloudModel)?.name ||
      'Atoma-N3-70B-SpecOps';
    set((state) => ({
      ideChatTurns: [
        ...state.ideChatTurns,
        {
          id: `ide-u-${Date.now()}`,
          sender: 'user',
          authorLabel: 'You',
          timestamp: 'Now',
          text: prompt,
        },
        {
          id: `ide-a-${Date.now()}`,
          sender: 'assistant',
          authorLabel: 'Builder-01',
          timestamp: 'Now',
          modelUsed: model,
          text: `Executed via ${model} in isolated seatbelt Hole. Patch staged on runbook-review worktree and verified against tests/acceptance.rs (12/12 passed).`,
          codePatch: {
            file: 'runbook.rs',
            added: 18,
            removed: 2,
            snippet: `44 + pub fn dedup_key(&self) -> String {
45 +     format!("{}:{}", self.source, self.event_type)
46 + }`,
          },
        },
      ],
    }));
  },

  swarmTasks: INITIAL_SWARM_TASKS,
  swarmDecomposedBanner: false,
  setSwarmDecomposedBanner: (swarmDecomposedBanner) => set({ swarmDecomposedBanner }),

  decomposeAndAssignToSwarm: () => {
    const idea = get().projectIdea;
    const docsCount = get().uploadedDocs.length;
    const newPackets: SwarmTaskPacket[] = [
      {
        id: 'pkt-01',
        title: `Scope & Contract: ${idea.slice(0, 48)}...`,
        agent: 'Coordinator',
        targetFile: 'docs/spec.md',
        status: 'completed',
        dependencies: [],
        tokens: '3.8K',
        duration: '0.9s',
      },
      {
        id: 'pkt-02',
        title: `Synthesize ${docsCount} uploaded docs into N³ worktree plan`,
        agent: 'Coordinator',
        targetFile: 'docs/plan.md',
        status: 'completed',
        dependencies: ['pkt-01'],
        tokens: '4.4K',
        duration: '1.2s',
      },
      {
        id: 'pkt-03',
        title: 'Extract equations, citations & retention bounds',
        agent: 'Researcher-02',
        targetFile: 'manuscripts/context-retention-paper.md',
        status: 'completed',
        dependencies: ['pkt-02'],
        tokens: '9.1K',
        duration: '2.4s',
      },
      {
        id: 'pkt-04',
        title: 'Stage deterministic validator & dedup_key patch in Hole',
        agent: 'Builder-01',
        targetFile: 'src/lib/runbook.rs',
        status: 'running',
        dependencies: ['pkt-02', 'pkt-03'],
        tokens: '14.2K',
        duration: '3.1s',
      },
      {
        id: 'pkt-05',
        title: 'Run 14 acceptance & property tests on staged patch',
        agent: 'Verifier-03',
        targetFile: 'tests/acceptance.rs',
        status: 'running',
        dependencies: ['pkt-04'],
        tokens: '6.0K',
        duration: '1.8s',
      },
      {
        id: 'pkt-06',
        title: 'Enforce Human Review Gate & seal N³ SQLite receipt',
        agent: 'Critic',
        targetFile: 'src/lib/receipt.rs',
        status: 'waiting',
        dependencies: ['pkt-05'],
        tokens: '3.1K',
        duration: 'Gate active',
      },
    ];
    set({
      swarmTasks: newPackets,
      swarmDecomposedBanner: true,
      cloudTab: 'swarm',
    });
  },

  activeSimTab: 'flare-dag',
  setActiveSimTab: (activeSimTab) => set({ activeSimTab }),

  commandPaletteOpen: false,
  setCommandPaletteOpen: (commandPaletteOpen) => set({ commandPaletteOpen }),

  referenceModalOpen: false,
  setReferenceModalOpen: (referenceModalOpen) => set({ referenceModalOpen }),

  flareColorBy: 'Activity',
  setFlareColorBy: (flareColorBy) => set({ flareColorBy }),
  flareDrawAs: 'Canvas',
  setFlareDrawAs: (flareDrawAs) => set({ flareDrawAs }),
  flareSelectedNode: 'insights.ts',
  setFlareSelectedNode: (flareSelectedNode) => set({ flareSelectedNode }),
  flareHoveredNode: null,
  setFlareHoveredNode: (flareHoveredNode) => set({ flareHoveredNode }),
  flareBannerDismissed: false,
  setFlareBannerDismissed: (flareBannerDismissed) => set({ flareBannerDismissed }),
  flareZoom: 72,
  setFlareZoom: (flareZoom) => set({ flareZoom }),

  orbitalRunning: true,
  setOrbitalRunning: (orbitalRunning) => set({ orbitalRunning }),
  orbitalSelectedNode: 'coder-02',
  setOrbitalSelectedNode: (orbitalSelectedNode) => set({ orbitalSelectedNode }),
  orbitalStepIndex: 0,
  advanceOrbitalStep: () =>
    set((s) => ({ orbitalStepIndex: (s.orbitalStepIndex + 1) % 9 })),

  runbookActiveStep: 'execute',
  setRunbookActiveStep: (runbookActiveStep) => set({ runbookActiveStep }),
  runbookExecuting: false,
  runbookCompletedIds: ['scope', 'plan', 'execute', 'test', 'review', 'receipt'],
  triggerRunbookLoop: () => {
    const order: RunbookNodeData['id'][] = [
      'scope',
      'plan',
      'execute',
      'test',
      'review',
      'receipt',
    ];
    set({ runbookExecuting: true, runbookCompletedIds: [], runbookActiveStep: 'scope' });
    order.forEach((stepId, idx) => {
      setTimeout(() => {
        set((state) => ({
          runbookActiveStep: stepId,
          runbookCompletedIds: [...state.runbookCompletedIds, stepId],
          runbookExecuting: idx < order.length - 1,
        }));
      }, (idx + 1) * 550);
    });
  },

  graphColorScheme: 'Galaxy ✨',
  setGraphColorScheme: (graphColorScheme) => set({ graphColorScheme }),
  graphSizeMode: 'PageRank',
  setGraphSizeMode: (graphSizeMode) => set({ graphSizeMode }),
  graphNodeSize: 0.5,
  setGraphNodeSize: (graphNodeSize) => set({ graphNodeSize }),
  graphShowLinks: true,
  setGraphShowLinks: (graphShowLinks) => set({ graphShowLinks }),
  graphLinkThickness: 0.8,
  setGraphLinkThickness: (graphLinkThickness) => set({ graphLinkThickness }),
  graphLinkOpacity: 0.14,
  setGraphLinkOpacity: (graphLinkOpacity) => set({ graphLinkOpacity }),
  graphRepulsion: 9,
  setGraphRepulsion: (graphRepulsion) => set({ graphRepulsion }),
  graphLinkLength: 195,
  setGraphLinkLength: (graphLinkLength) => set({ graphLinkLength }),
  graphPullToCenter: 0.245,
  setGraphPullToCenter: (graphPullToCenter) => set({ graphPullToCenter }),
  graphCloudSeed: 1,
  reformGraphCloud: () => set((s) => ({ graphCloudSeed: s.graphCloudSeed + 1 })),
}));
