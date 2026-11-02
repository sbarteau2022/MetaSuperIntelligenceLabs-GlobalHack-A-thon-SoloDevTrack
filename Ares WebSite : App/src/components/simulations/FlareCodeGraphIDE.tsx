import React, { useState } from 'react';
import { useAtomaStore, type FlareColorBy, type FlareDrawAs } from '../../store/useAtomaStore';
import {
  AlertTriangle,
  ChevronDown,
  ChevronRight,
  Folder,
  GitBranch,
  RefreshCw,
  Sparkles,
  Terminal,
} from '../ui/Icons';

interface DagNodeItem {
  id: string;
  group: 'SHARED' | 'ELECTRON' | 'SRC' | 'SCRIPTS' | 'E2E';
  x: number;
  y: number;
  cx?: string;
  badge?: string;
  hotness: number;
  risk: number;
  clusterColor: string;
  code: string;
  summary: string;
}

const FLARE_NODES: DagNodeItem[] = [
  {
    id: 'app.spec.ts',
    group: 'E2E',
    x: 195,
    y: 36,
    badge: '4⚑',
    hotness: 0.25,
    risk: 0.2,
    clusterColor: '#22c55e',
    summary: 'End-to-end Playwright suite validating DAG canvas layout and review drawer.',
    code: `import { test, expect } from '@playwright/test';\ntest('renders DAG activity lens and review banner', async ({ page }) => {\n  await page.goto('app://flare/graph');\n  await expect(page.locator('[data-lens="Activity"]')).toBeVisible();\n});`,
  },
  {
    id: 'commands.ts',
    group: 'SHARED',
    x: 42,
    y: 132,
    cx: 'cx50',
    hotness: 0.65,
    risk: 0.45,
    clusterColor: '#10b981',
    summary: 'Typed IPC command registry shared between Electron main and renderer.',
    code: `export interface FlareCommandMap {\n  'graph:relayout': { mode: 'Canvas' | 'Wheel' | 'Districts' };\n  'review:open-next': { riskOrder: boolean };\n  'agent:connect': { provider: 'claude' | 'codex' | 'opencode' };\n}`,
  },
  {
    id: 'paths.ts',
    group: 'SHARED',
    x: 42,
    y: 172,
    badge: '0t',
    hotness: 0.15,
    risk: 0.1,
    clusterColor: '#10b981',
    summary: 'Workspace path normalization and cross-platform POSIX resolver.',
    code: `export function normalizeRepoPath(input: string): string {\n  return input.replace(/\\\\/g, '/').replace(/^\\.\\//, '');\n}`,
  },
  {
    id: 'review.ts',
    group: 'SHARED',
    x: 42,
    y: 212,
    hotness: 0.2,
    risk: 0.3,
    clusterColor: '#10b981',
    summary: 'Tracks unreviewed disk writes and orders changed modules by cyclomatic risk.',
    code: `export function rankUnreviewedFiles(files: ChangedFile[]): ChangedFile[] {\n  return [...files].sort((a, b) => b.complexityScore - a.complexityScore);\n}`,
  },
  {
    id: 'smells.ts',
    group: 'SHARED',
    x: 42,
    y: 318,
    cx: 'cx51',
    hotness: 0.72,
    risk: 0.68,
    clusterColor: '#10b981',
    summary: 'Detects cyclic dependencies, god modules, and high-churn coupling.',
    code: `export function detectGraphSmells(graph: DependencyGraph): SmellReport[] {\n  return findStronglyConnectedComponents(graph).filter(scc => scc.length > 1);\n}`,
  },
  {
    id: 'symbols.ts',
    group: 'SHARED',
    x: 42,
    y: 356,
    hotness: 0.35,
    risk: 0.25,
    clusterColor: '#10b981',
    summary: 'AST symbol table extractor for TypeScript export/import bindings.',
    code: `export interface ExportedSymbol {\n  name: string;\n  line: number;\n  isTypeOnly: boolean;\n}`,
  },
  {
    id: 'scanner.ts',
    group: 'SHARED',
    x: 42,
    y: 394,
    hotness: 0.4,
    risk: 0.3,
    clusterColor: '#10b981',
    summary: 'Fast incremental filesystem scanner feeding the N³ dependency graph.',
    code: `export async function scanWorkspaceIncremental(root: string): Promise<ScanSnapshot> {\n  const entries = await fastGlob(['src/**/*.{ts,tsx}', 'shared/**/*.ts'], { cwd: root });\n  return buildSnapshot(entries);\n}`,
  },
  {
    id: 'resolver.ts',
    group: 'SHARED',
    x: 185,
    y: 112,
    hotness: 0.2,
    risk: 0.25,
    clusterColor: '#10b981',
    summary: 'Resolves tsconfig path aliases and barrel re-exports.',
    code: `export function resolveImportTarget(sourceFile: string, specifier: string): string | null {\n  return aliasMatcher(specifier) ?? relativePathResolver(sourceFile, specifier);\n}`,
  },
  {
    id: 'types.ts',
    group: 'SHARED',
    x: 185,
    y: 152,
    badge: '8⚑',
    hotness: 0.55,
    risk: 0.5,
    clusterColor: '#10b981',
    summary: 'Core graph node, edge, lens, and telemetry schema definitions.',
    code: `export interface GraphNodeMetrics {\n  complexity: number;\n  instability: number;\n  unreadEdits: number;\n  lastWrittenAt: number;\n}`,
  },
  {
    id: 'activity.ts',
    group: 'SHARED',
    x: 185,
    y: 356,
    hotness: 0.6,
    risk: 0.4,
    clusterColor: '#10b981',
    summary: 'Computes recency decay so the brightest file reflects the latest disk write.',
    code: `export function computeActivityHeat(lastWriteMs: number, nowMs: number): number {\n  const elapsedMinutes = (nowMs - lastWriteMs) / 60000;\n  return Math.exp(-elapsedMinutes / 45);\n}`,
  },
  {
    id: 'qgraph.ts',
    group: 'SHARED',
    x: 332,
    y: 112,
    cx: 'cx89',
    hotness: 0.88,
    risk: 0.78,
    clusterColor: '#10b981',
    summary: 'Quantum-fast directed graph layout & bezier edge bundler.',
    code: `export class QGraphEngine {\n  layoutSugiyama(nodes: QNode[], edges: QEdge[]): LayoutResult {\n    return this.assignLayers(nodes, edges).minimizeCrossings().routeSplines();\n  }\n}`,
  },
  {
    id: 'insights.ts',
    group: 'SHARED',
    x: 332,
    y: 152,
    cx: 'cx96',
    badge: '2⚑',
    hotness: 1.0,
    risk: 0.92,
    clusterColor: '#10b981',
    summary: 'Primary hotspot: synthesizes risk, churn, and architectural drift insights across 98 nodes.',
    code: `// shared/insights.ts — Written 2m ago by agent claude-code\nexport function computeWorkspaceInsights(graph: QGraph): InsightSummary {\n  const hotspots = graph.nodes.filter(n => n.complexity > 85 && n.churn > 0.6);\n  const cycles = graph.findCycles();\n  return { hotspots, cycles, riskOrderedFiles: rankByRisk(hotspots) };\n}`,
  },
  {
    id: 'parser.ts',
    group: 'SHARED',
    x: 332,
    y: 192,
    cx: 'cx120',
    badge: '4⚑',
    hotness: 0.95,
    risk: 0.96,
    clusterColor: '#10b981',
    summary: 'High-complexity TypeScript/TSX AST import & call-graph parser (cx120).',
    code: `export function parseModuleDependencies(filePath: string, sourceText: string): ParsedModule {\n  const ast = ts.createSourceFile(filePath, sourceText, ts.ScriptTarget.Latest, true);\n  return extractImportsExportsAndCalls(ast);\n}`,
  },
  {
    id: 'store.ts',
    group: 'ELECTRON',
    x: 472,
    y: 368,
    hotness: 0.58,
    risk: 0.52,
    clusterColor: '#f59e0b',
    summary: 'Persistent Electron state store syncing N³ SQLite receipts with local disk.',
    code: `export class ElectronSessionStore {\n  saveReviewCheckpoint(commitSha: string, reviewedFiles: string[]): void {\n    this.sqlite.prepare('INSERT INTO checkpoints VALUES (?, ?)').run(commitSha, JSON.stringify(reviewedFiles));\n  }\n}`,
  },
  {
    id: 'git.ts',
    group: 'ELECTRON',
    x: 562,
    y: 318,
    hotness: 0.76,
    risk: 0.64,
    clusterColor: '#f59e0b',
    summary: 'Git worktree diffing and shadow branch comparison engine.',
    code: `export async function getUnreviewedDiskChanges(cwd: string): Promise<GitFileDiff[]> {\n  const raw = await execGit(['diff', '--numstat', 'HEAD'], cwd);\n  return parseNumstat(raw);\n}`,
  },
  {
    id: 'shadow.ts',
    group: 'ELECTRON',
    x: 562,
    y: 356,
    hotness: 0.64,
    risk: 0.58,
    clusterColor: '#f59e0b',
    summary: 'Maintains shadow snapshots so agent writes can be rolled back atomically.',
    code: `export async function snapshotBeforeAgentWrite(filePath: string): Promise<ShadowReceipt> {\n  return shadowVault.capture(filePath);\n}`,
  },
  {
    id: 'session.ts',
    group: 'ELECTRON',
    x: 682,
    y: 334,
    cx: 'cx131',
    badge: '0t',
    hotness: 0.91,
    risk: 0.89,
    clusterColor: '#f59e0b',
    summary: 'Coordinates live agent PTY sessions, file watchers, and review gates (cx131).',
    code: `export class AgentSessionCoordinator {\n  async onFileWrittenToDisk(path: string) {\n    this.dirtyFiles.add(path);\n    this.broadcastGraphUpdate({ highlightRecent: path });\n  }\n}`,
  },
  {
    id: 'renderModel.ts',
    group: 'SRC',
    x: 805,
    y: 392,
    hotness: 0.78,
    risk: 0.62,
    clusterColor: '#389cff',
    summary: 'Transforms QGraph topology into GPU-accelerated canvas bezier primitives.',
    code: `export function buildCanvasRenderModel(graph: QGraph, lens: LensMode): RenderModel {\n  return { nodes: mapNodesToLens(graph.nodes, lens), splines: routeBundledSplines(graph.edges) };\n}`,
  },
  {
    id: 'api.ts',
    group: 'SRC',
    x: 805,
    y: 492,
    badge: '0t',
    hotness: 0.82,
    risk: 0.54,
    clusterColor: '#389cff',
    summary: 'Renderer bridge connecting React UI panels to the Electron session host.',
    code: `export const flareApi = {\n  reviewNextRiskFile: () => window.electron.invoke('review:open-next', { riskOrder: true }),\n  relayoutGraph: (mode: DrawMode) => window.electron.invoke('graph:relayout', { mode }),\n};`,
  },
  {
    id: 'FileTree.tsx',
    group: 'SRC',
    x: 930,
    y: 378,
    badge: '0t',
    hotness: 0.74,
    risk: 0.42,
    clusterColor: '#389cff',
    summary: 'Explorer tree component with live amber activity dots for agent-modified files.',
    code: `export function FileTree({ files, selected, onSelect }: FileTreeProps) {\n  return <aside className="explorer-tree">{files.map(renderFileRow)}</aside>;\n}`,
  },
  {
    id: 'Timeline.tsx',
    group: 'SRC',
    x: 930,
    y: 478,
    badge: '0t',
    hotness: 0.7,
    risk: 0.38,
    clusterColor: '#389cff',
    summary: 'Session edit scrub bar tracking agent writes over the last 60 minutes.',
    code: `export function SessionTimeline({ edits }: { edits: DiskEditEvent[] }) {\n  return <div className="timeline-strip">{edits.map(renderHeatTick)}</div>;\n}`,
  },
  {
    id: 'DetailsPanel.tsx',
    group: 'SRC',
    x: 1055,
    y: 362,
    badge: '0t',
    hotness: 0.68,
    risk: 0.45,
    clusterColor: '#389cff',
    summary: 'Side inspector displaying cyclomatic complexity, test coverage, and AST imports.',
    code: `export function DetailsPanel({ node }: { node: GraphNode }) {\n  return <section>{node.id} — complexity cx{node.complexity}</section>;\n}`,
  },
  {
    id: 'TerminalPanel.tsx',
    group: 'SRC',
    x: 1055,
    y: 492,
    cx: 'cx57',
    badge: '0t',
    hotness: 0.85,
    risk: 0.66,
    clusterColor: '#389cff',
    summary: 'Integrated PowerShell / bash PTY terminal hosting claude, codex, and opencode agents.',
    code: `export function TerminalPanel() {\n  return <div className="pty-host">PS C:\\Users\\malte\\AppData\\Local\\Temp\\fl-shot-JnGMw9\\flare&gt;</div>;\n}`,
  },
  {
    id: 'CanvasView.tsx',
    group: 'SRC',
    x: 1175,
    y: 426,
    cx: 'cx212',
    badge: '3⚑',
    hotness: 0.94,
    risk: 0.91,
    clusterColor: '#389cff',
    summary: 'Main interactive dependency graph canvas with pan, zoom, and lens shaders (cx212).',
    code: `export function CanvasView({ model, lens }: CanvasViewProps) {\n  return <svg className="dag-canvas">{model.splines.map(renderSpline)}</svg>;\n}`,
  },
];

