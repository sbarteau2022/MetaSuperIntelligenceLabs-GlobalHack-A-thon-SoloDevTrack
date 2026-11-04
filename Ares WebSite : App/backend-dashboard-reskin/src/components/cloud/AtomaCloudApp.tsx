import React, { Suspense, lazy, useState } from 'react';
import {
  useAtomaStore,
  CLOUD_MODELS,
  type CloudWorkspaceTab,
} from '../../store/useAtomaStore';
import { AtomaAgentBadge, AtomaLogoMark } from '../brand/AtomaBrandAssets';
import {
  AlertTriangle,
  ArrowRight,
  CheckCircle2,
  ChevronDown,
  Code2,
  Command,
  Cpu,
  Database,
  FileText,
  Folder,
  GitBranch,
  Layers,
  Play,
  Search,
  Sparkles,
  Terminal,
  Workflow,
} from '../ui/Icons';

// Cloud tabs are per-tab chunks — 'code' stays inline (default tab), the
// rest (Swarm/Mind/Research/Tools) load only when the rail is clicked.
const CloudSwarmView = lazy(() =>
  import('./CloudSwarmAndMindView').then((m) => ({ default: m.CloudSwarmView })),
);
const CloudMindGraphView = lazy(() =>
  import('./CloudSwarmAndMindView').then((m) => ({ default: m.CloudMindGraphView })),
);
const CloudResearchHubView = lazy(() =>
  import('./CloudResearchHubView').then((m) => ({ default: m.CloudResearchHubView })),
);
const ToolsMcpPage = lazy(() =>
  import('../pages/AtomaPages').then((m) => ({ default: m.ToolsMcpPage })),
);

function CloudTabFallback() {
  return (
    <div
      className="flex-1 flex flex-col items-center justify-center gap-3 bg-[#26262b] min-h-[420px]"
      aria-busy="true"
      aria-label="Loading workspace tab"
    >
      <div className="h-2.5 w-48 rounded-full bg-white/10 animate-pulse" />
      <div className="h-2.5 w-32 rounded-full bg-white/8 animate-pulse" />
      <div className="font-mono text-[10px] tracking-[0.2em] text-[#71717a] uppercase">
        Loading workspace…
      </div>
    </div>
  );
}

/**
 * Byte-for-byte implementation of 08-ide-chat-project-dag-v2-3.png ("Code" tab)
 * + Unified Authenticated Cloud Workspace Header & Left Rail switching across:
 *   Code (`</>`) | Swarm (`☍`) | Mind (`❖`) | Research (`📖`) | Tools (`🛠`) | Settings (`⚙`)
 */
