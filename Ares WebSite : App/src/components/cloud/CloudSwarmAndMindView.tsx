import React, { useState } from 'react';
import { useAtomaStore } from '../../store/useAtomaStore';
import { AtomaAgentBadge } from '../brand/AtomaBrandAssets';
import {
  CheckCircle2,
  Cpu,
  Database,
  FileText,
  Folder,
  GitBranch,
  Layers,
  Play,
  Search,
  Sparkles,
  Workflow,
} from '../ui/Icons';

/**
 * Byte-for-byte implementation of:
 * 1. Screen 09: Swarm Operations + Execution DAG + Holonomy Context Transport + Gantt Timeline + Dual Editors (09-swarm-holonomy-dag-v2-3.png)
 * 2. Screen 10: Mind Knowledge Explorer + 3,482-node Relational Graph + Node Inspector (10-mind-context-graph-v2-1.png)
 */
export const CloudSwarmView: React.FC = () => {
  const {
    swarmTasks,
    swarmDecomposedBanner,
    setSwarmDecomposedBanner,
    setCloudTab,
  } = useAtomaStore();

  const [selectedWorker, setSelectedWorker] = useState('Builder');
  const [coordinatorPrompt, setCoordinatorPrompt] = useState('');
  const [coordinatorReplies, setCoordinatorReplies] = useState<
    Array<{ sender: string; time: string; text: string }>
  >([
    {
      sender: 'You',
      time: '10:32',
      text: 'Continue with the implementation. Focus on deduplication before running the tests.',
    },
    {
      sender: 'Coordinator',
      time: '10:32',
      text: 'Acknowledged. Executing next steps across Builder-01 and Verifier-03 worktrees.',
    },
  ]);

  const workers = [
    {
      name: 'Coordinator',
      role: 'Plan & orchestrate',
      detail: 'Running execution plan · step 6/14',
      status: 'emerald',
    },
    {
      name: 'Builder',
      role: 'Implement changes',
      detail: 'Editing runbook.rs · 3 files · 2 tests',
      status: 'blue',
    },
    {
      name: 'Researcher',
      role: 'Gather evidence',
      detail: 'Retrieving sources · 12 sources synced',
      status: 'emerald',
    },
    {
      name: 'Verifier',
      role: 'Validate & test',
      detail: 'Running unit tests · 14/14 passing',
      status: 'emerald',
    },
    {
      name: 'Memory',
      role: 'Context & storage',
      detail: '3 chunks indexed in N³ SQLite',
      status: 'emerald',
    },
    {
      name: 'Critic',
      role: 'Adversarial review',
      detail: '2 issues (minor) · Gate active',
      status: 'amber',
    },
    {
      name: 'Observer',
      role: 'Telemetry & trace',
      detail: 'Live bus sequence #408',
      status: 'emerald',
    },
  ];

  const handleCoordSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!coordinatorPrompt.trim()) return;
    setCoordinatorReplies((prev) => [
      ...prev,
      { sender: 'You', time: 'Now', text: coordinatorPrompt },
      {
        sender: 'Coordinator',
        time: 'Now',
        text: `Dispatched packet update "${coordinatorPrompt}" to ${selectedWorker} worktree. Holonomy cosine similarity stable at 0.92.`,
      },
    ]);
    setCoordinatorPrompt('');
  };

  return (
    <div className="flex flex-col flex-1 min-h-0 bg-[#06090f] text-[#e8edf5] select-none">
      {swarmDecomposedBanner && (
        <div className="px-4 py-2 bg-emerald-500/15 border-b border-emerald-500/30 flex items-center justify-between text-xs">
          <span className="text-emerald-300 font-medium flex items-center gap-2">
            <CheckCircle2 className="w-4 h-4 text-emerald-400" />
            Research specification decomposed into {swarmTasks.length} dependency-ordered Swarm
            packets and assigned across 6 active workers.
          </span>
          <div className="flex items-center gap-2">
            <button
              onClick={() => setCloudTab('code')}
              className="px-2.5 py-1 rounded bg-emerald-500/25 hover:bg-emerald-500/35 text-emerald-200 font-mono text-[11px] cursor-pointer"
            >
              Inspect in Code IDE →
            </button>
            <button
              onClick={() => setSwarmDecomposedBanner(false)}
              className="text-emerald-300 hover:text-white px-1.5"
            >
              ×
            </button>
          </div>
        </div>
      )}

      {/* Main 3-Column Swarm Layout (Byte-for-Byte 09-swarm-holonomy-dag-v2-3.png) */}
      <div className="grid grid-cols-1 xl:grid-cols-[240px_1fr_320px] flex-1 min-h-0">
        {/* LEFT COLUMN: Swarm operations + Workers (6) + Local compute resources */}
        <aside className="bg-[#080c14] border-r border-white/8 p-3 flex flex-col justify-between space-y-4 overflow-y-auto">
          <div className="space-y-3">
            <div>
              <h3 className="text-xs font-semibold text-white">Swarm operations</h3>
              <p className="text-[11px] text-[#9ba5b4] mt-0.5 leading-relaxed">
                Coordinate agents, track progress and validate results across worktrees.
              </p>
            </div>

            {/* Active Study Card */}
            <div className="p-3 rounded-xl bg-[#0c1320] border border-[#389cff]/30 space-y-2">
              <div className="text-xs font-semibold text-white">N³ runbook study</div>
              <p className="text-[11px] text-[#9ba5b4] leading-relaxed">
                Implement the proposed changes in runbook.rs based on the coordinator plan.
              </p>
              <div className="flex items-center gap-2 text-[10px] font-mono">
                <span className="px-2 py-0.5 rounded bg-white/8 text-[#cbd5e1]">
                  ⎇ runbook-review
                </span>
                <span className="px-2 py-0.5 rounded bg-[#389cff]/20 text-[#389cff]">
                  ● In progress
                </span>
              </div>
              <div className="pt-1 space-y-1">
                <div className="flex justify-between text-[10px] font-mono text-[#9ba5b4]">
                  <span>Plan · Implement · Validate · Review</span>
                </div>
                <div className="h-1.5 rounded-full bg-white/10 overflow-hidden">
                  <div className="h-full w-[43%] bg-[#389cff] rounded-full" />
                </div>
                <div className="flex justify-between text-[10px] font-mono text-[#cbd5e1]">
                  <span>6/14 steps complete</span>
                  <span>43%</span>
                </div>
              </div>
            </div>

            {/* Workers (6) | Worktrees (3) */}
            <div className="flex rounded-lg bg-[#0c121d] p-0.5 border border-white/8 text-xs">
              <span className="flex-1 text-center py-1 rounded-md bg-[#389cff]/20 text-white font-medium">
                Workers (6)
              </span>
              <span className="flex-1 text-center py-1 text-[#9ba5b4]">Worktrees (3)</span>
            </div>

            <div className="space-y-1.5">
              {workers.map((w) => {
                const active = selectedWorker === w.name;
                return (
                  <button
                    key={w.name}
                    onClick={() => setSelectedWorker(w.name)}
                    className={`w-full p-2.5 rounded-xl text-left border transition cursor-pointer ${
                      active
                        ? 'bg-[#10192b] border-[#389cff] shadow-[0_0_20px_rgba(56,156,255,0.3)]'
                        : 'bg-[#0b101b] border-white/8 hover:border-white/20'
                    }`}
                  >
                    <div className="flex items-center justify-between">
                      <span className="text-xs font-semibold text-white">{w.name}</span>
                      <span
                        className={`w-2 h-2 rounded-full ${
                          w.status === 'blue'
                            ? 'bg-[#389cff] animate-ping'
                            : w.status === 'amber'
                            ? 'bg-amber-400'
                            : 'bg-emerald-400'
                        }`}
                      />
                    </div>
                    <div className="text-[11px] text-[#cbd5e1]">{w.role}</div>
                    <div className="text-[10px] font-mono text-[#9ba5b4] mt-0.5">{w.detail}</div>
                  </button>
                );
              })}
            </div>
          </div>

          {/* Local compute resources */}
          <div className="pt-3 border-t border-white/8 space-y-2 font-mono text-[10px]">
            <div className="flex items-center justify-between">
              <span className="text-[#9ba5b4]">Local compute resources</span>
              <span className="text-emerald-400">● Nominal</span>
            </div>
            <div className="grid grid-cols-3 gap-1.5">
              <div className="p-1.5 rounded bg-[#0c121d] border border-white/8">
                <div className="text-[#9ba5b4]">CPU</div>
                <div className="text-xs font-bold text-white">38%</div>
              </div>
              <div className="p-1.5 rounded bg-[#0c121d] border border-white/8">
                <div className="text-[#9ba5b4]">GPU</div>
                <div className="text-xs font-bold text-purple-300">62%</div>
              </div>
              <div className="p-1.5 rounded bg-[#0c121d] border border-white/8">
                <div className="text-[#9ba5b4]">Memory</div>
                <div className="text-xs font-bold text-cyan-300">71%</div>
              </div>
            </div>
          </div>
        </aside>

        {/* CENTER COLUMN: Execution DAG + Execution Timeline + Dual Code Editors */}
        <div className="flex flex-col min-w-0 bg-[#070b12]">
          {/* Top Execution DAG Header */}
          <div className="flex items-center justify-between px-4 py-2 border-b border-white/8 bg-[#0a0f18] text-xs">
            <span className="font-semibold text-white flex items-center gap-2">
              <Workflow className="w-4 h-4 text-[#389cff]" /> Execution DAG
            </span>
            <div className="flex items-center gap-1 bg-[#101724] p-0.5 rounded-lg border border-white/8 text-[11px]">
              <span className="px-2.5 py-0.5 rounded bg-[#389cff] text-white font-medium">
                Graph
              </span>
              <span className="px-2.5 py-0.5 text-[#9ba5b4]">Code</span>
              <span className="px-2.5 py-0.5 text-[#9ba5b4]">Logs</span>
              <span className="px-2.5 py-0.5 text-[#9ba5b4]">Traces</span>
            </div>
          </div>

          {/* 5-Column Execution DAG Canvas (matches 09-swarm-holonomy-dag-v2-3.png) */}
          <div className="p-4 linear-dot-bg border-b border-white/8 overflow-x-auto">
            <div className="grid grid-cols-5 gap-3 min-w-[780px] items-center">
              {/* Col 1: Objective & Coordinator */}
              <div className="space-y-3">
                <div className="p-3 rounded-xl bg-[#0c1320] border border-emerald-500/40 space-y-1">
                  <div className="flex justify-between text-xs font-semibold text-white">
                    <span>Objective</span>
                    <span className="text-emerald-400">✓</span>
                  </div>
                  <div className="text-[10px] text-[#9ba5b4]">Validate runbook proposal</div>
                  <div className="text-[10px] font-mono text-emerald-300">📄 spec.md ✓</div>
                </div>
                <div className="p-3 rounded-xl bg-[#0c1320] border border-emerald-500/40 space-y-1">
                  <div className="flex justify-between text-xs font-semibold text-white">
                    <span>Coordinator</span>
                    <span className="text-emerald-400">✓</span>
                  </div>
                  <div className="text-[10px] text-[#9ba5b4]">Decompose plan &amp; assign work</div>
                  <div className="text-[10px] font-mono text-emerald-300">📄 plan.md ✓</div>
                </div>
              </div>

              {/* Col 2: Retrieve sources, Inspect code, Define tests */}
              <div className="space-y-2.5">
                {[
                  ['Retrieve sources', 'Search code & docs (12)', 'web-search ✓'],
                  ['Inspect code', 'Analyze runbook.rs', 'code-index ✓'],
                  ['Define tests', 'Generate unit test plan', 'test-gen ✓'],
                ].map(([t, s, tool]) => (
                  <div
                    key={t}
                    className="p-2.5 rounded-xl bg-[#0c1320] border border-cyan-500/35 space-y-1"
                  >
                    <div className="flex justify-between text-xs font-semibold text-white">
                      <span>{t}</span>
                      <span className="w-2 h-2 rounded-full bg-emerald-400" />
                    </div>
                    <div className="text-[10px] text-[#9ba5b4]">{s}</div>
                    <div className="text-[10px] font-mono text-cyan-300">{tool}</div>
                  </div>
                ))}
              </div>

              {/* Col 3: Build patch */}
              <div>
                <div className="p-3.5 rounded-xl bg-[#101c30] border-2 border-[#389cff] shadow-[0_0_25px_rgba(56,156,255,0.4)] space-y-1.5">
                  <div className="flex justify-between text-xs font-semibold text-white">
                    <span>Build patch</span>
                    <span className="w-2 h-2 rounded-full bg-[#389cff] animate-ping" />
                  </div>
                  <div className="text-[11px] text-[#cbd5e1]">
                    Implement changes in runbook.rs
                  </div>
                  <div className="text-[10px] font-mono text-[#389cff]">
                    ⚡ Atoma-N3-70B ✓
                  </div>
                </div>
              </div>

              {/* Col 4: Unit tests, Contract check, Adversarial review */}
              <div className="space-y-2.5">
                {[
                  ['Unit tests', '14/14 passing', 'cargo test ✓', 'text-emerald-400'],
                  ['Contract check', 'Validate interfaces', 'spec-check ✓', 'text-cyan-300'],
                  ['Adversarial review', 'Check edge cases', 'code-review ●', 'text-amber-300'],
                ].map(([t, s, tool, col]) => (
                  <div
                    key={t}
                    className="p-2.5 rounded-xl bg-[#0c1320] border border-white/12 space-y-1"
                  >
                    <div className="text-xs font-semibold text-white">{t}</div>
                    <div className={`text-[10px] font-mono ${col}`}>{s}</div>
                    <div className="text-[10px] font-mono text-[#9ba5b4]">{tool}</div>
                  </div>
                ))}
              </div>

              {/* Col 5: Human approval, Receipt, Archive */}
              <div className="space-y-2.5">
                {[
                  ['Human approval', 'Review & approve changes'],
                  ['Receipt', 'Finalize & record outcome'],
                  ['Archive', 'Update N³ knowledge base'],
                ].map(([t, s]) => (
                  <div
                    key={t}
                    className="p-2.5 rounded-xl bg-[#0c1320] border border-white/12 space-y-1"
                  >
                    <div className="text-xs font-semibold text-white">{t}</div>
                    <div className="text-[10px] text-[#9ba5b4]">{s}</div>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Execution Timeline (Gantt Swimlanes) */}
          <div className="p-3 border-b border-white/8 bg-[#080c14] space-y-2">
            <div className="flex items-center justify-between text-xs">
              <span className="font-semibold text-white">
                Execution timeline <span className="text-[#9ba5b4] font-normal">(last 60m)</span>
              </span>
              <div className="flex items-center gap-3 text-[10px] font-mono">
                <span className="text-[#389cff]">● Running</span>
                <span className="text-emerald-400">● Completed</span>
                <span className="text-amber-400">● Waiting</span>
              </div>
            </div>
            <div className="space-y-1.5 font-mono text-[10px]">
              {[
                {
                  lane: 'Coordinator',
                  bars: [
                    { label: 'Plan', w: 'w-1/5', col: 'bg-blue-500/40 border-blue-400' },
                    { label: 'Decompose', w: 'w-1/5', col: 'bg-blue-500/40 border-blue-400' },
                    { label: 'Coordinate workers', w: 'w-2/5', col: 'bg-[#389cff] border-white/40' },
                  ],
                },
                {
                  lane: 'Builder',
                  bars: [
                    { label: 'Retrieve context', w: 'w-1/4', col: 'bg-white/10 border-white/20' },
                    { label: 'Implement changes', w: 'w-2/5', col: 'bg-[#389cff] border-white/40' },
                    { label: 'Run tests', w: 'w-1/4', col: 'bg-white/10 border-white/20' },
                  ],
                },
                {
                  lane: 'Verifier',
                  bars: [
                    { label: 'Generate tests', w: 'w-1/4', col: 'bg-white/10 border-white/20' },
                    { label: 'Run unit tests (14/14)', w: 'w-2/5', col: 'bg-emerald-500/40 border-emerald-400' },
                  ],
                },
              ].map((row) => (
                <div key={row.lane} className="grid grid-cols-[100px_1fr] items-center gap-2">
                  <span className="text-[#cbd5e1]">{row.lane}</span>
                  <div className="flex items-center gap-1.5 bg-[#05080e] p-1 rounded border border-white/6">
                    {row.bars.map((b) => (
                      <div
                        key={b.label}
                        className={`${b.w} px-2 py-0.5 rounded border text-white truncate ${b.col}`}
                      >
                        {b.label}
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Dual Code Editors: Builder – runbook.rs | Verifier – acceptance.rs */}
          <div className="grid grid-cols-1 lg:grid-cols-2 flex-1 min-h-[210px] divide-y lg:divide-y-0 lg:divide-x divide-white/8 bg-[#06090f] font-mono text-[11px]">
            <div className="p-3 space-y-2">
              <div className="flex items-center justify-between border-b border-white/8 pb-1.5">
                <span className="text-emerald-400 font-semibold">● Builder – runbook.rs</span>
                <span className="text-[10px] text-[#64748b]">
                  feature/runbook-updates › src › runbook.rs
                </span>
              </div>
              <pre className="text-[#cbd5e1] overflow-x-auto leading-relaxed">
{`108  /// Implement the proposed changes for N³ runbook.
109  /// Adds deduplication, validation and improved event structure.
110  use std::collections::{HashMap, HashSet};
111  use serde::{Deserialize, Serialize};
112
113  #[derive(Debug, Clone, Serialize, Deserialize)]
114  pub struct RunbookEvent {
115      pub id: String,
116      pub event_type: String,
117      pub source: String,
118      pub dedup_key: Option<String>,
119  }
120  impl RunbookEvent {
121      pub fn dedup_key(&self) -> String {
122          format!("{}:{}", self.source, self.event_type)
123      }
124  }`}
              </pre>
            </div>

            <div className="p-3 space-y-2">
              <div className="flex items-center justify-between border-b border-white/8 pb-1.5">
                <span className="text-emerald-400 font-semibold">● Verifier – acceptance.rs</span>
                <span className="text-[10px] text-[#64748b]">
                  feature/runbook-updates › src › tests › acceptance.rs
                </span>
              </div>
              <pre className="text-[#cbd5e1] overflow-x-auto leading-relaxed">
{`25  use anyhow::Result;
26  use atoma::runbook::{RunbookEvent, dedup_check};
27
28  #[tokio::test]
29  async fn test_event_deduplication() -> Result<()> {
30      let event = RunbookEvent {
31          id: "evt-001".to_string(),
32          event_type: "file_change".to_string(),
33          source: "runbook.rs".to_string(),
34          dedup_key: None,
35      };
36      assert_eq!(event.dedup_key(), "runbook.rs:file_change");
37      Ok(())
38  }`}
              </pre>
            </div>
          </div>
        </div>

        {/* RIGHT COLUMN: System Telemetry + Holonomy Context Transport + Coordinator Command */}
        <aside className="bg-[#080c14] border-l border-white/8 p-3 flex flex-col justify-between space-y-4 overflow-y-auto">
          <div className="space-y-4">
            {/* System telemetry */}
            <div className="space-y-2">
              <div className="flex items-center justify-between text-xs">
                <span className="font-semibold text-white">System telemetry</span>
                <span className="text-[10px] font-mono text-[#9ba5b4]">Last 60 minutes</span>
              </div>
              {[
                { label: 'CPU', val: '38%', col: '#389cff' },
                { label: 'GPU', val: '62%', col: '#a855f7' },
                { label: 'Memory', val: '71%', col: '#06b6d4' },
              ].map((m) => (
                <div
                  key={m.label}
                  className="p-2 rounded-lg bg-[#0b111c] border border-white/8 space-y-1"
                >
                  <div className="flex justify-between text-[11px] font-mono">
                    <span className="text-[#cbd5e1]">{m.label}</span>
                    <span className="text-white font-bold">{m.val}</span>
                  </div>
                  <svg viewBox="0 0 220 26" className="w-full h-6">
                    <path
                      d="M0,20 L25,14 L50,18 L75,9 L100,15 L125,7 L150,14 L175,8 L200,13 L220,6"
                      fill="none"
                      stroke={m.col}
                      strokeWidth="1.6"
                    />
                  </svg>
                </div>
              ))}
            </div>

            {/* Holonomy · Context transport (exact match to 09-swarm-holonomy-dag-v2-3.png) */}
            <div className="p-3 rounded-xl bg-[#0b111c] border border-white/10 space-y-2.5">
              <div className="text-xs font-semibold text-white">
                △ Holonomy · Context transport
              </div>
              <div className="grid grid-cols-[1fr_110px] gap-2 items-center">
                <svg viewBox="0 0 150 130" className="w-full h-28">
                  <polygon
                    points="75,14 20,108 130,108"
                    fill="rgba(56,156,255,0.12)"
                    stroke="#389cff"
                    strokeWidth="1.5"
                  />
                  <line x1="75" y1="14" x2="75" y2="75" stroke="#60a5fa" strokeDasharray="2 2" />
                  <line x1="20" y1="108" x2="75" y2="75" stroke="#60a5fa" strokeDasharray="2 2" />
                  <line x1="130" y1="108" x2="75" y2="75" stroke="#60a5fa" strokeDasharray="2 2" />
                  <circle cx="75" cy="14" r="4" fill="#389cff" />
                  <circle cx="20" cy="108" r="4" fill="#389cff" />
                  <circle cx="130" cy="108" r="4" fill="#389cff" />
                  <text x="82" y="18" fill="#ffffff" fontSize="8" fontFamily="JetBrains Mono">
                    A Problem
                  </text>
                  <text x="6" y="122" fill="#9ba5b4" fontSize="8" fontFamily="JetBrains Mono">
                    B Evidence
                  </text>
                  <text x="92" y="122" fill="#9ba5b4" fontSize="8" fontFamily="JetBrains Mono">
                    C Validation
                  </text>
                </svg>

                <div className="space-y-1.5 font-mono text-[10px]">
                  <div className="px-2 py-1 rounded bg-[#10192b] border border-[#389cff]/40 text-white">
                    A → B → C → A
                  </div>
                  <div className="flex justify-between text-[#9ba5b4]">
                    <span>Cosine sim.</span>
                    <span className="text-white font-bold">0.92</span>
                  </div>
                  <div className="flex justify-between text-[#9ba5b4]">
                    <span>Residual</span>
                    <span className="text-emerald-400 font-bold">0.08</span>
                  </div>
                </div>
              </div>
            </div>

            {/* Coordinator command chat */}
            <div className="p-3 rounded-xl bg-[#0b111c] border border-white/10 space-y-2.5 text-xs">
              <div className="flex items-center justify-between">
                <span className="font-semibold text-white">Coordinator command</span>
                <span className="px-2 py-0.5 rounded bg-[#389cff]/20 text-[#389cff] font-mono text-[10px]">
                  {selectedWorker}
                </span>
              </div>

              <div className="space-y-2 max-h-40 overflow-y-auto">
                {coordinatorReplies.map((r, i) => (
                  <div key={i} className="space-y-0.5">
                    <div className="flex justify-between text-[10px] font-mono text-[#9ba5b4]">
                      <span className="text-white font-semibold">{r.sender}</span>
                      <span>{r.time}</span>
                    </div>
                    <p className="text-[11px] text-[#cbd5e1]">{r.text}</p>
                  </div>
                ))}
              </div>

              <form onSubmit={handleCoordSubmit} className="flex gap-1.5 pt-1">
                <input
                  type="text"
                  value={coordinatorPrompt}
                  onChange={(e) => setCoordinatorPrompt(e.target.value)}
                  placeholder="Direct the coordinator..."
                  className="flex-1 px-2.5 py-1.5 rounded bg-[#06090f] border border-white/12 text-xs text-white focus:outline-none"
                />
                <button
                  type="submit"
                  className="px-3 py-1.5 rounded bg-[#389cff] text-white font-semibold text-xs cursor-pointer"
                >
                  ↗
                </button>
              </form>
            </div>
          </div>
        </aside>
      </div>
    </div>
  );
};

/**
 * Screen 10: Mind Context Graph (Byte-for-Byte 10-mind-context-graph-v2-1.png)
 */
export const CloudMindGraphView: React.FC = () => {
  const [selectedHub, setSelectedHub] = useState('Human review gate');
  const [nodeSize, setNodeSize] = useState(1.2);
  const [linkOpacity, setLinkOpacity] = useState(0.35);
  const [repulsion, setRepulsion] = useState(0.7);

  const hubs = [
    {
      id: 'Human review gate',
      x: 50,
      y: 48,
      color: '#389cff',
      type: 'Constraint',
      cluster: 'Operations & Runbooks',
      desc: 'Consequential changes remain gated by explicit human review. Ensures safety, compliance and traceability for high-impact modifications.',
    },
    {
      id: 'Acceptance contract',
      x: 44,
      y: 34,
      color: '#60a5fa',
      type: 'Contract',
      cluster: 'Verification & Safety',
      desc: 'Formal pre-registered test assertions required before promoting a worktree patch.',
    },
    {
      id: 'Design contract',
      x: 36,
      y: 44,
      color: '#389cff',
      type: 'Specification',
      cluster: 'Code & Schemas',
      desc: 'Defines module boundaries, event deduplication rules, and invariant checks.',
    },
    {
      id: 'runbook.rs',
      x: 61,
      y: 41,
      color: '#38bdf8',
      type: 'Source Module',
      cluster: 'Code & Schemas',
      desc: 'Primary Rust implementation of Runbook validation and event deduplication.',
    },
    {
      id: 'Receipt schema',
      x: 42,
      y: 61,
      color: '#a855f7',
      type: 'SQLite Table',
      cluster: 'Memory & Context',
      desc: 'Append-only N³ SQLite receipt table recording SHA-256 artifact digests.',
    },
    {
      id: 'Verification',
      x: 62,
      y: 58,
      color: '#10b981',
      type: 'Gate',
      cluster: 'Verification & Safety',
      desc: 'Automated cargo test + judge verification harness.',
    },
    {
      id: 'Context retention',
      x: 56,
      y: 68,
      color: '#389cff',
      type: 'Study',
      cluster: 'Research & Manuscripts',
      desc: 'Quantifies objective recovery R = |O_T ∩ O_0| / |O_0| across compaction.',
    },
  ];

  const activeHub = hubs.find((h) => h.id === selectedHub) || hubs[0];

  return (
    <div className="grid grid-cols-1 xl:grid-cols-[240px_1fr_320px] flex-1 min-h-0 bg-[#06090f] text-[#e8edf5] select-none">
      {/* Left Knowledge Explorer + Graph Controls */}
      <aside className="bg-[#080c14] border-r border-white/8 p-3 space-y-4 overflow-y-auto">
        <div className="text-xs font-semibold text-white">Knowledge explorer</div>
        <div className="space-y-1 font-mono text-[11px] text-[#cbd5e1]">
          <div className="text-white font-semibold">▾ 📁 N³ research hub</div>
          <div className="pl-3 space-y-1">
            <div className="px-2 py-1 rounded bg-[#389cff]/20 text-white border border-[#389cff]/40">
              📁 Research universe
            </div>
            <div className="pl-3 space-y-1 text-[#9ba5b4]">
              <div>▸ 📁 Runbook study</div>
              <div>▸ 📁 Design contracts</div>
              <div>▸ 📁 Manuscripts</div>
              <div>▸ 📁 Experiments</div>
              <div>▸ 📁 Receipts</div>
            </div>
          </div>
        </div>

        <div className="pt-3 border-t border-white/8 space-y-2.5 text-[11px]">
          <div className="text-xs font-semibold text-white">Graph controls</div>
          <div className="space-y-1">
            <div className="flex justify-between text-[#9ba5b4]">
              <span>Node size</span>
              <span className="font-mono text-white">{nodeSize.toFixed(1)}x</span>
            </div>
            <input
              type="range"
              min={0.6}
              max={2.0}
              step={0.1}
              value={nodeSize}
              onChange={(e) => setNodeSize(parseFloat(e.target.value))}
              className="w-full accent-[#389cff]"
            />
          </div>
          <div className="space-y-1">
            <div className="flex justify-between text-[#9ba5b4]">
              <span>Link opacity</span>
              <span className="font-mono text-white">{linkOpacity.toFixed(2)}</span>
            </div>
            <input
              type="range"
              min={0.1}
              max={0.9}
              step={0.05}
              value={linkOpacity}
              onChange={(e) => setLinkOpacity(parseFloat(e.target.value))}
              className="w-full accent-[#389cff]"
            />
          </div>
          <div className="space-y-1">
            <div className="flex justify-between text-[#9ba5b4]">
              <span>Repulsion</span>
              <span className="font-mono text-white">{repulsion.toFixed(2)}</span>
            </div>
            <input
              type="range"
              min={0.2}
              max={1.5}
              step={0.05}
              value={repulsion}
              onChange={(e) => setRepulsion(parseFloat(e.target.value))}
              className="w-full accent-[#389cff]"
            />
          </div>
        </div>
      </aside>

      {/* Center Relational Graph Canvas (3,482 nodes | 8,216 links) */}
      <div className="flex flex-col min-w-0 bg-[#05070c]">
        <div className="flex items-center justify-between px-4 py-2 border-b border-white/8 bg-[#090d16] text-xs">
          <div className="flex items-center gap-3">
            <span className="font-semibold text-white border-b-2 border-[#389cff] pb-1">
              Relational graph
            </span>
            <span className="text-[#9ba5b4]">Local graph</span>
            <span className="text-[#9ba5b4]">Sources</span>
          </div>
          <div className="font-mono text-[11px] text-[#9ba5b4]">
            3,482 nodes &nbsp;|&nbsp; 8,216 links &nbsp;|&nbsp; 3 hops
          </div>
        </div>

        <div className="relative flex-1 min-h-[480px] overflow-hidden bg-radial from-[#0d1829] via-[#060911] to-[#04060a]">
          {/* Glowing Relational Web */}
          <svg viewBox="0 0 100 100" className="w-full h-full">
            {hubs.slice(1).map((h) => (
              <line
                key={h.id}
                x1={hubs[0].x}
                y1={hubs[0].y}
                x2={h.x}
                y2={h.y}
                stroke="#389cff"
                strokeOpacity={linkOpacity + 0.35}
                strokeWidth="0.45"
              />
            ))}
          </svg>

          {hubs.map((h) => {
            const isSelected = selectedHub === h.id;
            return (
              <button
                key={h.id}
                onClick={() => setSelectedHub(h.id)}
                style={{ left: `${h.x}%`, top: `${h.y}%` }}
                className={`absolute -translate-x-1/2 -translate-y-1/2 px-3 py-1.5 rounded-full border text-xs font-medium flex items-center gap-2 transition cursor-pointer ${
                  isSelected
                    ? 'bg-[#102038] border-[#389cff] text-white shadow-[0_0_30px_rgba(56,156,255,0.8)] scale-110 z-20'
                    : 'bg-[#0b121e]/90 border-white/20 text-[#cbd5e1] hover:border-[#389cff]'
                }`}
              >
                <span
                  className="w-2.5 h-2.5 rounded-full"
                  style={{ backgroundColor: h.color }}
                />
                <span>{h.id}</span>
              </button>
            );
          })}
        </div>

        {/* Bottom Selected Path Bar */}
        <div className="px-4 py-2.5 border-t border-white/8 bg-[#080c14] flex flex-wrap items-center gap-2 text-[11px] font-mono">
          <span className="text-[#9ba5b4]">Selected path (6 nodes):</span>
          {[
            'Acceptance contract',
            'Human review gate',
            'Design contract',
            'runbook.rs',
            'Verification',
            'Context retention',
          ].map((n, i) => (
            <React.Fragment key={n}>
              {i > 0 && <span className="text-[#64748b]">→</span>}
              <span
                onClick={() => setSelectedHub(n)}
                className={`px-2 py-0.5 rounded cursor-pointer ${
                  selectedHub === n
                    ? 'bg-[#389cff]/25 border border-[#389cff] text-white'
                    : 'bg-white/5 text-[#cbd5e1]'
                }`}
              >
                {n}
              </span>
            </React.Fragment>
          ))}
        </div>
      </div>

      {/* Right Node Inspector */}
      <aside className="bg-[#080c14] border-l border-white/8 p-4 space-y-4 overflow-y-auto">
        <div className="flex items-center justify-between border-b border-white/8 pb-2">
          <span className="text-xs font-semibold text-white">Node inspector</span>
          <span className="px-2 py-0.5 rounded bg-[#389cff]/20 text-[#389cff] font-mono text-[10px]">
            {activeHub.type}
          </span>
        </div>

        <div>
          <h3 className="text-lg font-display font-bold text-white">{activeHub.id}</h3>
          <div className="text-xs text-[#389cff] mt-0.5">● {activeHub.cluster}</div>
          <p className="text-xs text-[#9ba5b4] mt-3 leading-relaxed bg-[#0c121e] p-3 rounded-xl border border-white/8">
            {activeHub.desc}
          </p>
        </div>

        <div className="space-y-2 pt-2 border-t border-white/8 text-xs">
          <div className="font-semibold text-white">Linked sources (3)</div>
          {[
            ['docs/design.md', 'Section 2.4 · Human review'],
            ['src/runbook.rs', 'Implements review gate'],
            ['tests/acceptance.rs', 'Validates gated changes'],
          ].map(([file, desc]) => (
            <div
              key={file}
              className="p-2.5 rounded-lg bg-[#0c121e] border border-white/8 space-y-0.5"
            >
              <div className="font-mono text-white text-[11px]">{file}</div>
              <div className="text-[10px] text-[#9ba5b4]">{desc}</div>
            </div>
          ))}
        </div>
      </aside>
    </div>
  );
};