const FLARE_EDGES: Array<[string, string]> = [
  ['commands.ts', 'types.ts'],
  ['smells.ts', 'qgraph.ts'],
  ['symbols.ts', 'parser.ts'],
  ['scanner.ts', 'parser.ts'],
  ['scanner.ts', 'activity.ts'],
  ['types.ts', 'qgraph.ts'],
  ['types.ts', 'insights.ts'],
  ['types.ts', 'parser.ts'],
  ['qgraph.ts', 'renderModel.ts'],
  ['qgraph.ts', 'CanvasView.tsx'],
  ['insights.ts', 'git.ts'],
  ['insights.ts', 'shadow.ts'],
  ['insights.ts', 'session.ts'],
  ['insights.ts', 'renderModel.ts'],
  ['insights.ts', 'FileTree.tsx'],
  ['insights.ts', 'DetailsPanel.tsx'],
  ['insights.ts', 'CanvasView.tsx'],
  ['insights.ts', 'TerminalPanel.tsx'],
  ['parser.ts', 'store.ts'],
  ['parser.ts', 'git.ts'],
  ['parser.ts', 'session.ts'],
  ['parser.ts', 'api.ts'],
  ['parser.ts', 'Timeline.tsx'],
  ['activity.ts', 'session.ts'],
  ['activity.ts', 'Timeline.tsx'],
  ['git.ts', 'session.ts'],
  ['shadow.ts', 'session.ts'],
  ['session.ts', 'renderModel.ts'],
  ['session.ts', 'api.ts'],
  ['session.ts', 'FileTree.tsx'],
  ['session.ts', 'TerminalPanel.tsx'],
  ['session.ts', 'CanvasView.tsx'],
  ['renderModel.ts', 'DetailsPanel.tsx'],
  ['renderModel.ts', 'CanvasView.tsx'],
  ['api.ts', 'Timeline.tsx'],
  ['api.ts', 'TerminalPanel.tsx'],
  ['FileTree.tsx', 'DetailsPanel.tsx'],
  ['FileTree.tsx', 'CanvasView.tsx'],
  ['DetailsPanel.tsx', 'CanvasView.tsx'],
];