const CloudCodeIdeView: React.FC = () => {
  const {
    ideChatTurns,
    addIdeChatTurn,
    selectedCloudModel,
    setSelectedCloudModel,
    setCloudTab,
  } = useAtomaStore();

  const [selectedFile, setSelectedFile] = useState('runbook.rs');
  const [selectedAgentLane, setSelectedAgentLane] = useState<'Builder' | 'Researcher' | 'Verifier'>(
    'Builder'
  );
  const [promptInput, setPromptInput] = useState('');
  const [reviewBannerOpen, setReviewBannerOpen] = useState(true);
  const [colorBy, setColorBy] = useState<'Activity' | 'Risk' | 'Tests'>('Activity');

  const fileContents: Record<string, string> = {
    'runbook.rs': `12   use crate::validation::{ValidationError, Result};
13   use crate::contract::DesignContract;
14   use crate::review::HumanReviewGate;
15
16   pub fn validate_runbook(runbook: &Runbook, contract: &DesignContract) -> Result<Vec<ValidationError>> {
17       let mut errors = Vec::new();
18 +     // 1. Basic structure and metadata
19 +     if runbook.steps.is_empty() {
20 +         errors.push(ValidationError::new("runbook must define at least one step"));
21 +     }
22       // 2. Verify each step against the contract
23       for (idx, step) in runbook.steps.iter().enumerate() {
24           if let Err(e) = validate_step(step, contract) {
25               errors.push(e.with_context(format!("step {}", idx + 1)));
26           }
27       }
28 +     // 3. Human review gate required for all runbooks
29 +     if !runbook.requires_human_review() {
30 +         errors.push(ValidationError::new("runbook must include a human review gate"));
31 +     }
32       if errors.is_empty() {
33           Ok(vec![])
34       } else {
35           Err(errors)
36       }
37   }`,
    'acceptance.rs': `01   use crate::runbook::{Runbook, validate_runbook};
02   use crate::contract::DesignContract;
03
04   #[test]
05   fn enforces_human_review_gate_on_consequential_changes() {
06       let contract = DesignContract::default_strict();
07       let runbook = Runbook::fixture_with_dedup_events();
08       assert!(validate_runbook(&runbook, &contract).is_ok());
09   }`,
    'receipt.rs': `01   use sha2::{Sha256, Digest};
02
03   pub struct N3Receipt {
04       pub run_id: String,
05       pub sha256_hash: String,
06       pub human_reviewed: bool,
07   }`,
  };

  const handleSend = (e: React.FormEvent) => {
    e.preventDefault();
    if (!promptInput.trim()) return;
    addIdeChatTurn(promptInput);
    setPromptInput('');
  };

  return (
    <div className="grid grid-cols-1 xl:grid-cols-[235px_1fr_390px] flex-1 min-h-0 bg-[#26262b] text-[#ffffff] select-none">
      {/* LEFT COLUMN: runbook-study Worktree + Agents (3) */}
      <aside className="bg-[#2e2e33] border-r border-white/8 flex flex-col justify-between p-3 space-y-4 overflow-y-auto">
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold text-white">runbook-study</span>
            <span className="text-[10px] font-mono text-[#71717a]">↕</span>
          </div>

          <div className="px-2.5 py-1.5 rounded-lg bg-[#2e2e33] border border-white/10 flex items-center justify-between text-xs font-mono text-[#ffffff]">
            <span className="flex items-center gap-1.5">
              <GitBranch className="w-3.5 h-3.5 text-[#f59e0b]" /> runbook-review
            </span>
            <ChevronDown className="w-3.5 h-3.5 text-[#64748b]" />
          </div>

          {/* File Tree matching 08-ide-chat-project-dag-v2-3.png */}
          <div className="space-y-1 font-mono text-[11px]">
            <div className="text-[#71717a] px-1">▾ 📁 docs</div>
            <div className="pl-4 space-y-0.5 text-[#71717a]">
              <div className="px-2 py-0.5">📄 design.md</div>
              <div className="px-2 py-0.5">📄 architecture.md</div>
            </div>

            <div className="text-[#71717a] px-1 pt-1">▾ 📁 src</div>
            <div className="pl-3 space-y-0.5">
              <div className="text-[#71717a] px-1">▾ 📁 lib</div>
              <div className="pl-3 space-y-0.5">
                {[
                  { name: 'runbook.rs', badge: 'M', col: 'text-[#f59e0b]' },
                  { name: 'validator.rs', badge: 'M', col: 'text-[#f59e0b]' },
                  { name: 'context.rs', badge: '', col: '' },
                  { name: 'mission.rs', badge: 'A', col: 'text-emerald-400' },
                  { name: 'receipt.rs', badge: 'M', col: 'text-[#f59e0b]' },
                ].map((f) => (
                  <button
                    key={f.name}
                    onClick={() =>
                      setSelectedFile(
                        f.name in fileContents ? f.name : 'runbook.rs'
                      )
                    }
                    className={`w-full flex items-center justify-between px-2 py-1 rounded text-left transition cursor-pointer ${
                      selectedFile === f.name
                        ? 'bg-[#f59e0b]/20 text-white border border-[#f59e0b]/40'
                        : 'text-[#ffffff] hover:bg-white/5'
                    }`}
                  >
                    <span>⚙️ {f.name}</span>
                    {f.badge && <span className={`text-[10px] ${f.col}`}>{f.badge}</span>}
                  </button>
                ))}
              </div>

              <div className="text-[#71717a] px-1 pt-1">▾ 📁 ui</div>
              <div className="pl-3 space-y-0.5 text-[#ffffff]">
                <div className="flex justify-between px-2 py-0.5">
                  <span>⚛ RunbookPanel.tsx</span>
                  <span className="text-[#f59e0b] text-[10px]">M</span>
                </div>
                <div className="px-2 py-0.5 text-[#71717a]">⚛ GraphCanvas.tsx</div>
              </div>
            </div>

            <div className="text-[#71717a] px-1 pt-1">▾ 📁 tests</div>
            <div className="pl-4 space-y-0.5">
              <button
                onClick={() => setSelectedFile('acceptance.rs')}
                className={`w-full flex items-center justify-between px-2 py-1 rounded text-left ${
                  selectedFile === 'acceptance.rs'
                    ? 'bg-[#f59e0b]/20 text-white border border-[#f59e0b]/40'
                    : 'text-[#ffffff]'
                }`}
              >
                <span>🧪 acceptance.rs</span>
                <span className="text-[#f59e0b] text-[10px]">M</span>
              </button>
              <div className="px-2 py-0.5 text-[#71717a]">🧪 runbook_tests.rs</div>
            </div>
          </div>
        </div>

        {/* Bottom Agents (3) Roster with Sparklines */}
        <div className="pt-3 border-t border-white/8 space-y-2">
          <div className="flex items-center justify-between text-xs">
            <span className="font-semibold text-white">Agents (3)</span>
            <span className="text-[#71717a]">+</span>
          </div>
          {[
            {
              name: 'Builder-01',
              task: 'Implement runbook...',
              pct: '72%',
              tokens: '12.4K tokens',
              dot: 'bg-emerald-400',
              stroke: '#f59e0b',
            },
            {
              name: 'Researcher-02',
              task: 'Validate contracts',
              pct: '41%',
              tokens: '8.7K tokens',
              dot: 'bg-[#f59e0b]',
              stroke: '#71717a',
            },
            {
              name: 'Verifier-03',
              task: 'Prepare review',
              pct: '18%',
              tokens: '5.1K tokens',
              dot: 'bg-zinc-400',
              stroke: '#71717a',
            },
          ].map((ag) => (
            <div
              key={ag.name}
              className="p-2.5 rounded-xl bg-[#2e2e33] border border-white/8 space-y-1"
            >
              <div className="flex items-center justify-between text-xs">
                <span className="font-semibold text-white flex items-center gap-1.5">
                  <span className={`w-2 h-2 rounded-full ${ag.dot}`} />
                  {ag.name}
                </span>
                <span className="font-mono text-[#f59e0b] font-bold">{ag.pct}</span>
              </div>
              <div className="text-[10px] text-[#71717a]">{ag.task}</div>
              <div className="flex items-center justify-between pt-1">
                <svg viewBox="0 0 80 18" className="w-16 h-4">
                  <path
                    d="M0,14 L15,10 L30,12 L45,6 L60,9 L80,3"
                    fill="none"
                    stroke={ag.stroke}
                    strokeWidth="1.5"
                  />
                </svg>
                <span className="font-mono text-[9px] text-[#71717a]">{ag.tokens}</span>
              </div>
            </div>
          ))}
        </div>
      </aside>

      {/* CENTER COLUMN: Project DAG / Dependency Graph + Rust Diff Code Editor + Terminal */}
      <div className="flex flex-col min-w-0 bg-[#2e2e33]">
        {/* Top Graph Sub-Tabs */}
        <div className="flex items-center justify-between px-4 py-2 border-b border-white/8 bg-[#2e2e33] text-xs">
          <div className="flex items-center gap-4">
            <span className="text-[#71717a]">Project DAG</span>
            <span className="text-white font-semibold border-b-2 border-[#f59e0b] pb-1">
              Dependency graph
            </span>
            <span className="text-[#71717a]">Run timeline</span>
          </div>
          <span className="font-mono text-[11px] text-[#71717a]">»</span>
        </div>

        {/* Amber Review Required Banner */}
        {reviewBannerOpen && (
          <div className="mx-3 mt-2.5 px-3.5 py-2 rounded-lg bg-[#231b0c] border border-amber-500/40 flex flex-wrap items-center justify-between gap-2 text-xs">
            <div className="flex items-center gap-2">
              <AlertTriangle className="w-4 h-4 text-amber-400 shrink-0" />
              <div>
                <div className="font-semibold text-amber-300">
                  3 changed files · Review required
                </div>
                <div className="text-[11px] text-amber-200/80">
                  Changes affect 5 dependent files across 3 modules
                </div>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={() => setSelectedFile('runbook.rs')}
                className="px-3 py-1 rounded bg-amber-400 hover:bg-amber-300 text-black font-semibold text-xs cursor-pointer"
              >
                Review changes →
              </button>
              <button
                onClick={() => setReviewBannerOpen(false)}
                className="px-2.5 py-1 rounded bg-white/10 text-[#ffffff] text-xs cursor-pointer"
              >
                Dismiss
              </button>
            </div>
          </div>
        )}

        {/* 4-Column Module Dependency Graph + 6-Stage Runbook Strip (Byte-for-Byte 08-ide-chat-project-dag-v2-3.png) */}
        <div className="p-3 linear-dot-bg border-b border-white/8 space-y-3">
          <div className="flex items-center justify-between text-[11px]">
            <div className="flex items-center gap-3 font-mono text-[10px] text-[#71717a]">
              <span>
                <i className="inline-block w-2 h-2 rounded-full bg-amber-400 mr-1" />
                Changed files (3)
              </span>
              <span>
                <i className="inline-block w-2 h-2 rounded-full bg-amber-500/60 mr-1" />
                Affected files (5)
              </span>
              <span>
                <i className="inline-block w-2 h-2 rounded-full bg-[#f59e0b] mr-1" />
                Selected
              </span>
            </div>

            <div className="flex items-center gap-1 bg-[#0e1420] p-0.5 rounded border border-white/10">
              <span className="px-2 text-[10px] text-[#71717a]">Color by:</span>
              {(['Activity', 'Risk', 'Tests'] as const).map((m) => (
                <button
                  key={m}
                  onClick={() => setColorBy(m)}
                  className={`px-2 py-0.5 rounded text-[10px] font-medium cursor-pointer ${
                    colorBy === m ? 'bg-[#f59e0b] text-white' : 'text-[#71717a]'
                  }`}
                >
                  {m}
                </button>
              ))}
            </div>
          </div>

          {/* 4 Module Columns: core (6) -> runbook (4) -> ui (5) -> adapters (6) */}
          <div className="grid grid-cols-4 gap-3 font-mono text-[11px]">
            {[
              {
                title: 'core (6)',
                color: 'text-emerald-400',
                files: [
                  { name: 'context.rs', state: 'normal' },
                  { name: 'types.rs', state: 'normal' },
                  { name: 'mission.rs', state: 'amber' },
                  { name: 'events.rs', state: 'normal' },
                  { name: 'store.rs', state: 'normal' },
                  { name: 'pressure.rs', state: 'amber' },
                ],
              },
              {
                title: 'runbook (4)',
                color: 'text-amber-400',
                files: [
                  { name: 'validator.rs', state: 'amber' },
                  { name: 'runbook.rs', state: 'selected' },
                  { name: 'receipt.rs', state: 'amber' },
                  { name: 'acceptance.rs', state: 'amber' },
                ],
              },
              {
                title: 'ui (5)',
                color: 'text-[#f59e0b]',
                files: [
                  { name: 'RunbookPanel.tsx', state: 'normal' },
                  { name: 'GraphCanvas.tsx', state: 'normal' },
                  { name: 'workflow.ts', state: 'normal' },
                  { name: 'Timeline.tsx', state: 'normal' },
                  { name: 'DetailsPanel.tsx', state: 'normal' },
                ],
              },
              {
                title: 'adapters (6)',
                color: 'text-emerald-400',
                files: [
                  { name: 'api.rs', state: 'normal' },
                  { name: 'executor.rs', state: 'normal' },
                  { name: 'scheduler.rs', state: 'normal' },
                  { name: 'monitor.rs', state: 'normal' },
                  { name: 'integrations.rs', state: 'normal' },
                  { name: 'metrics.rs', state: 'normal' },
                ],
              },
            ].map((col) => (
              <div
                key={col.title}
                className="p-2.5 rounded-xl bg-[#0a101b]/90 border border-white/10 space-y-1.5"
              >
                <div className={`text-[10px] font-semibold ${col.color}`}>● {col.title}</div>
                {col.files.map((f) => (
                  <button
                    key={f.name}
                    onClick={() =>
                      setSelectedFile(
                        f.name in fileContents ? f.name : 'runbook.rs'
                      )
                    }
                    className={`w-full px-2 py-1 rounded text-left border text-[10px] truncate transition cursor-pointer ${
                      f.state === 'selected'
                        ? 'bg-[#34343a] border-[#f59e0b] text-white shadow-none'
                        : f.state === 'amber'
                        ? 'bg-amber-500/15 border-amber-400/60 text-amber-200'
                        : 'bg-[#0d1422] border-white/8 text-[#ffffff]'
                    }`}
                  >
                    {f.name}
                  </button>
                ))}
              </div>
            ))}
          </div>

          {/* 6-Stage Runbook Progress Strip */}
          <div className="grid grid-cols-6 gap-2 pt-1 font-mono text-[10px]">
            {[
              { name: 'Scope', sub: 'Complete', state: 'done' },
              { name: 'Plan', sub: 'Complete', state: 'done' },
              { name: 'Implement', sub: 'In progress', state: 'active' },
              { name: 'Verify', sub: 'Pending', state: 'pending' },
              { name: 'Human review', sub: 'Waiting', state: 'waiting' },
              { name: 'Receipt', sub: 'Pending', state: 'pending' },
            ].map((st) => (
              <div
                key={st.name}
                className={`p-2 rounded-lg border ${
                  st.state === 'active'
                    ? 'bg-[#34343a] border-[#f59e0b] text-white'
                    : st.state === 'done'
                    ? 'bg-[#0b1320] border-emerald-500/40 text-white'
                    : st.state === 'waiting'
                    ? 'bg-[#1b160c] border-amber-500/40 text-amber-200'
                    : 'bg-[#2e2e33] border-white/10 text-[#71717a]'
                }`}
              >
                <div className="font-semibold">{st.name}</div>
                <div className="text-[9px] opacity-80">{st.sub}</div>
              </div>
            ))}
          </div>
        </div>

        {/* Center Code Diff Editor with Tabs */}
        <div className="flex-1 flex flex-col bg-[#26262b] min-h-[210px]">
          <div className="flex items-center gap-1 px-3 pt-1.5 border-b border-white/8 bg-[#2e2e33] font-mono text-xs">
            {(['runbook.rs', 'acceptance.rs', 'receipt.rs'] as const).map((tab) => (
              <button
                key={tab}
                onClick={() => setSelectedFile(tab)}
                className={`px-3 py-1.5 rounded-t-md flex items-center gap-1.5 cursor-pointer ${
                  selectedFile === tab
                    ? 'bg-[#26262b] text-white border-t border-x border-white/12'
                    : 'text-[#71717a] hover:text-white'
                }`}
              >
                <span>{tab}</span>
                <span className="text-[#f59e0b] text-[10px]">M</span>
              </button>
            ))}
          </div>

          <div className="px-4 py-1 border-b border-white/6 text-[11px] font-mono text-[#64748b]">
            src › runbook.rs › validate_runbook
          </div>

          <pre className="p-4 font-mono text-[11px] leading-relaxed overflow-x-auto flex-1">
            {(fileContents[selectedFile] || fileContents['runbook.rs'])
              .split('\n')
              .map((line, idx) => {
                const isAdded = line.includes(' + ');
                return (
                  <div
                    key={idx}
                    className={`px-2 rounded ${
                      isAdded
                        ? 'bg-emerald-500/15 text-emerald-200 border-l-2 border-emerald-400'
                        : 'text-[#ffffff]'
                    }`}
                  >
                    {line}
                  </div>
                );
              })}
          </pre>
        </div>

        {/* Bottom Terminal & CPU/RAM/Tokens Bar */}
        <div className="border-t border-white/10 bg-[#2e2e33] p-3 grid grid-cols-1 lg:grid-cols-[1fr_240px] gap-4 font-mono text-[11px]">
          <div className="space-y-1">
            <div className="flex items-center gap-3 text-xs border-b border-white/8 pb-1 mb-1">
              <span className="text-[#f59e0b] font-semibold">Terminal</span>
              <span className="text-[#71717a]">Checks</span>
              <span className="text-[#71717a]">Telemetry</span>
              <span className="text-[#71717a]">Run output</span>
            </div>
            <div className="text-[#ffffff]">$ cargo check</div>
            <div className="text-emerald-400">
              Checking atoma-runbook v0.2.1 (/workspace/atoma) · Finished in 1.24s
            </div>
            <div className="text-[#ffffff]">$ cargo test validate_runbook</div>
            <div className="text-emerald-400">
              test result: ok. 12 passed; 0 failed; 0 ignored (0.31s)
            </div>
          </div>

          <div className="grid grid-cols-3 gap-2 items-center border-l border-white/8 pl-3">
            <div>
              <div className="text-[10px] text-[#71717a]">CPU</div>
              <div className="text-xs font-bold text-white">28%</div>
            </div>
            <div>
              <div className="text-[10px] text-[#71717a]">RAM</div>
              <div className="text-xs font-bold text-zinc-300">1.2GB</div>
            </div>
            <div>
              <div className="text-[10px] text-[#71717a]">Tokens</div>
              <div className="text-xs font-bold text-emerald-300">12.4K</div>
            </div>
          </div>
        </div>
      </div>

      {/* RIGHT COLUMN: AGENT COMMAND Chat (Byte-for-Byte 08-ide-chat-project-dag-v2-3.png) */}
      <aside className="bg-[#2e2e33] border-l border-white/8 flex flex-col justify-between min-h-0">
        <div>
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-white/8">
            <span className="font-mono text-xs font-semibold tracking-wider text-white flex items-center gap-2">
              <AtomaLogoMark className="w-4 h-4" glow={false} /> AGENT COMMAND
            </span>
            <span className="px-2 py-0.5 rounded bg-[#f59e0b]/20 border border-[#f59e0b]/40 text-[#f59e0b] font-mono text-[10px]">
              Context 3
            </span>
          </div>

          {/* 3 Agent Lane Tabs: Builder 3 | Researcher 1 | Verifier 1 */}
          <div className="grid grid-cols-3 border-b border-white/8 text-xs bg-[#2e2e33]">
            {(
              [
                { id: 'Builder', count: 3 },
                { id: 'Researcher', count: 1 },
                { id: 'Verifier', count: 1 },
              ] as const
            ).map((lane) => (
              <button
                key={lane.id}
                onClick={() => setSelectedAgentLane(lane.id)}
                className={`py-2 font-medium flex items-center justify-center gap-1.5 border-b-2 transition cursor-pointer ${
                  selectedAgentLane === lane.id
                    ? 'border-[#f59e0b] text-white bg-[#34343a]'
                    : 'border-transparent text-[#71717a] hover:text-white'
                }`}
              >
                <span>{lane.id}</span>
                <span className="px-1.5 py-0.2 rounded-full bg-[#f59e0b]/25 text-[#f59e0b] font-mono text-[10px]">
                  {lane.count}
                </span>
              </button>
            ))}
          </div>
        </div>

        {/* Chat Stream */}
        <div className="flex-1 overflow-y-auto p-4 space-y-4 text-xs">
          {ideChatTurns.map((turn) => (
            <div key={turn.id} className="space-y-2">
              <div className="flex items-center gap-2">
                {turn.sender === 'user' ? (
                  <span className="w-6 h-6 rounded-full bg-[#1e293b] border border-white/15 flex items-center justify-center font-mono text-[10px] font-bold text-white">
                    SB
                  </span>
                ) : (
                  <AtomaAgentBadge />
                )}
                <span className="font-semibold text-white">{turn.authorLabel}</span>
                <span className="text-[10px] font-mono text-[#64748b]">{turn.timestamp}</span>
              </div>

              <div className="pl-8 space-y-2.5">
                <p className="text-[#ffffff] leading-relaxed">{turn.text}</p>

                {turn.codePatch && (
                  <div className="rounded-xl bg-[#2e2e33] border border-white/12 p-3 space-y-2">
                    <div className="flex items-center justify-between text-[11px] font-mono">
                      <span className="text-white font-semibold">{turn.codePatch.file}</span>
                      <span className="text-emerald-400">+{turn.codePatch.added}</span>
                      <span className="text-rose-400">-{turn.codePatch.removed}</span>
                      <span className="text-amber-300">● Review required</span>
                    </div>
                    <pre className="p-2 rounded bg-[#26262b] text-emerald-300 font-mono text-[10px] overflow-x-auto leading-relaxed">
                      {turn.codePatch.snippet}
                    </pre>
                    <div className="grid grid-cols-2 gap-2 pt-1">
                      <button
                        onClick={() => setSelectedFile('runbook.rs')}
                        className="py-1.5 px-3 rounded-lg bg-[#f59e0b] hover:bg-[#d97706] text-white font-semibold text-xs cursor-pointer"
                      >
                        Review changes
                      </button>
                      <button
                        onClick={() => setSelectedFile('runbook.rs')}
                        className="py-1.5 px-3 rounded-lg bg-white/6 hover:bg-white/12 border border-white/10 text-white text-xs cursor-pointer"
                      >
                        Open in editor
                      </button>
                    </div>
                  </div>
                )}
              </div>
            </div>
          ))}

          {/* Tool activity 5 steps */}
          <div className="rounded-xl bg-[#2e2e33] border border-white/10 p-3 space-y-1.5 font-mono text-[11px]">
            <div className="text-white font-sans font-semibold text-xs">
              ▾ Tool activity <span className="text-[#71717a] font-normal">5 steps</span>
            </div>
            <div className="flex justify-between text-[#ffffff]">
              <span>✓ Read runbook.rs</span>
              <span className="text-[#64748b]">1.2s</span>
            </div>
            <div className="flex justify-between text-[#ffffff]">
              <span>✓ Inspect acceptance contract</span>
              <span className="text-[#64748b]">0.8s</span>
            </div>
            <div className="flex justify-between text-[#ffffff]">
              <span>✓ Search for existing dedup logic</span>
              <span className="text-[#64748b]">0.6s</span>
            </div>
            <div className="flex justify-between text-[#ffffff]">
              <span>✓ Prepare patch with stable event IDs</span>
              <span className="text-[#64748b]">1.4s</span>
            </div>
            <div className="flex justify-between text-emerald-300">
              <span>✓ Run tests cargo test validate_runbook</span>
              <span className="text-[#64748b]">2.1s</span>
            </div>
          </div>

          {/* Cited sources 3 */}
          <div className="space-y-1.5">
            <div className="text-[11px] text-[#71717a]">▾ Cited sources (3)</div>
            <div className="grid grid-cols-3 gap-1.5 font-mono text-[10px]">
              {[
                ['design.md', '§ Runbook validation'],
                ['acceptance.rs', '§ Human review gate'],
                ['memory.rs', '§ Stable event IDs'],
              ].map(([f, s]) => (
                <div key={f} className="p-2 rounded-lg bg-[#2e2e33] border border-white/8">
                  <div className="text-white font-semibold truncate">{f}</div>
                  <div className="text-[9px] text-[#71717a] truncate">{s}</div>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Bottom Agent Command Input Bar */}
        <form
          onSubmit={handleSend}
          className="p-3 border-t border-white/10 bg-[#26262b] space-y-2"
        >
          <input
            type="text"
            value={promptInput}
            onChange={(e) => setPromptInput(e.target.value)}
            placeholder={`Direct ${selectedAgentLane}-01 or stage patch in seatbelt Hole...`}
            className="w-full px-3 py-2 rounded-lg bg-[#2e2e33] border border-white/12 text-xs text-white focus:border-[#f59e0b] focus:outline-none"
          />
          <div className="flex items-center justify-between gap-2 text-[11px]">
            <select
              value={selectedCloudModel}
              onChange={(e) => setSelectedCloudModel(e.target.value)}
              className="bg-[#2e2e33] border border-white/12 rounded-md px-2 py-1 text-[11px] font-mono text-[#ffffff] max-w-[190px]"
            >
              {CLOUD_MODELS.map((m) => (
                <option key={m.id} value={m.id}>
                  {m.name}
                </option>
              ))}
            </select>

            <div className="flex items-center gap-1.5">
              <button
                type="button"
                onClick={() => setCloudTab('research')}
                className="px-2.5 py-1 rounded bg-white/6 hover:bg-white/12 text-[#ffffff] text-[10px] font-mono cursor-pointer"
              >
                📖 Research
              </button>
              <button
                type="submit"
                className="px-3.5 py-1 rounded-lg bg-[#f59e0b] hover:bg-[#d97706] text-white font-semibold text-xs cursor-pointer"
              >
                Send ↗
              </button>
            </div>
          </div>
        </form>
      </aside>
    </div>
  );
};

/**
 * Cloud Settings & Model Endpoints View (BEAM, oMLX Nemotron, Muse API, Atoma Cloud N³)
 */
const CloudSettingsView: React.FC = () => {
  const { selectedCloudModel, setSelectedCloudModel } = useAtomaStore();

  return (
    <div className="max-w-5xl mx-auto p-8 space-y-8 text-[#ffffff]">
      <div>
        <div className="font-mono text-xs uppercase tracking-[0.2em] text-[#f59e0b]">
          CLOUD &amp; LOCAL MODEL ENDPOINTS · SEATBELT SANDBOX
        </div>
        <h2 className="font-display text-3xl font-bold text-white mt-1">
          Model Serving &amp; Agent Lanes
        </h2>
        <p className="text-sm text-[#71717a] mt-1">
          Configure cloud frontier models and local Apple Silicon oMLX / WebGPU lanes for Builder,
          Researcher, and Verifier.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {CLOUD_MODELS.map((m) => {
          const active = selectedCloudModel === m.id;
          return (
            <div
              key={m.id}
              onClick={() => setSelectedCloudModel(m.id)}
              className={`p-5 rounded-2xl border cursor-pointer transition space-y-2 ${
                active
                  ? 'bg-[#34343a] border-[#f59e0b] shadow-none'
                  : 'bg-[#090d16] border-white/10 hover:border-white/25'
              }`}
            >
              <div className="flex items-center justify-between">
                <span className="font-mono text-sm font-bold text-white">{m.name}</span>
                <span className="px-2 py-0.5 rounded-full bg-emerald-500/15 text-emerald-300 font-mono text-[10px]">
                  ● {m.status}
                </span>
              </div>
              <div className="text-xs text-[#ffffff]">{m.provider}</div>
              <div className="text-[11px] font-mono text-[#71717a]">{m.protocol}</div>
              <div className="pt-2 border-t border-white/8 flex items-center justify-between font-mono text-[11px] text-[#f59e0b]">
                <span>{m.latency}</span>
                <span>{m.rate}</span>
                <span>{m.kvMode}</span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

/**
 * Full Authenticated Cloud Workspace Shell ("Behind the Login Wall")
 */
export const AtomaCloudApp: React.FC = () => {
  const {
    cloudTab,
    setCloudTab,
    workspaceName,
    selectedCloudModel,
    setSelectedCloudModel,
    setActivePage,
    setCommandPaletteOpen,
    userHandle,
  } = useAtomaStore();

  const activeModelObj =
    CLOUD_MODELS.find((m) => m.id === selectedCloudModel) || CLOUD_MODELS[0];

  const railItems: Array<{ id: CloudWorkspaceTab; label: string; icon: string }> = [
    { id: 'code', label: 'Code', icon: '</>' },
    { id: 'swarm', label: 'Swarm', icon: '☍' },
    { id: 'mind', label: 'Mind', icon: '❖' },
    { id: 'research', label: 'Research', icon: '📖' },
    { id: 'tools', label: 'Tools', icon: '🛠' },
    { id: 'settings', label: 'Settings', icon: '⚙' },
  ];

  return (
    <div className="min-h-screen bg-[#26262b] text-[#ffffff] flex flex-col">
      {/* Top Authenticated Cloud Workspace Bar (matches 08, 09, 10, 11 headers) */}
      <header className="h-12 px-4 bg-[#2e2e33] border-b border-white/10 flex items-center justify-between gap-4 select-none">
        <div className="flex items-center gap-3 min-w-0">
          <button
            onClick={() => setActivePage('platform')}
            className="flex items-center gap-2.5 text-left group cursor-pointer"
            title="Return to Public Atoma Marketing Site"
          >
            <AtomaLogoMark className="w-6 h-6" />
            <span className="font-display font-bold text-xs tracking-[0.18em] text-white">
              ATOMA / ARES
            </span>
          </button>
          <span className="text-white/20">|</span>
          <span className="text-xs text-[#71717a] hidden sm:inline truncate">
            ARES SpecOps Agentic Lab
          </span>
          <span className="text-white/20 hidden md:inline">/</span>
          <span className="text-xs text-[#71717a] hidden md:inline">Research workspace</span>
          <span className="text-white/20 hidden md:inline">/</span>
          <span className="text-xs font-semibold text-white truncate">{workspaceName}</span>
        </div>

        {/* Center Search / Jump Bar */}
        <button
          onClick={() => setCommandPaletteOpen(true)}
          className="hidden md:flex items-center justify-between w-80 px-3 py-1.5 rounded-lg bg-[#2e2e33] border border-white/10 text-xs text-[#71717a] hover:border-[#f59e0b]/50 cursor-pointer"
        >
          <span className="flex items-center gap-2">
            <Search className="w-3.5 h-3.5 text-[#f59e0b]" />
            <span>Search or jump to...</span>
          </span>
          <kbd className="px-1.5 py-0.2 rounded bg-white/8 font-mono text-[10px]">⌘ K</kbd>
        </button>

        {/* Right Cloud Model Serving Pill + Public Site Switcher + User Avatar */}
        <div className="flex items-center gap-2.5">
          <div className="flex items-center gap-2 px-3 py-1 rounded-lg bg-[#2e2e33] border border-[#f59e0b]/35 text-xs">
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
            <select
              value={selectedCloudModel}
              onChange={(e) => setSelectedCloudModel(e.target.value)}
              className="bg-transparent font-mono text-[11px] text-white focus:outline-none cursor-pointer"
            >
              {CLOUD_MODELS.map((m) => (
                <option key={m.id} value={m.id} className="bg-[#0b101b] text-white">
                  Cloud · {m.name} ({m.rate})
                </option>
              ))}
            </select>
          </div>

          <button
            onClick={() => setActivePage('platform')}
            className="px-2.5 py-1 rounded-lg bg-white/6 hover:bg-white/12 border border-white/10 text-[11px] font-mono text-[#ffffff] cursor-pointer"
          >
            Public Site ↗
          </button>

          <div
            title={`Signed in as Stewart Barteau (${activeModelObj.name})`}
            className="w-7 h-7 rounded-full bg-[#34343a] border border-[#f59e0b]/50 flex items-center justify-center font-mono text-[11px] font-bold text-white"
          >
            {userHandle}
          </div>
        </div>
      </header>

      {/* Main Workspace Body: Far-Left Vertical Icon Rail + Active Screen */}
      <div className="flex flex-1 min-h-0">
        {/* Far-Left Vertical Icon Rail (Code, Swarm, Mind, Research, Tools, Settings) */}
        <nav className="w-16 bg-[#070a11] border-r border-white/10 flex flex-col items-center py-3 gap-2 select-none shrink-0">
          {railItems.map((item) => {
            const active = cloudTab === item.id;
            return (
              <button
                key={item.id}
                onClick={() => setCloudTab(item.id)}
                className={`w-12 py-2 rounded-xl flex flex-col items-center justify-center gap-1 transition cursor-pointer ${
                  active
                    ? 'bg-[#34343a] text-[#f59e0b] border border-[#f59e0b]/50 shadow-none'
                    : 'text-[#8e96a8] hover:text-white hover:bg-white/5 border border-transparent'
                }`}
              >
                <span className="font-mono text-sm leading-none">{item.icon}</span>
                <span className="text-[10px] font-medium">{item.label}</span>
              </button>
            );
          })}
        </nav>

        {/* Active Cloud Workspace Screen */}
        <div className="flex-1 flex flex-col min-w-0 min-h-0">
          {cloudTab === 'code' && <CloudCodeIdeView />}
          {cloudTab !== 'code' && (
            <Suspense fallback={<CloudTabFallback />}>
              {cloudTab === 'swarm' && <CloudSwarmView />}
              {cloudTab === 'mind' && <CloudMindGraphView />}
              {cloudTab === 'research' && <CloudResearchHubView />}
              {cloudTab === 'tools' && (
                <div className="flex-1 overflow-y-auto bg-black">
                  <div className="border-b border-white/10 bg-[#2e2e33] px-6 py-3 flex flex-wrap items-center justify-between gap-3">
                    <span className="font-mono text-[11px] uppercase tracking-[0.2em] text-[#71717a]">
                      Tools · Embedded catalog
                    </span>
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => setActivePage('tools-mcp')}
                        className="px-3 py-1.5 rounded-lg bg-white/6 hover:bg-white/12 border border-white/12 text-xs text-white transition cursor-pointer"
                      >
                        Open public catalog
                      </button>
                      <button
                        onClick={() => setCloudTab('code')}
                        className="px-3 py-1.5 rounded-lg bg-[#f59e0b] hover:bg-[#d97706] text-white text-xs font-semibold transition cursor-pointer"
                      >
                        Back to Code
                      </button>
                    </div>
                  </div>
                  <ToolsMcpPage />
                </div>
              )}
            </Suspense>
          )}
          {cloudTab === 'settings' && (
            <div className="flex-1 overflow-y-auto bg-[#26262b]">
              <CloudSettingsView />
            </div>
          )}
        </div>
      </div>

      {/* Bottom Cloud Status Footer */}
      <footer className="h-7 px-4 bg-[#26262b] border-t border-white/10 flex items-center justify-between font-mono text-[10px] text-[#71717a] select-none">
        <div className="flex items-center gap-4">
          <span className="text-emerald-400">● Sample session · Active</span>
          <span>Model: {activeModelObj.name}</span>
          <span>KV: {activeModelObj.kvMode}</span>
        </div>
        <div className="flex items-center gap-4">
          <span className="text-[#ffffff]">☁ Cloud workspace · Local compute</span>
          <span className="text-emerald-400">✓ All changes saved to N³ SQLite</span>
        </div>
      </footer>
    </div>
  );
};
