import React, { useState } from 'react';
import {
  useAtomaStore,
  CLOUD_MODELS,
  type UploadedDocItem,
} from '../../store/useAtomaStore';
import { AtomaAgentBadge, AtomaLogoMark } from '../brand/AtomaBrandAssets';
import {
  ArrowRight,
  CheckCircle2,
  ChevronDown,
  ChevronRight,
  Code2,
  Cpu,
  Database,
  FileCheck2,
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
 * Byte-for-byte implementation of 11-research-hub-reader-v2.png + 08-research-hub-v20.png
 * plus the Document Upload -> Workspace Scaffolder -> Research Chat -> Swarm Task Decomposition pipeline.
 */
export const CloudResearchHubView: React.FC = () => {
  const {
    projectIdea,
    setProjectIdea,
    uploadedDocs,
    addUploadedDoc,
    scaffoldWorkspaceFromIdea,
    researchChatTurns,
    addResearchChatTurn,
    decomposeAndAssignToSwarm,
    selectedCloudModel,
    setSelectedCloudModel,
  } = useAtomaStore();

  const [ideaInput, setIdeaInput] = useState(projectIdea);
  const [chatInput, setChatInput] = useState('');
  const [selectedDoc, setSelectedDoc] = useState('context-retention-paper.md');
  const [activeReaderTab, setActiveReaderTab] = useState<'Manuscript' | 'Notes' | 'Evidence'>(
    'Manuscript'
  );
  const [activeAssistantTab, setActiveAssistantTab] = useState<
    'Discuss' | 'Evidence' | 'Agent activity'
  >('Discuss');
  const [activePageThumb, setActivePageThumb] = useState(4);
  const [intakeExpanded, setIntakeExpanded] = useState(true);

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (!files || files.length === 0) return;
    Array.from(files).forEach((f) => {
      const ext = f.name.split('.').pop()?.toUpperCase() || 'MD';
      const kind: UploadedDocItem['kind'] =
        ext === 'PDF' ? 'PDF' : ext === 'RS' || ext === 'TS' ? 'CODE' : ext === 'YAML' ? 'YAML' : 'MD';
      addUploadedDoc({
        id: `doc-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
        name: f.name,
        kind,
        size: `${Math.max(4, Math.round(f.size / 1024))} KB`,
        sections: 5,
        status: 'Verified',
        summary: `Uploaded by user into Research worktree · Indexed into N³ relational graph.`,
      });
    });
  };

  const handleChatSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!chatInput.trim()) return;
    addResearchChatTurn(chatInput);
    setChatInput('');
  };

  return (
    <div className="flex flex-col flex-1 min-h-0 bg-[#06090f] text-[#e8edf5] select-none">
      {/* Top Document Ingestion & Idea Scaffolder Bar */}
      <div className="border-b border-white/10 bg-gradient-to-r from-[#09111e] via-[#0c1626] to-[#080d16] px-4 py-2.5">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex items-center gap-2.5">
            <span className="px-2 py-0.5 rounded bg-[#389cff]/20 border border-[#389cff]/40 text-[#389cff] font-mono text-[10px] font-semibold">
              STEP 1 · RESEARCH INGESTION &amp; SCAFFOLDER
            </span>
            <span className="text-xs text-[#cbd5e1] font-medium">
              Upload reference docs, define the overall idea, iron out details in Research Chat, then
              decompose to the Swarm.
            </span>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={() => setIntakeExpanded(!intakeExpanded)}
              className="px-2.5 py-1 rounded bg-white/6 hover:bg-white/12 border border-white/10 text-[11px] font-mono text-[#c9cfd7] cursor-pointer"
            >
              {intakeExpanded ? 'Hide Scaffolder ▲' : 'Open Scaffolder ▼'}
            </button>
            <button
              onClick={decomposeAndAssignToSwarm}
              className="px-3.5 py-1.5 rounded-lg bg-[#389cff] hover:bg-[#2589f5] text-white text-xs font-semibold flex items-center gap-1.5 shadow-[0_0_20px_rgba(56,156,255,0.45)] transition cursor-pointer"
            >
              <Sparkles className="w-3.5 h-3.5" />
              <span>Decompose &amp; Assign to Swarm →</span>
            </button>
          </div>
        </div>

        {intakeExpanded && (
          <div className="mt-3 pt-3 border-t border-white/8 grid grid-cols-1 lg:grid-cols-[1.2fr_0.8fr] gap-4 items-center">
            <div className="flex items-center gap-2">
              <input
                type="text"
                value={ideaInput}
                onChange={(e) => setIdeaInput(e.target.value)}
                placeholder="Describe your overall idea or research hypothesis for the model to scaffold..."
                className="flex-1 px-3.5 py-2 rounded-lg bg-[#05080d] border border-white/15 text-xs text-white focus:border-[#389cff] focus:outline-none"
              />
              <button
                onClick={() => scaffoldWorkspaceFromIdea(ideaInput)}
                className="px-3.5 py-2 rounded-lg bg-white/10 hover:bg-[#389cff] text-white text-xs font-medium shrink-0 transition cursor-pointer"
              >
                Scaffold Workspace ↻
              </button>
            </div>

            <div className="flex items-center justify-between gap-2 bg-[#05080d] border border-white/10 rounded-lg px-3 py-1.5">
              <div className="flex items-center gap-2 overflow-x-auto text-[11px] font-mono">
                <span className="text-[#9ba5b4] shrink-0">Docs ({uploadedDocs.length}):</span>
                {uploadedDocs.slice(0, 3).map((d) => (
                  <span
                    key={d.id}
                    className="px-2 py-0.5 rounded bg-[#101826] border border-[#389cff]/30 text-[#cbd5e1] truncate max-w-[140px]"
                  >
                    {d.name}
                  </span>
                ))}
              </div>
              <label className="px-2.5 py-1 rounded bg-[#389cff]/20 hover:bg-[#389cff]/30 border border-[#389cff]/40 text-[#389cff] text-[11px] font-mono font-medium shrink-0 cursor-pointer">
                + Upload Doc
                <input type="file" multiple onChange={handleFileUpload} className="hidden" />
              </label>
            </div>
          </div>
        )}
      </div>

      {/* Main 3-Column Research Hub Reader Layout (Byte-for-Byte 11-research-hub-reader-v2.png) */}
      <div className="grid grid-cols-1 xl:grid-cols-[260px_1fr_380px] flex-1 min-h-0">
        {/* LEFT COLUMN: Research worktree + Relational context mini-graph + Sources (3) */}
        <aside className="bg-[#080c14] border-r border-white/8 flex flex-col justify-between p-3 space-y-3 overflow-y-auto">
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold text-white flex items-center gap-1.5">
                <Database className="w-3.5 h-3.5 text-[#389cff]" /> Research worktree
              </span>
              <span className="text-[10px] font-mono text-[#9ba5b4]">⊞</span>
            </div>

            {/* Git Branch Selector */}
            <div className="px-2.5 py-1.5 rounded-lg bg-[#0d131f] border border-white/10 flex items-center justify-between text-xs font-mono text-[#cbd5e1]">
              <span className="flex items-center gap-1.5 truncate">
                <GitBranch className="w-3.5 h-3.5 text-[#389cff]" /> study / context-retention
              </span>
              <ChevronDown className="w-3.5 h-3.5 text-[#64748b]" />
            </div>

            {/* Worktree File Hierarchy */}
            <div className="space-y-1.5 font-mono text-[11px]">
              {[
                {
                  folder: 'manuscripts/',
                  files: [
                    'context-retention-paper.md',
                    'design-contract.md',
                    'related-work.md',
                  ],
                },
                {
                  folder: 'notes/',
                  files: ['methods-note.md'],
                },
                {
                  folder: 'experiments/',
                  files: ['ablation.yaml', 'run-config.yaml'],
                },
                {
                  folder: 'evidence/',
                  files: ['receipt.json', 'source-metadata.md'],
                },
              ].map((grp) => (
                <div key={grp.folder} className="space-y-0.5">
                  <div className="flex items-center gap-1.5 text-[#9ba5b4] px-1.5 py-0.5">
                    <ChevronDown className="w-3 h-3" />
                    <Folder className="w-3.5 h-3.5 text-[#389cff]" />
                    <span>{grp.folder}</span>
                  </div>
                  <div className="pl-5 space-y-0.5">
                    {grp.files.map((file) => {
                      const active = selectedDoc === file;
                      return (
                        <button
                          key={file}
                          onClick={() => setSelectedDoc(file)}
                          className={`w-full flex items-center gap-1.5 px-2 py-1 rounded text-left transition cursor-pointer ${
                            active
                              ? 'bg-[#389cff]/20 text-white border border-[#389cff]/40'
                              : 'text-[#9ba5b4] hover:text-white hover:bg-white/5'
                          }`}
                        >
                          <FileText className="w-3 h-3 text-[#389cff] shrink-0" />
                          <span className="truncate">{file}</span>
                        </button>
                      );
                    })}
                  </div>
                </div>
              ))}
            </div>

            {/* Relational Context Mini-Graph (matches bottom-left of 11-research-hub-reader-v2.png) */}
            <div className="pt-2 border-t border-white/8 space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-xs font-semibold text-white">Relational context</span>
                <div className="flex rounded bg-[#0d131f] p-0.5 border border-white/8 text-[10px]">
                  <span className="px-2 py-0.5 rounded bg-[#389cff] text-white">Graph</span>
                  <span className="px-2 py-0.5 text-[#9ba5b4]">Table</span>
                </div>
              </div>

              <div className="relative h-44 rounded-xl bg-[#05080e] border border-white/10 p-2.5 overflow-hidden">
                <svg viewBox="0 0 220 150" className="w-full h-full">
                  {/* Connecting web lines */}
                  <g stroke="rgba(56,156,255,0.22)" strokeWidth="0.8">
                    <line x1="45" y1="38" x2="110" y2="75" />
                    <line x1="175" y1="38" x2="110" y2="75" />
                    <line x1="45" y1="118" x2="110" y2="75" />
                    <line x1="175" y1="118" x2="110" y2="75" />
                    <line x1="45" y1="38" x2="175" y2="38" />
                    <line x1="45" y1="118" x2="175" y2="118" />
                  </g>
                  {/* Cluster 1: Workflows (Blue) */}
                  <circle cx="45" cy="38" r="18" fill="rgba(56,156,255,0.18)" />
                  <circle cx="45" cy="38" r="4" fill="#389cff" />
                  {/* Cluster 2: Evaluation (Purple) */}
                  <circle cx="175" cy="38" r="18" fill="rgba(168,85,247,0.18)" />
                  <circle cx="175" cy="38" r="4" fill="#a855f7" />
                  {/* Cluster 3: Compaction (Amber) */}
                  <circle cx="45" cy="118" r="18" fill="rgba(245,158,11,0.18)" />
                  <circle cx="45" cy="118" r="4" fill="#f59e0b" />
                  {/* Cluster 4: Agent Systems (Emerald) */}
                  <circle cx="175" cy="118" r="18" fill="rgba(16,185,129,0.18)" />
                  <circle cx="175" cy="118" r="4" fill="#10b981" />
                  {/* Center Active Card */}
                  <rect
                    x="52"
                    y="54"
                    width="116"
                    height="44"
                    rx="6"
                    fill="#0d1522"
                    stroke="#389cff"
                    strokeWidth="1.2"
                  />
                  <text x="60" y="69" fill="#ffffff" fontSize="8" fontFamily="JetBrains Mono">
                    context-retention-paper.md
                  </text>
                  <text x="60" y="81" fill="#9ba5b4" fontSize="7.5" fontFamily="JetBrains Mono">
                    design-contract.md · 8 sources
                  </text>
                </svg>
                <div className="absolute top-1.5 left-2.5 text-[9px] font-mono text-[#389cff]">
                  Workflows (28)
                </div>
                <div className="absolute top-1.5 right-2.5 text-[9px] font-mono text-purple-400">
                  Evaluation (31)
                </div>
                <div className="absolute bottom-1.5 left-2.5 text-[9px] font-mono text-amber-400">
                  Compaction (27)
                </div>
                <div className="absolute bottom-1.5 right-2.5 text-[9px] font-mono text-emerald-400">
                  Agent Systems (34)
                </div>
              </div>
            </div>

            {/* Sources (3) List */}
            <div className="pt-2 border-t border-white/8 space-y-1.5">
              <div className="flex items-center justify-between text-[11px]">
                <span className="font-semibold text-white">
                  Sources ({uploadedDocs.length})
                </span>
                <span className="text-[#389cff] font-mono text-[10px]">View all →</span>
              </div>
              {uploadedDocs.slice(0, 3).map((src, i) => (
                <div
                  key={src.id}
                  className="p-2 rounded-lg bg-[#0c121d] border border-white/8 flex items-center justify-between text-[11px]"
                >
                  <div className="min-w-0">
                    <div className="font-mono text-white truncate">
                      {i + 1}. {src.name}
                    </div>
                    <div className="text-[10px] text-[#64748b] truncate">{src.kind} source</div>
                  </div>
                  <span className="px-1.5 py-0.5 rounded bg-emerald-500/15 text-emerald-300 font-mono text-[9px] shrink-0">
                    ● {src.status}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </aside>

        {/* CENTER COLUMN: Manuscript Reader + Equation + Figure 1 + Bottom Research DAG */}
        <div className="flex flex-col min-w-0 bg-[#070b12]">
          {/* Document Tab Bar */}
          <div className="flex items-center gap-1 px-3 pt-2 border-b border-white/8 bg-[#0a0f18] text-xs">
            {[
              'Context retention study',
              'Design contract',
              'Methods note',
            ].map((tab, idx) => (
              <div
                key={tab}
                className={`px-3 py-1.5 rounded-t-lg flex items-center gap-2 font-medium border-t border-x ${
                  idx === 0
                    ? 'bg-[#101724] text-white border-white/12'
                    : 'bg-transparent text-[#8e96a8] border-transparent'
                }`}
              >
                <FileText className="w-3.5 h-3.5 text-[#389cff]" />
                <span>{tab}</span>
              </div>
            ))}
          </div>

          {/* Secondary Reader Toolbar */}
          <div className="flex flex-wrap items-center justify-between gap-2 px-4 py-2 border-b border-white/8 bg-[#0c121c] text-xs">
            <div className="flex items-center gap-1">
              {(['Manuscript', 'Notes', 'Evidence'] as const).map((t) => (
                <button
                  key={t}
                  onClick={() => setActiveReaderTab(t)}
                  className={`px-3 py-1 rounded-md font-medium transition cursor-pointer ${
                    activeReaderTab === t
                      ? 'bg-[#389cff]/20 text-[#389cff] border border-[#389cff]/40'
                      : 'text-[#9ba5b4] hover:text-white'
                  }`}
                >
                  {t} {t === 'Notes' ? '(3)' : t === 'Evidence' ? '(5)' : ''}
                </button>
              ))}
            </div>

            <div className="flex items-center gap-2 font-mono text-[11px] text-[#9ba5b4]">
              <span>Page {activePageThumb} / 12</span>
              <span className="px-2 py-0.5 rounded bg-white/5 text-white">100%</span>
              <button className="px-2 py-0.5 rounded bg-white/5 hover:bg-white/10 text-[#cbd5e1]">
                ✎ Annotate
              </button>
              <button className="px-2 py-0.5 rounded bg-white/5 hover:bg-white/10 text-[#cbd5e1]">
                § Cite
              </button>
              <button className="px-2 py-0.5 rounded bg-white/5 hover:bg-white/10 text-[#cbd5e1]">
                ⊞ Split view
              </button>
            </div>
          </div>

          {/* Reader Split Body: Left Page Thumbnails & Outline + Right High-Contrast Paper Canvas */}
          <div className="grid grid-cols-1 md:grid-cols-[150px_1fr] flex-1 min-h-0 overflow-y-auto">
            {/* Page Thumbnails + Outline */}
            <div className="bg-[#090e17] border-r border-white/8 p-3 space-y-4 hidden md:block">
              <div className="space-y-2">
                <div className="text-[10px] font-mono uppercase tracking-wider text-[#9ba5b4]">
                  Pages
                </div>
                {[3, 4, 5].map((pg) => (
                  <button
                    key={pg}
                    onClick={() => setActivePageThumb(pg)}
                    className={`w-full p-2 rounded-lg border text-left transition cursor-pointer ${
                      activePageThumb === pg
                        ? 'bg-[#121c2d] border-[#389cff] shadow-[0_0_15px_rgba(56,156,255,0.3)]'
                        : 'bg-[#0c111a] border-white/10 opacity-75 hover:opacity-100'
                    }`}
                  >
                    <div className="text-[10px] font-mono text-[#9ba5b4] mb-1">Page {pg}</div>
                    <div className="h-16 rounded bg-[#e8edf5] p-1.5 space-y-1">
                      <div className="h-1.5 w-3/4 bg-slate-700 rounded" />
                      <div className="h-1 w-full bg-slate-400 rounded" />
                      <div className="h-1 w-5/6 bg-slate-400 rounded" />
                      <div className="h-3 w-full bg-blue-200 rounded mt-1" />
                    </div>
                  </button>
                ))}
              </div>

              <div className="pt-3 border-t border-white/8 space-y-1.5 text-[11px]">
                <div className="text-[10px] font-mono uppercase tracking-wider text-[#9ba5b4]">
                  Outline
                </div>
                <div className="text-[#9ba5b4]">1. Introduction</div>
                <div className="text-[#389cff] font-medium">2. Experimental design</div>
                <div className="pl-2 text-[10px] text-[#9ba5b4] space-y-1">
                  <div>2.1 Workloads &amp; tasks</div>
                  <div>2.2 Conditions</div>
                  <div>2.3 Metrics</div>
                </div>
                <div className="text-[#9ba5b4]">3. Results</div>
                <div className="text-[#9ba5b4]">4. Analysis</div>
                <div className="text-[#9ba5b4]">5. Related work</div>
              </div>
            </div>

            {/* White/Ivory Academic Manuscript Sheet (exact match to 11-research-hub-reader-v2.png) */}
            <div className="p-5 bg-[#101622] overflow-y-auto">
              <div className="max-w-3xl mx-auto rounded-xl bg-[#f4f6f9] text-[#111827] p-8 shadow-2xl space-y-5 font-sans">
                <div className="flex items-center justify-between text-[11px] font-mono text-slate-500 border-b border-slate-200 pb-2">
                  <span>Atoma Research &nbsp;|&nbsp; Internal draft · Working manuscript</span>
                  <span>Selected: {selectedDoc}</span>
                </div>

                <h1 className="text-2xl sm:text-3xl font-bold tracking-tight text-slate-900 font-serif">
                  Context retention in long-running agent workflows
                </h1>

                <div className="space-y-1.5">
                  <h4 className="text-xs font-bold uppercase tracking-wider text-slate-700">
                    Abstract
                  </h4>
                  <p className="text-xs leading-relaxed text-slate-700">
                    We investigate how well agent systems retain task objectives over extended,
                    multi-step workflows in the presence of context compaction, tool use and
                    retrieval. We present a controlled evaluation framework, compare several
                    retrieval and memory configurations, and analyze failure modes where objectives
                    are lost, diluted or replaced despite fluent output.
                  </p>
                </div>

                <div className="space-y-2">
                  <h3 className="text-base font-bold text-slate-900 font-serif">
                    2. Experimental design
                  </h3>
                  <p className="text-xs leading-relaxed text-slate-700">
                    We evaluate objective retention in long-running agent workflows by comparing the
                    set of initial objectives <em>O</em>
                    <sub>0</sub> with the set of objectives recovered at evaluation time{' '}
                    <em>O</em>
                    <sub>T</sub>. Retention is defined as:
                  </p>

                  {/* Equation (1) */}
                  <div className="my-3 py-3 px-6 rounded-lg bg-slate-100 border border-slate-300 flex items-center justify-between font-mono text-xs text-slate-900">
                    <span className="mx-auto">
                      R = (retained objectives / initial objectives) = |O_T ∩ O_0| / |O_0|
                    </span>
                    <span className="text-slate-500">(1)</span>
                  </div>

                  {/* Highlighted Claim [3] */}
                  <p className="text-xs leading-relaxed text-slate-700">
                    <span className="inline-block px-1.5 py-0.5 rounded bg-blue-600 text-white font-mono text-[10px] mr-1.5">
                      [3]
                    </span>
                    <mark className="bg-blue-200/80 text-slate-900 font-semibold px-1 rounded">
                      Objective retention must be evaluated independently of fluency.
                    </mark>{' '}
                    An agent can produce coherent and plausible output while failing to retain key
                    objectives from earlier in the workflow. We therefore measure objective
                    retention using explicit criteria rather than subjective judgement of output
                    quality.
                  </p>
                </div>

                {/* Figure 1 Diagram inside the Manuscript */}
                <div className="p-4 rounded-xl bg-white border border-slate-300 space-y-3">
                  <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                    {[
                      {
                        title: 'Context',
                        sub: '(working state)',
                        items: ['task objectives', 'prior decisions', 'retrieved evidence'],
                      },
                      {
                        title: 'Retrieval',
                        sub: '(relevant context)',
                        items: ['semantic search', 'runbook memory', 'external sources'],
                      },
                      {
                        title: 'Verification',
                        sub: '(objective check)',
                        items: ['compare to O_0', 'detect omissions', 'record evidence'],
                      },
                    ].map((box) => (
                      <div
                        key={box.title}
                        className="p-3 rounded-lg bg-slate-50 border border-slate-200 text-xs"
                      >
                        <div className="font-bold text-slate-900">{box.title}</div>
                        <div className="text-[10px] text-slate-500 mb-2">{box.sub}</div>
                        <ul className="list-disc list-inside text-[11px] text-slate-700 space-y-0.5">
                          {box.items.map((it) => (
                            <li key={it}>{it}</li>
                          ))}
                        </ul>
                      </div>
                    ))}
                  </div>
                  <div className="text-[11px] text-slate-600 text-center">
                    <strong>Figure 1.</strong> Overview of the context retention evaluation loop.
                    The agent retrieves relevant context, executes the next step and verifies that
                    the original objectives are still represented in its working state.
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Bottom Research DAG Bar (1. Question -> 2. Sources -> 3. Hypothesis -> 4a/4b -> 5. Compare -> 6. Human review -> 7. Receipt) */}
          <div className="border-t border-white/10 bg-[#080c14] p-3">
            <div className="flex items-center justify-between mb-2 text-xs">
              <div className="flex items-center gap-3">
                <span className="px-2.5 py-1 rounded bg-[#389cff]/20 border border-[#389cff]/40 text-[#389cff] font-mono text-[11px] font-semibold">
                  Research DAG
                </span>
                <span className="text-[#9ba5b4] text-[11px]">Telemetry</span>
                <span className="text-[#9ba5b4] text-[11px]">Receipts (2) ●</span>
              </div>
              <span className="font-mono text-[10px] text-emerald-400">
                ● N³ SQLite Lineage Active
              </span>
            </div>

            <div className="grid grid-cols-2 sm:grid-cols-4 lg:grid-cols-7 gap-2 font-mono text-[10px]">
              {[
                { step: '1. Question', sub: 'Define objectives', state: 'done' },
                { step: '2. Sources', sub: 'Collect & link', state: 'done' },
                { step: '3. Hypothesis', sub: 'Expected behavior', state: 'done' },
                { step: '4a. Control', sub: 'Full context', state: 'done' },
                { step: '4b. Treatment', sub: 'Compaction + RAG', state: 'running' },
                { step: '5. Compare', sub: 'Analyze & validate', state: 'pending' },
                { step: '6. Human review', sub: 'Accept & Receipt', state: 'pending' },
              ].map((node) => (
                <div
                  key={node.step}
                  className={`p-2 rounded-lg border ${
                    node.state === 'running'
                      ? 'bg-amber-500/15 border-amber-400 text-amber-200'
                      : node.state === 'done'
                      ? 'bg-[#0e1624] border-emerald-500/40 text-white'
                      : 'bg-[#0b101a] border-white/10 text-[#9ba5b4]'
                  }`}
                >
                  <div className="font-semibold flex items-center justify-between">
                    <span>{node.step}</span>
                    {node.state === 'done' && <span className="text-emerald-400">✓</span>}
                    {node.state === 'running' && <span className="text-amber-400">●</span>}
                  </div>
                  <div className="text-[9px] opacity-80 truncate mt-0.5">{node.sub}</div>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* RIGHT COLUMN: Research Assistant Chat + Evidence Table + Proposed Runbook -> Decompose to Swarm */}
        <aside className="bg-[#080c14] border-l border-white/8 flex flex-col justify-between min-h-0">
          <div>
            <div className="flex items-center justify-between px-4 py-2.5 border-b border-white/8">
              <span className="text-xs font-semibold text-white flex items-center gap-2">
                <Sparkles className="w-3.5 h-3.5 text-[#389cff]" /> Research assistant
              </span>
              <div className="flex items-center gap-1 text-[11px]">
                {(['Discuss', 'Evidence', 'Agent activity'] as const).map((tab) => (
                  <button
                    key={tab}
                    onClick={() => setActiveAssistantTab(tab)}
                    className={`px-2.5 py-1 rounded font-medium transition cursor-pointer ${
                      activeAssistantTab === tab
                        ? 'bg-[#389cff]/20 text-white border border-[#389cff]/40'
                        : 'text-[#9ba5b4] hover:text-white'
                    }`}
                  >
                    {tab}
                  </button>
                ))}
              </div>
            </div>
          </div>

          {/* Chat Thread & Evidence Cards */}
          <div className="flex-1 overflow-y-auto p-4 space-y-4 text-xs">
            {researchChatTurns.map((turn) => (
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
                  {turn.modelUsed && (
                    <span className="ml-auto text-[9px] font-mono px-1.5 py-0.5 rounded bg-[#389cff]/15 text-[#389cff]">
                      {turn.modelUsed}
                    </span>
                  )}
                </div>

                <div className="pl-8 space-y-2">
                  <p className="text-[#cbd5e1] leading-relaxed">{turn.text}</p>
                  {turn.bullets && (
                    <div className="space-y-1.5">
                      {turn.bullets.map((b, i) => (
                        <div
                          key={i}
                          className="p-2.5 rounded-lg bg-[#0d1422] border border-white/8 text-[11px] text-[#e2e8f0] flex items-start gap-2"
                        >
                          <span className="px-1.5 py-0.2 rounded bg-white/10 font-mono text-[10px] text-[#389cff]">
                            {i + 1}
                          </span>
                          <span>{b}</span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            ))}

            {/* Tool activity 3 steps */}
            <div className="rounded-xl bg-[#0b111c] border border-white/10 p-3 space-y-1.5 font-mono text-[11px]">
              <div className="text-white font-sans font-semibold text-xs mb-1">
                ▾ Tool activity <span className="text-[#9ba5b4] font-normal">3 steps</span>
              </div>
              <div className="flex items-center justify-between text-[#cbd5e1]">
                <span>✓ Read selected passage context-retention-paper.md §2</span>
                <span className="text-[#64748b]">1.2s</span>
              </div>
              <div className="flex items-center justify-between text-[#cbd5e1]">
                <span>✓ Trace 3 linked sources</span>
                <span className="text-[#64748b]">2.8s</span>
              </div>
              <div className="flex items-center justify-between text-[#cbd5e1]">
                <span>✓ Compare experimental conditions (4 configs)</span>
                <span className="text-[#64748b]">1.6s</span>
              </div>
            </div>

            {/* Key evidence from linked sources */}
            <div className="rounded-xl bg-[#0b111c] border border-white/10 p-3 space-y-2">
              <div className="text-xs font-semibold text-white">
                Key evidence from linked sources
              </div>
              <div className="divide-y divide-white/8 text-[11px]">
                {[
                  ['1', 'Objective retention vs fluency', 'context-reten... §2', 'Selected'],
                  ['2', 'Compaction failure modes', 'acceptance-c... p.7', 'Verified'],
                  ['3', 'Retrieval preserves objectives', 'memory.rs L24', 'Linked'],
                ].map(([idx, claim, src, status]) => (
                  <div key={idx} className="py-1.5 flex items-center justify-between gap-2">
                    <span className="text-white truncate">
                      {idx}. {claim}
                    </span>
                    <span className="font-mono text-[10px] text-[#9ba5b4] shrink-0">{src}</span>
                    <span className="font-mono text-[10px] text-emerald-400 shrink-0">
                      ● {status}
                    </span>
                  </div>
                ))}
              </div>
            </div>

            {/* Proposed Runbook Card -> Decompose to Swarm CTA */}
            <div className="rounded-xl bg-gradient-to-br from-[#101c30] to-[#0a101b] border border-[#389cff]/40 p-3.5 space-y-2.5 shadow-lg">
              <div className="flex items-center justify-between">
                <span className="text-xs font-semibold text-white">
                  Proposed runbook:{' '}
                  <span className="text-[#389cff]">Context retention ablation</span>
                </span>
              </div>
              <p className="text-[11px] text-[#9ba5b4] leading-relaxed">
                Run controlled ablations to quantify objective retention across compaction and
                retrieval strategies, then decompose tasks to the 6-agent Swarm.
              </p>
              <div className="flex items-center gap-2">
                <button
                  onClick={decomposeAndAssignToSwarm}
                  className="flex-1 py-2 px-3 rounded-lg bg-[#389cff] hover:bg-[#2589f5] text-white font-semibold text-xs flex items-center justify-center gap-1.5 transition cursor-pointer"
                >
                  <span>Decompose &amp; Assign to Swarm</span>
                  <ArrowRight className="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          </div>

          {/* Bottom Prompt Composer with Cloud Model Selector */}
          <form
            onSubmit={handleChatSubmit}
            className="p-3 border-t border-white/10 bg-[#06090f] space-y-2"
          >
            <input
              type="text"
              value={chatInput}
              onChange={(e) => setChatInput(e.target.value)}
              placeholder="Iron out details with the model before decomposing to the swarm..."
              className="w-full px-3 py-2 rounded-lg bg-[#0c121d] border border-white/12 text-xs text-white focus:border-[#389cff] focus:outline-none"
            />
            <div className="flex items-center justify-between gap-2 text-[11px]">
              <select
                value={selectedCloudModel}
                onChange={(e) => setSelectedCloudModel(e.target.value)}
                className="bg-[#0c121d] border border-white/12 rounded-md px-2 py-1 text-[11px] font-mono text-[#cbd5e1]"
              >
                {CLOUD_MODELS.map((m) => (
                  <option key={m.id} value={m.id}>
                    {m.name} ({m.rate})
                  </option>
                ))}
              </select>

              <button
                type="submit"
                className="px-3.5 py-1.5 rounded-lg bg-[#389cff] hover:bg-[#2589f5] text-white font-semibold text-xs cursor-pointer"
              >
                Send ↗
              </button>
            </div>
          </form>
        </aside>
      </div>
    </div>
  );
};