const EXPLORER_TREE = [
  { folder: 'e2e', files: [{ name: 'app.spec.ts', dirty: false }] },
  {
    folder: 'electron',
    files: [
      { name: 'main.ts', dirty: false },
      { name: 'preload.ts', dirty: false },
      { name: 'session.ts', dirty: true },
    ],
  },
  {
    folder: 'scripts',
    files: [
      { name: '_shot-readme.mjs', dirty: false },
      { name: 'build-main.mjs', dirty: false },
      { name: 'dev.mjs', dirty: false },
      { name: 'shot-insights.mjs', dirty: false },
    ],
  },
  {
    folder: 'shared',
    files: [
      { name: 'activity.ts', dirty: false },
      { name: 'commands.ts', dirty: false },
      { name: 'graph.ts', dirty: true },
      { name: 'insights.ts', dirty: true },
      { name: 'parser.ts', dirty: true },
      { name: 'qgraph.ts', dirty: true },
      { name: 'review.ts', dirty: false },
      { name: 'scanner.ts', dirty: false },
      { name: 'smells.ts', dirty: false },
      { name: 'symbols.ts', dirty: false },
      { name: 'types.ts', dirty: false },
    ],
  },
  {
    folder: 'src',
    files: [
      { name: 'CanvasView.tsx', dirty: true },
      { name: 'DetailsPanel.tsx', dirty: false },
      { name: 'FileTree.tsx', dirty: false },
      { name: 'TerminalPanel.tsx', dirty: true },
      { name: 'Timeline.tsx', dirty: false },
      { name: 'api.ts', dirty: false },
      { name: 'renderModel.ts', dirty: false },
    ],
  },
];

export const FlareCodeGraphIDE: React.FC = () => {
  const {
    flareColorBy,
    setFlareColorBy,
    flareDrawAs,
    setFlareDrawAs,
    flareSelectedNode,
    setFlareSelectedNode,
    flareHoveredNode,
    setFlareHoveredNode,
    flareBannerDismissed,
    setFlareBannerDismissed,
    flareZoom,
    setFlareZoom,
  } = useAtomaStore();

  const [activeTopTab, setActiveTopTab] = useState<'Graph' | 'Tasks' | 'Review' | 'Insights'>('Graph');
  const [terminalInput, setTerminalInput] = useState('');
  const [terminalHistory, setTerminalHistory] = useState<string[]>([
    'Windows PowerShell',
    'Copyright (C) Microsoft Corporation. Tutti i diritti riservati.',
    'Installa la versione più recente di PowerShell per nuove funzionalità e miglioramenti. https://aka.ms/PSWindows',
    'PS C:\\Users\\malte\\AppData\\Local\\Temp\\fl-shot-JnGMw9\\flare> claude --model sonnet-4 "audit shared/insights.ts and shared/parser.ts risk hotspots"',
    '✓ Agent connected to MCP /flare-9a88a5 · 7 files written to shadow disk · 0 cyclic regressions',
  ]);
  const [openFolders, setOpenFolders] = useState<Record<string, boolean>>({
    e2e: true,
    electron: true,
    scripts: false,
    shared: true,
    src: true,
  });

  const activeNode =
    FLARE_NODES.find((n) => n.id === (flareHoveredNode || flareSelectedNode)) || FLARE_NODES[11];

  const handleReviewNext = () => {
    const riskOrder = ['parser.ts', 'insights.ts', 'CanvasView.tsx', 'session.ts', 'qgraph.ts'];
    const currentIdx = riskOrder.indexOf(flareSelectedNode);
    const next = riskOrder[(currentIdx + 1) % riskOrder.length];
    setFlareSelectedNode(next);
  };

  const handleTerminalSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!terminalInput.trim()) return;
    setTerminalHistory((prev) => [
      ...prev,
      `PS C:\\Users\\malte\\AppData\\Local\\Temp\\fl-shot-JnGMw9\\flare> ${terminalInput}`,
      `[ares-harness] Executed "${terminalInput}" across 98 nodes (220 edges) — N³ receipt committed.`,
    ]);
    setTerminalInput('');
  };

  const getNodeAccent = (node: DagNodeItem) => {
    if (flareColorBy === 'Clusters') return node.clusterColor;
    if (flareColorBy === 'Risk' || flareColorBy === 'Hotspots') {
      return node.risk > 0.75 ? '#ef4444' : node.risk > 0.45 ? '#f59e0b' : '#3b82f6';
    }
    if (flareColorBy === 'Tests') {
      return node.badge?.includes('0t') ? '#f97316' : '#22c55e';
    }
    return node.hotness > 0.8
      ? '#f59e0b'
      : node.hotness > 0.55
      ? '#d97706'
      : 'rgba(148, 163, 184, 0.35)';
  };

  return (
    <div className="w-full rounded-xl overflow-hidden border border-white/12 bg-[#0b0c0e] text-[#e2e8f0] shadow-2xl flex flex-col select-none">
      {!flareBannerDismissed && (
        <div className="flex flex-wrap items-center gap-2.5 px-4 py-2 bg-[#1c160b] border-b border-amber-500/30 text-xs">
          <AlertTriangle className="w-3.5 h-3.5 text-amber-400 shrink-0" />
          <span className="text-amber-200/90">
            7 files changed since last review —{' '}
            <strong className="text-amber-300 font-semibold">already written to disk</strong>. Ordered
            by risk.
          </span>
          <div className="flex items-center gap-1.5 ml-2">
            <button
              onClick={handleReviewNext}
              className="px-2.5 py-1 rounded bg-amber-500/20 hover:bg-amber-500/30 border border-amber-400/40 text-amber-200 font-medium transition"
            >
              Review next
            </button>
            <button
              onClick={() => setFlareBannerDismissed(true)}
              className="px-2.5 py-1 rounded bg-white/5 hover:bg-white/10 border border-white/10 text-[#9ba5b4] transition"
            >
              Dismiss all
            </button>
            <button
              onClick={() => setActiveTopTab('Review')}
              className="px-2.5 py-1 rounded bg-white/5 hover:bg-white/10 border border-white/10 text-[#e2e8f0] transition"
            >
              Open review →
            </button>
          </div>
          <span className="ml-auto font-mono text-[11px] text-amber-400/80 hidden sm:inline">
            Active hotspot: {flareSelectedNode}
          </span>
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-[230px_1fr] min-h-[660px]">
        <aside className="bg-[#0e1014] border-r border-white/8 flex flex-col">
          <div className="flex items-center justify-between px-3 py-2.5 border-b border-white/8 text-[11px] font-mono uppercase tracking-wider text-[#8e96a8]">
            <span>EXPLORER</span>
            <div className="flex items-center gap-1.5 text-[#64748b]">
              <span>⊞</span>
              <span>⊕</span>
              <span>↻</span>
            </div>
          </div>

          <div className="flex-1 overflow-y-auto py-2 px-1.5 space-y-1 text-xs font-mono">
            {EXPLORER_TREE.map((group) => {
              const isOpen = openFolders[group.folder];
              return (
                <div key={group.folder}>
                  <button
                    onClick={() =>
                      setOpenFolders((prev) => ({ ...prev, [group.folder]: !prev[group.folder] }))
                    }
                    className="w-full flex items-center gap-1.5 px-2 py-1 rounded hover:bg-white/5 text-[#cbd5e1] text-left"
                  >
                    {isOpen ? (
                      <ChevronDown className="w-3.5 h-3.5 text-[#64748b]" />
                    ) : (
                      <ChevronRight className="w-3.5 h-3.5 text-[#64748b]" />
                    )}
                    <Folder className="w-3.5 h-3.5 text-amber-400/90" />
                    <span>{group.folder}</span>
                  </button>
                  {isOpen && (
                    <div className="pl-5 space-y-0.5 mt-0.5">
                      {group.files.map((f) => {
                        const isSelected = flareSelectedNode === f.name;
                        return (
                          <button
                            key={f.name}
                            onClick={() => setFlareSelectedNode(f.name)}
                            className={`w-full flex items-center justify-between px-2 py-1 rounded text-[11px] transition ${
                              isSelected
                                ? 'bg-amber-500/20 text-amber-200 border border-amber-500/40'
                                : 'text-[#9ba5b4] hover:bg-white/5 hover:text-white'
                            }`}
                          >
                            <span className="flex items-center gap-1.5 truncate">
                              <span className="text-[10px] text-[#389cff]">T</span>
                              <span className="truncate">{f.name}</span>
                            </span>
                            {f.dirty && (
                              <span className="w-2 h-2 rounded-full bg-amber-400 shrink-0" />
                            )}
                          </button>
                        );
                      })}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </aside>

        <div className="flex flex-col min-w-0 bg-[#0b0d11]">
          <div className="flex items-center gap-1 px-3 pt-2 border-b border-white/8 bg-[#0e1015] text-xs">
            {(
              [
                { id: 'Graph', label: '◆ Graph' },
                { id: 'Tasks', label: '☰ Tasks' },
                { id: 'Review', label: '✓ Review', count: 1 },
                { id: 'Insights', label: '◆ Insights', count: 2 },
              ] as const
            ).map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTopTab(tab.id)}
                className={`px-3.5 py-2 rounded-t-md font-medium flex items-center gap-1.5 border-b-2 transition ${
                  activeTopTab === tab.id
                    ? 'bg-[#151821] text-white border-amber-400'
                    : 'text-[#8e96a8] border-transparent hover:text-white'
                }`}
              >
                <span>{tab.label}</span>
                {'count' in tab && (
                  <span className="px-1.5 py-0.2 rounded bg-amber-500/20 text-amber-300 font-mono text-[10px]">
                    {tab.count}
                  </span>
                )}
              </button>
            ))}
          </div>

          <div className="p-2.5 border-b border-white/8 bg-[#101319] space-y-2">
            <div className="flex flex-wrap items-center gap-2 text-[11px]">
              <div className="flex items-center bg-[#161922] border border-white/10 rounded-md p-0.5">
                <span className="px-2 py-1 font-mono text-[10px] text-[#64748b] uppercase">
                  COLOUR BY
                </span>
                {(
                  [
                    'Clusters',
                    'Activity',
                    'Hotspots',
                    'Risk',
                    'Tests',
                    'Instability',
                    'Unread',
                    'Cycles',
                  ] as FlareColorBy[]
                ).map((mode) => (
                  <button
                    key={mode}
                    onClick={() => setFlareColorBy(mode)}
                    className={`px-2 py-1 rounded text-[11px] font-medium transition ${
                      flareColorBy === mode
                        ? 'bg-[#d97706] text-white shadow'
                        : 'text-[#9ba5b4] hover:text-white'
                    }`}
                  >
                    {mode}
                  </button>
                ))}
              </div>

              <div className="flex items-center bg-[#161922] border border-white/10 rounded-md p-0.5">
                <span className="px-2 py-1 font-mono text-[10px] text-[#64748b] uppercase">
                  DRAW AS
                </span>
                {(['Canvas', 'Wheel', 'Districts'] as FlareDrawAs[]).map((draw) => (
                  <button
                    key={draw}
                    onClick={() => setFlareDrawAs(draw)}
                    className={`px-2 py-1 rounded text-[11px] font-medium transition ${
                      flareDrawAs === draw
                        ? 'bg-[#d97706] text-white'
                        : 'text-[#9ba5b4] hover:text-white'
                    }`}
                  >
                    {draw}
                  </button>
                ))}
              </div>

              <div className="flex items-center gap-1 ml-auto">
                <button
                  onClick={() => setFlareZoom(72)}
                  className="px-2 py-1 rounded bg-[#161922] border border-white/10 text-[#9ba5b4] hover:text-white flex items-center gap-1"
                >
                  <RefreshCw className="w-3 h-3" /> Re-layout
                </button>
                <div className="flex items-center bg-[#161922] border border-white/10 rounded px-1.5 py-0.5 font-mono text-[11px]">
                  <button
                    onClick={() => setFlareZoom(Math.max(50, flareZoom - 8))}
                    className="px-1.5 text-[#9ba5b4] hover:text-white"
                  >
                    −
                  </button>
                  <span className="px-1.5 text-white">{flareZoom}%</span>
                  <button
                    onClick={() => setFlareZoom(Math.min(110, flareZoom + 8))}
                    className="px-1.5 text-[#9ba5b4] hover:text-white"
                  >
                    +
                  </button>
                </div>
              </div>
            </div>

            <div className="flex flex-wrap items-center gap-3 text-[11px]">
              <div className="inline-flex items-center gap-2 px-2.5 py-1 rounded bg-[#171a22] border border-white/8 text-[#cbd5e1]">
                <strong className="text-white">{flareColorBy}</strong>
                <span className="text-[#9ba5b4]">
                  The session’s changes in order — the brightest file was written last.
                </span>
                <span className="text-[10px] text-[#64748b] ml-1">earlier</span>
                <span className="w-14 h-2 rounded-full bg-gradient-to-r from-[#3b2814] via-[#b45309] to-[#fbbf24]" />
                <span className="text-[10px] text-amber-300">latest</span>
              </div>

              <div className="flex items-center gap-2.5 font-mono text-[11px] text-[#cbd5e1]">
                <span className="text-[#64748b]">FOLDERS 0/5 FOLDED</span>
                <span className="inline-flex items-center gap-1">
                  <i className="w-2 h-2 rounded-full bg-[#389cff]" /> src (33)
                </span>
                <span className="inline-flex items-center gap-1">
                  <i className="w-2 h-2 rounded-full bg-[#f97316]" /> tests (25)
                </span>
                <span className="inline-flex items-center gap-1">
                  <i className="w-2 h-2 rounded-full bg-[#10b981]" /> shared (15)
                </span>
                <span className="inline-flex items-center gap-1">
                  <i className="w-2 h-2 rounded-full bg-[#eab308]" /> electron (12)
                </span>
              </div>
            </div>
          </div>

          <div className="relative flex-1 overflow-x-auto overflow-y-hidden linear-dot-bg min-h-[380px]">
            <div
              className="relative w-[1320px] h-[540px] origin-top-left transition-transform duration-200"
              style={{ transform: `scale(${flareZoom / 88})` }}
            >
              <div className="absolute left-[42px] top-[92px] font-mono text-[10px] tracking-widest text-[#10b981] uppercase">
                SHARED
              </div>
              <div className="absolute left-[472px] top-[288px] font-mono text-[10px] tracking-widest text-[#f59e0b] uppercase">
                ELECTRON
              </div>
              <div className="absolute left-[805px] top-[345px] font-mono text-[10px] tracking-widest text-[#389cff] uppercase">
                SRC
              </div>

              <svg className="absolute inset-0 w-[1320px] h-[540px] pointer-events-none">
                <defs>
                  <linearGradient id="flare-edge-hot" x1="0%" y1="0%" x2="100%" y2="0%">
                    <stop offset="0%" stopColor="#fbbf24" stopOpacity="0.95" />
                    <stop offset="50%" stopColor="#f59e0b" stopOpacity="0.8" />
                    <stop offset="100%" stopColor="#d97706" stopOpacity="0.65" />
                  </linearGradient>
                  <linearGradient id="flare-edge-dim" x1="0%" y1="0%" x2="100%" y2="0%">
                    <stop offset="0%" stopColor="#94a3b8" stopOpacity="0.24" />
                    <stop offset="100%" stopColor="#64748b" stopOpacity="0.14" />
                  </linearGradient>
                </defs>

                {FLARE_EDGES.map(([fromId, toId], idx) => {
                  const fromNode = FLARE_NODES.find((n) => n.id === fromId);
                  const toNode = FLARE_NODES.find((n) => n.id === toId);
                  if (!fromNode || !toNode) return null;

                  const x1 = fromNode.x + 118;
                  const y1 = fromNode.y + 16;
                  const x2 = toNode.x;
                  const y2 = toNode.y + 16;
                  const dx = Math.max(60, (x2 - x1) * 0.55);
                  const d = `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`;

                  const focusId = flareHoveredNode || flareSelectedNode;
                  const isConnected = fromId === focusId || toId === focusId;
                  const isHotOrigin =
                    fromId === 'insights.ts' || fromId === 'parser.ts' || fromId === 'qgraph.ts';

                  return (
                    <path
                      key={idx}
                      d={d}
                      fill="none"
                      stroke={
                        isConnected || isHotOrigin
                          ? 'url(#flare-edge-hot)'
                          : 'url(#flare-edge-dim)'
                      }
                      strokeWidth={isConnected ? 2.4 : isHotOrigin ? 1.7 : 1.1}
                      opacity={
                        flareHoveredNode
                          ? isConnected
                            ? 1
                            : 0.18
                          : isHotOrigin
                          ? 0.88
                          : 0.35
                      }
                    />
                  );
                })}
              </svg>

              {FLARE_NODES.map((node) => {
                const isSelected = flareSelectedNode === node.id;
                const isHovered = flareHoveredNode === node.id;
                const borderColor = getNodeAccent(node);

                return (
                  <button
                    key={node.id}
                    onClick={() => setFlareSelectedNode(node.id)}
                    onMouseEnter={() => setFlareHoveredNode(node.id)}
                    onMouseLeave={() => setFlareHoveredNode(null)}
                    style={{
                      left: `${node.x}px`,
                      top: `${node.y}px`,
                      borderColor: isSelected || isHovered ? '#fbbf24' : borderColor,
                    }}
                    className={`absolute w-[118px] h-[32px] px-2 rounded-md bg-[#14171f]/95 border flex items-center justify-between text-[11px] font-mono transition-all ${
                      isSelected
                        ? 'ring-2 ring-amber-400/50 shadow-[0_0_24px_rgba(245,158,11,0.35)] text-white z-20 scale-105'
                        : 'text-[#cbd5e1] hover:text-white hover:scale-105 z-10'
                    }`}
                  >
                    <span className="truncate font-medium">{node.id}</span>
                    <span className="flex items-center gap-1 shrink-0 ml-1">
                      {node.cx && (
                        <span className="px-1 py-0.2 rounded bg-amber-500/20 text-amber-300 text-[9px]">
                          {node.cx}
                        </span>
                      )}
                      {node.badge && (
                        <span className="text-[9px] text-rose-400/90">{node.badge}</span>
                      )}
                    </span>
                  </button>
                );
              })}
            </div>

            <div className="absolute top-3 right-3 w-96 rounded-lg bg-[#11141c]/95 border border-white/12 shadow-2xl backdrop-blur-xl p-3 space-y-2 z-30">
              <div className="flex items-center justify-between border-b border-white/8 pb-2">
                <div className="flex items-center gap-2">
                  <span className="w-2.5 h-2.5 rounded-full bg-amber-400" />
                  <span className="font-mono text-xs font-semibold text-white">
                    {activeNode.id}
                  </span>
                  <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-white/8 text-[#9ba5b4]">
                    {activeNode.group}
                  </span>
                </div>
                {activeNode.cx && (
                  <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-amber-500/20 text-amber-300 border border-amber-500/30">
                    Complexity {activeNode.cx}
                  </span>
                )}
              </div>
              <p className="text-[11px] text-[#9ba5b4] leading-relaxed">{activeNode.summary}</p>
              <pre className="p-2.5 rounded bg-[#090b10] border border-white/8 text-[10px] font-mono text-[#cbd5e1] overflow-x-auto leading-relaxed">
                {activeNode.code}
              </pre>
            </div>
          </div>

          <div className="border-t border-white/10 bg-[#090a0d]">
            <div className="flex items-center gap-3 px-3 py-1.5 border-b border-white/8 bg-[#101218] text-xs">
              <span className="px-2.5 py-1 rounded bg-[#1a1d26] text-white font-mono text-[11px] flex items-center gap-1.5 border border-white/10">
                <Terminal className="w-3 h-3 text-amber-400" /> terminal 1 ×
              </span>
              <span className="text-[#64748b]">+</span>
              <span className="text-[#9ba5b4] text-[11px]">▤ Commands</span>
              <span className="text-[#64748b] text-[11px] font-mono">
                run your agent here — claude · codex · opencode
              </span>
              <button
                onClick={() =>
                  setTerminalHistory((prev) => [
                    ...prev,
                    '✓ Connected ARES Spec Ops local harness to MCP /flare-9a88a5 (SQLite N³ active)',
                  ])
                }
                className="ml-auto px-2.5 py-1 rounded bg-[#181c26] hover:bg-[#389cff] hover:text-white border border-white/12 text-[#cbd5e1] text-[11px] font-medium flex items-center gap-1 transition"
              >
                <Sparkles className="w-3 h-3 text-[#389cff]" /> Connect agent
              </button>
            </div>

            <div className="p-3 font-mono text-[11px] space-y-1 max-h-36 overflow-y-auto text-[#cbd5e1]">
              {terminalHistory.map((line, i) => (
                <div
                  key={i}
                  className={
                    line.startsWith('✓')
                      ? 'text-emerald-400'
                      : line.startsWith('[ares-harness]')
                      ? 'text-[#389cff]'
                      : 'text-[#cbd5e1]'
                  }
                >
                  {line}
                </div>
              ))}
              <form onSubmit={handleTerminalSubmit} className="flex items-center gap-2 pt-1">
                <span className="text-amber-300 shrink-0">
                  PS C:\Users\malte\AppData\Local\Temp\fl-shot-JnGMw9\flare&gt;
                </span>
                <input
                  type="text"
                  value={terminalInput}
                  onChange={(e) => setTerminalInput(e.target.value)}
                  placeholder="Type a command or agent prompt and press Enter..."
                  className="bg-transparent text-white focus:outline-none w-full"
                />
              </form>
            </div>

            <div className="flex items-center justify-between px-3 py-1 border-t border-white/8 bg-[#07080b] font-mono text-[10px] text-[#8e96a8]">
              <div className="flex items-center gap-4">
                <span className="flex items-center gap-1 text-[#cbd5e1]">
                  <GitBranch className="w-3 h-3 text-[#389cff]" /> main
                </span>
                <span>98 nodes · 220 edges</span>
                <span>7 changed on disk</span>
                <span className="text-amber-400 font-semibold">7 to review</span>
              </div>
              <div className="flex items-center gap-4">
                <span className="text-emerald-400">✦ MCP /flare-9a88a5</span>
                <span className="hidden sm:inline">
                  C:\Users\malte\AppData\Local\Temp\fl-shot-JnGMw9\flare
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
