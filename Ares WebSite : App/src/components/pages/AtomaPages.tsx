import React, { Suspense, lazy, useRef, useState } from 'react';
import gsap from 'gsap';
import { useGSAP } from '@gsap/react';
import { useReducedMotion } from '../../hooks/useReducedMotion';

gsap.registerPlugin(useGSAP);
import {
  useAtomaStore,
  CLOUD_MODELS,
  type CloudWorkspaceTab,
  type SimulationTabId,
} from '../../store/useAtomaStore';
import { AtomaLogoMark, BranchingWorkflowConceptSVG } from '../brand/AtomaBrandAssets';
import Hero from '../hero/Hero';
import ProofSection from '../proof/ProofSection';
import ArchitectureSection from '../architecture/ArchitectureSection';
import {
  ArrowRight,
  Boxes,
  CheckCircle2,
  Database,
  FileText,
  GitBranch,
  Layers,
  Search,
  Sparkles,
  Workflow,
} from '../ui/Icons';

// ── Heavy interactive labs load on demand (three.js / @xyflow stay out of
//    the initial page bundle; each tab pulls its chunk only when opened). ──
const AtomaSculptureCanvas = lazy(() =>
  import('../webgl/AtomaSculptures').then((m) => ({ default: m.AtomaSculptureCanvas })),
);
const ForceGraph3D = lazy(() =>
  import('../webgl/ForceGraph3D').then((m) => ({ default: m.ForceGraph3D })),
);
const FlareCodeGraphIDE = lazy(() =>
  import('../simulations/FlareCodeGraphIDE').then((m) => ({ default: m.FlareCodeGraphIDE })),
);
const OrbitalAgentOS = lazy(() =>
  import('../simulations/OrbitalAgentOS').then((m) => ({ default: m.OrbitalAgentOS })),
);
const AresRunbookLab = lazy(() =>
  import('../simulations/AresRunbookLab').then((m) => ({ default: m.AresRunbookLab })),
);
const SwarmDAG = lazy(() => import('../proof/SwarmDAG'));

function LabFallback({ label = 'Loading interactive lab…' }: { label?: string }) {
  return (
    <div
      className="rounded-3xl border border-white/10 bg-[#05070c] p-10 min-h-[480px] flex flex-col items-center justify-center gap-4"
      aria-busy="true"
      aria-label={label}
    >
      <div className="h-2.5 w-48 rounded-full bg-white/10 animate-pulse" />
      <div className="h-2.5 w-32 rounded-full bg-white/8 animate-pulse" />
      <div className="font-mono text-[11px] tracking-[0.2em] text-[#9ba5b4] uppercase">{label}</div>
    </div>
  );
}

export const InteractiveSimulationShowcase: React.FC = () => {
  const { activeSimTab, setActiveSimTab, setActivePage, setCloudTab } = useAtomaStore();

  const tabs: Array<{
    id: SimulationTabId;
    label: string;
    badge: string;
    subtitle: string;
  }> = [
    {
      id: 'flare-dag',
      label: '01 · Flare Codebase DAG & IDE',
      badge: '98 Nodes · 220 Edges',
      subtitle: 'Interactive Activity, Risk & Hotspot dependency graph with live code & agent PTY.',
    },
    {
      id: 'orbital-os',
      label: '02 · Orbital Agent OS',
      badge: '12 Agents · WebGPU',
      subtitle: 'Autonomous pipeline orchestrator with dual Rust/C++ editors & real-time telemetry.',
    },
    {
      id: 'ares-runbook',
      label: '03 · ARES Runbook Explorer',
      badge: 'N³ SQLite Receipts',
      subtitle: 'Step-by-step runbook contract simulation from Scope to cryptographic Receipt.',
    },
    {
      id: 'galaxy-3d',
      label: '04 · 3D Galaxy & Nebula Vault',
      badge: '2,400 Nodes · R3F WebGL',
      subtitle: 'Real-time 3D force-directed point-cloud with PageRank, Physics & Neural Cortex presets.',
    },
    {
      id: 'xyflow-swarm',
      label: '05 · ReactFlow Swarm Holonomy DAG',
      badge: '@xyflow/react · Live',
      subtitle: 'Draggable multi-agent ReactFlow DAG with animated edges and telemetry proof table.',
    },
  ];

  return (
    <section id="interactive-simulations" className="py-20 border-t border-white/8">
      <div className="max-w-[1360px] mx-auto px-6">
        <div className="flex flex-col lg:flex-row lg:items-end justify-between gap-6 mb-8">
          <div>
            <div className="font-mono text-[11px] uppercase tracking-[0.2em] text-[#389cff] mb-2">
              INTERACTIVE PRODUCT SIMULATIONS · STATE-OF-THE-ART ENGINE
            </div>
            <h2 className="font-display text-3xl sm:text-5xl font-semibold tracking-tight text-white">
              Inspect the loop. Live in the browser.
            </h2>
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={() => {
                setCloudTab('code');
                setActivePage('cloud-app');
              }}
              className="px-4 py-2.5 rounded-xl bg-[#389cff] hover:bg-[#2589f5] text-white text-xs font-semibold flex items-center gap-2 shadow-[0_0_25px_rgba(56,156,255,0.5)] cursor-pointer"
            >
              <span>Open Full Authenticated Cloud App</span>
              <ArrowRight className="w-4 h-4" />
            </button>
          </div>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-2.5 mb-6">
          {tabs.map((t) => {
            const active = activeSimTab === t.id;
            return (
              <button
                key={t.id}
                onClick={() => setActiveSimTab(t.id)}
                className={`p-3.5 rounded-xl text-left transition-all border cursor-pointer ${
                  active
                    ? 'bg-[#101622] border-[#389cff] shadow-[0_0_30px_-8px_rgba(56,156,255,0.45)]'
                    : 'bg-[#0a0d13] border-white/8 hover:border-white/20'
                }`}
              >
                <div className="flex items-center justify-between gap-2 mb-1.5">
                  <span className="font-mono text-[11px] font-semibold text-white">{t.label}</span>
                </div>
                <div className="mb-1.5">
                  <span
                    className={`px-2 py-0.5 rounded text-[10px] font-mono ${
                      active
                        ? 'bg-[#389cff]/20 text-[#389cff]'
                        : 'bg-white/5 text-[#9ba5b4]'
                    }`}
                  >
                    {t.badge}
                  </span>
                </div>
                <p className="text-[11px] text-[#9ba5b4] line-clamp-2 leading-relaxed">
                  {t.subtitle}
                </p>
              </button>
            );
          })}
        </div>

        <div className="mt-2">
          {activeSimTab === 'flare-dag' && (
            <Suspense fallback={<LabFallback label="Loading Flare DAG…" />}>
              <FlareCodeGraphIDE />
            </Suspense>
          )}
          {activeSimTab === 'orbital-os' && (
            <Suspense fallback={<LabFallback label="Loading Orbital Agent OS…" />}>
              <OrbitalAgentOS />
            </Suspense>
          )}
          {activeSimTab === 'ares-runbook' && (
            <Suspense fallback={<LabFallback label="Loading ARES runbook…" />}>
              <AresRunbookLab />
            </Suspense>
          )}
          {activeSimTab === 'galaxy-3d' && (
            <Suspense fallback={<LabFallback label="Loading 3D force graph…" />}>
              <ForceGraph3D />
            </Suspense>
          )}
          {activeSimTab === 'xyflow-swarm' && (
            <Suspense fallback={<LabFallback label="Loading swarm DAG…" />}>
              <SwarmDAG />
            </Suspense>
          )}
        </div>
      </div>
    </section>
  );
};

export const SwarmLegacyPage: React.FC = () => (
  <div>
    <Hero />
    <ProofSection />
    <ArchitectureSection />
  </div>
);

/**
 * PUBLIC RESEARCH HUB PAGE (Byte-for-Byte 08-research-hub-v20.png)
 * + Direct Document Upload & Idea Intake that launches the Authenticated Research Workspace
 */
export const ResearchHubPublicPage: React.FC = () => {
  const {
    setActivePage,
    setCloudTab,
    scaffoldWorkspaceFromIdea,
    projectIdea,
  } = useAtomaStore();

  const [ideaText, setIdeaText] = useState(projectIdea);
  const [proofPrompt, setProofPrompt] = useState('');
  const [proofLines, setProofLines] = useState<string[]>([
    'Line 1: λ < 1 spectral radius condition ensures contraction stability across multi-agent steps.',
    'Line 2: For any ε > 0, we bound residual context drift via Eq. 7.',
  ]);

  const handleExplainSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!proofPrompt.trim()) return;
    setProofLines((prev) => [
      ...prev,
      `Q: ${proofPrompt} → Verified via Theorem 3.1: contraction factor λ = 0.92 guarantees convergence in O(log(1/ε)) iterations.`,
    ]);
    setProofPrompt('');
  };

  const handleLaunchResearchWorkspace = (e: React.FormEvent) => {
    e.preventDefault();
    scaffoldWorkspaceFromIdea(ideaText);
    setCloudTab('research');
    setActivePage('cloud-app');
    window.scrollTo({ top: 0, behavior: 'smooth' });
  };

  return (
    <div className="max-w-[1280px] mx-auto px-6 py-16 space-y-20">
      {/* Header Title: RESEARCH HUB — Fast answers. Clear proofs. */}
      <div className="flex flex-wrap items-baseline justify-between gap-6 border-b border-white/10 pb-8">
        <div className="flex flex-wrap items-baseline gap-6">
          <h1 className="font-display text-5xl sm:text-6xl font-bold tracking-tight text-white">
            RESEARCH HUB
          </h1>
          <span className="text-xl sm:text-2xl text-[#9ba5b4] font-medium">
            Fast answers. Clear proofs.
          </span>
        </div>

        <button
          onClick={() => {
            setCloudTab('research');
            setActivePage('cloud-app');
          }}
          className="px-5 py-2.5 rounded-xl bg-[#389cff] hover:bg-[#2589f5] text-white text-xs font-semibold flex items-center gap-2 shadow-[0_0_25px_rgba(56,156,255,0.45)] cursor-pointer"
        >
          <span>Open Full Research Workspace &amp; Scaffolder</span>
          <ArrowRight className="w-4 h-4" />
        </button>
      </div>

      {/* Interactive 3-Column Research Hub Showcase (exact match to 08-research-hub-v20.png) */}
      <div className="grid grid-cols-1 lg:grid-cols-[260px_1fr_300px] gap-6">
        {/* Col 1: MANUSCRIPT READER */}
        <div className="titanium-card rounded-2xl p-6 flex flex-col justify-between space-y-5">
          <div className="space-y-4">
            <div className="text-xs font-mono uppercase tracking-wider text-white font-semibold">
              MANUSCRIPT READER
            </div>
            <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded bg-white/8 border border-white/10 font-mono text-[10px] text-[#cbd5e1]">
              📄 PDF
            </span>

            <div>
              <h3 className="text-lg font-display font-bold text-white leading-snug">
                On Verified Reasoning in Linear Systems
              </h3>
              <div className="text-xs text-[#9ba5b4] mt-1">A. Chen · Oct 2024</div>
            </div>

            <div className="p-4 rounded-xl bg-[#0b1528] border border-[#389cff]/40 space-y-1.5">
              <div className="text-xs font-bold text-white">Proof 3.1</div>
              <p className="text-xs text-[#dbe4f0] font-mono leading-relaxed">
                Convergence holds when λ &lt; 1. For all ε &gt; 0, ∃N s.t. ∀n &gt; N, error &lt; ε.
              </p>
            </div>

            <div className="text-xs text-[#9ba5b4]">Linear bound · see Eq. 7</div>
          </div>

          <div className="pt-4 border-t border-white/8 text-xs font-mono text-[#cbd5e1]">
            Page 4 of 12 · 86% match
          </div>
        </div>

        {/* Col 2: VISUALS SYSTEM · ACTIVE */}
        <div className="titanium-card rounded-2xl p-6 flex flex-col justify-between">
          <div className="flex items-center gap-2 mb-4">
            <span className="text-xs font-mono uppercase tracking-wider text-[#9ba5b4]">
              VISUALS SYSTEM ·
            </span>
            <span className="px-2 py-0.5 rounded bg-[#389cff]/25 border border-[#389cff]/50 text-[#389cff] font-mono text-[10px] font-bold">
              ACTIVE
            </span>
          </div>

          <div className="relative flex-1 min-h-[340px] rounded-xl bg-[#05070c] border border-white/8 overflow-hidden flex items-center justify-center p-4">
            <svg viewBox="0 0 500 320" className="w-full h-full">
              {/* Subtle CAD background lines */}
              <line x1="60" y1="20" x2="60" y2="300" stroke="rgba(255,255,255,0.05)" />
              <line x1="250" y1="20" x2="250" y2="300" stroke="rgba(255,255,255,0.05)" />
              <line x1="40" y1="160" x2="460" y2="160" stroke="rgba(255,255,255,0.05)" />

              {/* Orbital Ellipse & Edges */}
              <ellipse
                cx="260"
                cy="145"
                rx="145"
                ry="75"
                transform="rotate(14 260 145)"
                fill="none"
                stroke="rgba(201,207,215,0.35)"
                strokeWidth="1.2"
              />
              <line
                x1="130"
                y1="145"
                x2="210"
                y2="98"
                stroke="#389cff"
                strokeWidth="1.5"
              />
              <line
                x1="210"
                y1="98"
                x2="345"
                y2="82"
                stroke="rgba(255,255,255,0.4)"
                strokeDasharray="3 3"
              />
              <line
                x1="210"
                y1="98"
                x2="290"
                y2="195"
                stroke="#389cff"
                strokeWidth="1.5"
              />
              <line
                x1="345"
                y1="82"
                x2="355"
                y2="178"
                stroke="rgba(255,255,255,0.4)"
              />

              {/* Nodes */}
              <circle cx="130" cy="145" r="6" fill="#0b1320" stroke="#ffffff" strokeWidth="2" />
              <text x="80" y="150" fill="#ffffff" fontSize="11" fontWeight="bold">
                Attention
              </text>

              <circle cx="210" cy="98" r="6" fill="#389cff" />
              <text x="220" y="102" fill="#ffffff" fontSize="11" fontWeight="bold">
                Transformer
              </text>

              <circle cx="345" cy="82" r="7" fill="#0b1320" stroke="#ffffff" strokeWidth="2" />
              <text x="358" y="86" fill="#ffffff" fontSize="11" fontWeight="bold">
                Sparse Layer
              </text>

              <circle cx="248" cy="145" r="8" fill="#0b1320" stroke="#ffffff" strokeWidth="2" />
              <circle cx="248" cy="145" r="3" fill="#ffffff" />
              <text x="275" y="142" fill="#ffffff" fontSize="11" fontWeight="bold">
                Gradient Flow
              </text>

              <circle cx="290" cy="195" r="7" fill="#389cff" />
              <text x="302" y="199" fill="#389cff" fontSize="11" fontWeight="bold">
                Proof: Convergence
              </text>

              {/* Lower-right convergence plot */}
              <g transform="translate(290, 220)">
                <line x1="0" y1="70" x2="150" y2="70" stroke="rgba(255,255,255,0.4)" />
                <line x1="20" y1="10" x2="20" y2="80" stroke="rgba(255,255,255,0.4)" />
                <path
                  d="M 20 62 Q 55 25, 85 45 T 140 20"
                  fill="none"
                  stroke="#ffffff"
                  strokeWidth="1.6"
                />
                <path
                  d="M 20 50 Q 65 48, 140 46"
                  fill="none"
                  stroke="#389cff"
                  strokeWidth="1.2"
                />
              </g>
            </svg>
          </div>

          <div className="pt-3 text-center font-mono text-xs text-[#cbd5e1]">
            Map · 12 nodes · 18 edges · last updated 2 min ago
          </div>
        </div>

        {/* Col 3: AI CHAT */}
        <div className="titanium-card rounded-2xl p-6 flex flex-col justify-between space-y-4">
          <div className="space-y-3">
            <div className="text-xs font-mono uppercase tracking-wider text-white font-semibold">
              AI CHAT
            </div>
            <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-[#389cff]/20 border border-[#389cff]/40 text-[#389cff] font-mono text-[10px] font-bold">
              ✦ AI · ON
            </span>

            <div className="p-3.5 rounded-xl bg-[#101c32] border border-[#389cff]/40 space-y-1">
              <div className="text-xs font-bold text-[#389cff]">AI</div>
              <div className="text-xs font-semibold text-white">Explain proof line-by-line</div>
            </div>

            {proofLines.map((line, idx) => (
              <div
                key={idx}
                className="p-3.5 rounded-xl bg-[#0c121e] border border-white/10 text-xs text-white leading-relaxed"
              >
                {line}
              </div>
            ))}
          </div>

          <form onSubmit={handleExplainSubmit} className="flex items-center gap-2 pt-2">
            <input
              type="text"
              value={proofPrompt}
              onChange={(e) => setProofPrompt(e.target.value)}
              placeholder="Ask about the paper..."
              className="flex-1 px-3 py-2 rounded-lg bg-[#06090f] border border-white/12 text-xs text-white focus:outline-none"
            />
            <button
              type="submit"
              className="px-3 py-2 rounded-lg bg-[#389cff] text-white font-bold text-xs cursor-pointer"
            >
              ›
            </button>
          </form>
        </div>
      </div>

      {/* Idea Intake -> Scaffold Workspace & Decompose to Swarm Card */}
      <form
        onSubmit={handleLaunchResearchWorkspace}
        className="titanium-card rounded-2xl p-8 border border-[#389cff]/40 space-y-4"
      >
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <div className="font-mono text-xs uppercase tracking-[0.2em] text-[#389cff]">
              FROM PAPER &amp; IDEA TO SWARM IMPLEMENTATION
            </div>
            <h2 className="font-display text-2xl sm:text-3xl font-bold text-white mt-1">
              Upload your docs, state the overall idea, and scaffold the Research Workspace.
            </h2>
          </div>
          <button
            type="submit"
            className="px-5 py-3 rounded-xl bg-[#389cff] hover:bg-[#2589f5] text-white font-semibold text-xs flex items-center gap-2 shadow-[0_0_25px_rgba(56,156,255,0.5)] cursor-pointer"
          >
            <span>Scaffold Workspace &amp; Open Research Chat</span>
            <ArrowRight className="w-4 h-4" />
          </button>
        </div>

        <input
          type="text"
          value={ideaText}
          onChange={(e) => setIdeaText(e.target.value)}
          className="w-full px-4 py-3 rounded-xl bg-[#05080d] border border-white/15 text-sm text-white focus:border-[#389cff] focus:outline-none"
          placeholder="Enter your overall project idea or hypothesis..."
        />
      </form>

      {/* 3 Feature Blocks: Ask anything about the paper | Understand the proof | From paper to implementation */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {[
          ['Ask anything about the paper', 'Ask. Get precise answers.'],
          ['Understand the proof', 'Step through logic. Line by line.'],
          ['From paper to implementation', 'Guide to code. Advance your agentic system.'],
        ].map(([title, sub]) => (
          <div key={title} className="titanium-card rounded-2xl p-7 space-y-3">
            <CheckCircle2 className="w-6 h-6 text-[#389cff]" />
            <h3 className="text-xl font-display font-bold text-white">{title}</h3>
            <p className="text-sm text-[#9ba5b4]">{sub}</p>
          </div>
        ))}
      </div>

      {/* Bottom 2 Cards: API sign up | MCP AI research vault */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="titanium-card rounded-2xl p-8 space-y-4">
          <h3 className="text-2xl font-display font-bold text-white">API sign up</h3>
          <p className="text-sm text-[#9ba5b4]">Pull vetted materials via API.</p>
          <button
            onClick={() => setActivePage('sign-up')}
            className="px-5 py-2.5 rounded-lg bg-[#389cff] hover:bg-[#2589f5] text-white font-semibold text-xs inline-flex items-center gap-2 cursor-pointer"
          >
            <span>Get API key →</span>
          </button>
          <div className="text-xs font-mono text-[#9ba5b4]">Key in seconds. No billing.</div>
        </div>

        <div className="titanium-card rounded-2xl p-8 space-y-4">
          <h3 className="text-2xl font-display font-bold text-white">MCP AI research vault</h3>
          <p className="text-sm text-[#9ba5b4]">
            Download plugin for your harness. Powered by Atoma N3.
          </p>
          <button
            onClick={() => setActivePage('downloads')}
            className="w-full py-2.5 px-4 rounded-lg bg-white/6 hover:bg-white/12 border border-white/15 text-[#389cff] font-semibold text-xs cursor-pointer"
          >
            Download MCP Vault →
          </button>
          <div className="text-xs font-mono text-[#9ba5b4] text-center">
            Plugin · Atoma N3 · v0.3.1
          </div>
        </div>
      </div>
    </div>
  );
};

export const PlatformHomePage: React.FC = () => {
  const {
    setActivePage,
    setCloudTab,
    runbookActiveStep,
    setRunbookActiveStep,
    setActiveSimTab,
  } = useAtomaStore();

  return (
    <div>
      <section className="relative pt-16 pb-20 border-b border-white/8 overflow-hidden">
        <div className="max-w-[1280px] mx-auto px-6 grid grid-cols-1 lg:grid-cols-[1.1fr_0.9fr] gap-12 items-center">
          <div className="space-y-6">
            <h1 className="font-display text-5xl sm:text-7xl font-bold tracking-[-0.04em] leading-[0.96] text-[#f5f7fa]">
              Principled
              <br />
              intelligence
              <br />
              with receipts.
            </h1>
            <p className="text-xl text-[#c9cfd7] font-medium max-w-xl leading-snug">
              An independent engineering lab for builders with bigger ambitions.
            </p>
            <p className="text-base text-[#9ba5b4] max-w-lg leading-relaxed">
              We build tools, runbooks, and research systems that keep the evidence connected to
              the work.
            </p>
            <div className="flex flex-wrap items-center gap-3.5 pt-2">
              <button
                onClick={() => {
                  setCloudTab('code');
                  setActivePage('cloud-app');
                }}
                className="px-6 py-3 rounded-lg bg-[#389cff] hover:bg-[#2589f5] text-white font-medium text-sm inline-flex items-center gap-2 transition shadow-[0_0_25px_-4px_rgba(56,156,255,0.65)] cursor-pointer"
              >
                <span>Launch Cloud Workspace (IDE &amp; Swarm)</span>
                <ArrowRight className="w-4 h-4" />
              </button>
              <button
                onClick={() => setActivePage('research-hub')}
                className="px-6 py-3 rounded-lg bg-transparent hover:bg-white/5 border border-white/15 text-[#f5f7fa] font-medium text-sm transition cursor-pointer"
              >
                Open Research Hub
              </button>
            </div>
          </div>

          <LazySculpture
            variant="hero"
            height="h-[460px]"
            label="ATOMA 3D EXTRUDED MONOLITH · N³ CORE"
          />
        </div>
      </section>

      <section className="py-12 border-b border-white/8">
        <div className="max-w-[1280px] mx-auto px-6 grid grid-cols-1 md:grid-cols-3 gap-8 divide-y md:divide-y-0 md:divide-x divide-white/8">
          <div className="flex items-start gap-4 pt-4 md:pt-0">
            <div className="w-11 h-11 rounded-xl bg-[#0c0f14] border border-white/12 flex items-center justify-center shrink-0 text-[#c9cfd7]">
              <Layers className="w-5 h-5" />
            </div>
            <div>
              <h3 className="text-base font-semibold text-white">Build with context</h3>
              <p className="text-xs text-[#9ba5b4] mt-1.5 leading-relaxed">
                Bring your goals, code, and research together so the system understands the bigger
                picture.
              </p>
            </div>
          </div>

          <div className="flex items-start gap-4 pt-4 md:pt-0 md:pl-8">
            <div className="w-11 h-11 rounded-xl bg-[#0c0f14] border border-white/12 flex items-center justify-center shrink-0 text-[#c9cfd7]">
              <GitBranch className="w-5 h-5" />
            </div>
            <div>
              <h3 className="text-base font-semibold text-white">Test the idea</h3>
              <p className="text-xs text-[#9ba5b4] mt-1.5 leading-relaxed">
                Turn uncertainty into structured experiments with repeatable workflows.
              </p>
            </div>
          </div>

          <div className="flex items-start gap-4 pt-4 md:pt-0 md:pl-8">
            <div className="w-11 h-11 rounded-xl bg-[#0c0f14] border border-white/12 flex items-center justify-center shrink-0 text-[#c9cfd7]">
              <FileText className="w-5 h-5" />
            </div>
            <div>
              <h3 className="text-base font-semibold text-white">Keep the evidence</h3>
              <p className="text-xs text-[#9ba5b4] mt-1.5 leading-relaxed">
                Every decision, result, and artifact is connected and traceable by design.
              </p>
            </div>
          </div>
        </div>
      </section>

      <section className="py-24 border-b border-white/8">
        <div className="max-w-[1280px] mx-auto px-6 grid grid-cols-1 lg:grid-cols-[0.9fr_1.1fr] gap-14 items-center">
          <div className="space-y-5">
            <div className="font-mono text-xs uppercase tracking-[0.2em] text-[#389cff]">
              ARES
            </div>
            <h2 className="font-display text-4xl sm:text-6xl font-bold tracking-[-0.035em] leading-[0.98] text-white">
              Small team.
              <br />
              Extraordinary reach.
            </h2>
            <p className="text-base text-[#9ba5b4] leading-relaxed max-w-md">
              ARES Spec Ops Research Lab brings project context, repeatable workflows, and human
              judgment into one workspace.
            </p>
            <div className="flex flex-wrap items-center gap-3 pt-2">
              <button
                onClick={() => {
                  setCloudTab('research');
                  setActivePage('cloud-app');
                }}
                className="px-5 py-2.5 rounded-lg bg-[#389cff] hover:bg-[#2589f5] text-white font-medium text-sm inline-flex items-center gap-2 transition cursor-pointer"
              >
                <span>Inside the research lab</span>
                <ArrowRight className="w-4 h-4" />
              </button>
              <button
                onClick={() => {
                  setActiveSimTab('ares-runbook');
                  document
                    .getElementById('interactive-simulations')
                    ?.scrollIntoView({ behavior: 'smooth' });
                }}
                className="px-5 py-2.5 rounded-lg bg-transparent hover:bg-white/5 border border-white/15 text-white font-medium text-sm transition cursor-pointer"
              >
                Explore the design
              </button>
            </div>
          </div>

          <BranchingWorkflowConceptSVG
            activeStep={runbookActiveStep}
            onSelectStep={(step) => setRunbookActiveStep(step)}
          />
        </div>
      </section>

      <InteractiveSimulationShowcase />
    </div>
  );
};

export const ToolsMcpPage: React.FC = () => {
  const { setActivePage, setCloudTab } = useAtomaStore();
  const [filter, setFilter] = useState<'All capabilities' | 'Tools' | 'MCP' | 'Skills' | 'Research'>(
    'All capabilities'
  );
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCap, setSelectedCap] = useState('N³ Traceability');

  const capabilities = [
    {
      category: 'Tools',
      title: 'Research-grade harness',
      desc: 'Mission execution, runbook parsing, and evidence receipts.',
      badge: 'Source available',
      dotColor: 'bg-emerald-400',
      env: 'Local Rust / TypeScript CLI',
      evidence: 'Harness specs & runbooks',
      adapters: 'STDIO & local socket',
      access: 'Source review',
    },
    {
      category: 'Research',
      title: 'N³ Traceability',
      desc: 'A SQLite foundation for decisions, execution, and acceptance.',
      badge: 'Local foundation',
      dotColor: 'bg-[#389cff]',
      env: 'Local SQLite',
      evidence: 'Schema and tests',
      adapters: 'Not connected',
      access: 'Source review',
    },
    {
      category: 'Skills',
      title: 'Capability Tree',
      desc: 'Skills organized by discipline and verification needs.',
      badge: 'Repository catalog',
      dotColor: 'bg-[#389cff]',
      env: 'Declarative SKILL.md manifest',
      evidence: 'Verification gate rules',
      adapters: 'Local filesystem',
      access: 'Repository catalog',
    },
    {
      category: 'MCP',
      title: 'MCP connections',
      desc: 'Interfaces for connecting tools with explicit permissions.',
      badge: 'Integration planned',
      dotColor: 'bg-amber-400',
      env: 'Model Context Protocol JSON-RPC',
      evidence: 'Permission boundary schema',
      adapters: 'Explicit allowlist',
      access: 'Contract preview',
    },
    {
      category: 'Tools',
      title: 'Runbook contracts',
      desc: 'Define the objective, constraints, and acceptance gates.',
      badge: 'Design & source',
      dotColor: 'bg-[#389cff]',
      env: 'YAML / TypeScript AST',
      evidence: 'Pre-registration schema',
      adapters: 'ARES Spec Ops Runner',
      access: 'Source review',
    },
    {
      category: 'Research',
      title: 'Memory experiments',
      desc: 'Explore context retention and knowledge reuse.',
      badge: 'Experimental',
      dotColor: 'bg-purple-400',
      env: 'Local N³ knowledge packs',
      evidence: 'Retrieval benchmark runs',
      adapters: 'In development',
      access: 'Experimental branch',
    },
  ];

  const filtered = capabilities.filter((c) => {
    const matchesCat = filter === 'All capabilities' || c.category === filter;
    const matchesSearch =
      c.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      c.desc.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesCat && matchesSearch;
  });

  const activeCap = capabilities.find((c) => c.title === selectedCap) || capabilities[1];

  return (
    <div className="max-w-[1280px] mx-auto px-6 py-16 space-y-20">
      <div className="grid grid-cols-1 lg:grid-cols-[1.15fr_0.85fr] gap-12 items-center">
        <div className="space-y-5">
          <div className="font-mono text-xs uppercase tracking-[0.2em] text-[#9ba5b4]">
            TOOLS &amp; MCP CATALOG
          </div>
          <h1 className="font-display text-5xl sm:text-6xl font-bold tracking-[-0.035em] leading-[0.98] text-white">
            Capability,
            <br />
            with a clear contract.
          </h1>
          <p className="text-base text-[#9ba5b4] max-w-lg leading-relaxed">
            Explore the tools, skills, and connections behind ARES. Understand what they do, where
            they run, and what is ready.
          </p>
        </div>

        <div className="titanium-card rounded-2xl p-8 flex items-center justify-between relative overflow-hidden">
          <div className="w-28 h-32 rounded-2xl bg-gradient-to-br from-[#2a3241] to-[#0d1118] border border-white/20 flex items-center justify-center shadow-2xl">
            <div className="w-12 h-16 rounded-lg bg-[#389cff]/20 border-2 border-[#389cff] flex items-center justify-center shadow-[0_0_25px_rgba(56,156,255,0.8)]">
              <AtomaLogoMark className="w-7 h-7" />
            </div>
          </div>
          <div className="flex-1 space-y-3 pl-4">
            {[0, 1, 2, 3].map((idx) => (
              <div key={idx} className="flex items-center gap-2">
                <div className="h-2.5 flex-1 rounded-full bg-gradient-to-r from-[#389cff] via-[#c9cfd7]/40 to-white/10" />
                <span className="font-mono text-[10px] text-[#9ba5b4]">MCP_0{idx + 1}</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      <div className="titanium-card rounded-2xl p-5 space-y-4">
        <div className="flex items-center gap-3 px-4 py-3 rounded-xl bg-[#06080c] border border-white/10">
          <Search className="w-4 h-4 text-[#9ba5b4]" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Find a tool, skill, or connection"
            className="bg-transparent text-sm text-white focus:outline-none w-full"
          />
          <kbd className="px-2 py-0.5 rounded bg-white/8 text-[11px] font-mono text-[#9ba5b4]">/</kbd>
        </div>

        <div className="flex flex-wrap items-center justify-between gap-4">
          <div className="flex flex-wrap items-center gap-2">
            {(['All capabilities', 'Tools', 'MCP', 'Skills', 'Research'] as const).map((chip) => (
              <button
                key={chip}
                onClick={() => setFilter(chip)}
                className={`px-4 py-1.5 rounded-full text-xs font-medium transition cursor-pointer ${
                  filter === chip
                    ? 'bg-[#389cff] text-white'
                    : 'bg-white/5 hover:bg-white/10 text-[#9ba5b4] border border-white/10'
                }`}
              >
                {chip}
              </button>
            ))}
          </div>
          <span className="text-xs text-[#9ba5b4]">
            A curated view of the engineering repository.
          </span>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {filtered.map((item) => {
          const isSelected = selectedCap === item.title;
          return (
            <button
              type="button"
              key={item.title}
              onClick={() => setSelectedCap(item.title)}
              aria-pressed={isSelected}
              className={`titanium-card rounded-2xl p-6 flex flex-col justify-between text-left cursor-pointer transition ${
                isSelected ? 'ring-2 ring-[#389cff]' : ''
              }`}
            >
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <div className="w-10 h-10 rounded-xl bg-white/5 border border-white/10 flex items-center justify-center text-[#c9cfd7]">
                    <Database className="w-5 h-5" />
                  </div>
                  <span className="font-mono text-[10px] uppercase tracking-widest text-[#9ba5b4]">
                    {item.category}
                  </span>
                </div>
                <h3 className="text-xl font-display font-semibold text-white">{item.title}</h3>
                <p className="text-sm text-[#9ba5b4] leading-relaxed">{item.desc}</p>
              </div>

              <div className="pt-6 mt-6 border-t border-white/8 space-y-4">
                <span className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-white/5 border border-white/10 text-xs text-[#e2e8f0]">
                  <span className={`w-2 h-2 rounded-full ${item.dotColor}`} />
                  {item.badge}
                </span>
                <div className="flex items-center justify-between text-xs font-medium text-[#389cff]">
                  <span>Inspect the details</span>
                  <ArrowRight className="w-3.5 h-3.5" />
                </div>
              </div>
            </button>
          );
        })}
      </div>

      <div className="titanium-card rounded-2xl p-8 sm:p-10 grid grid-cols-1 lg:grid-cols-2 gap-10 items-center">
        <div className="space-y-5">
          <div className="font-mono text-xs uppercase tracking-[0.2em] text-[#9ba5b4]">
            CAPABILITY DETAIL
          </div>
          <h2 className="font-display text-3xl sm:text-4xl font-bold text-white">
            Know the boundary before you connect.
          </h2>
          <div className="pt-2 space-y-2">
            <div className="flex items-center gap-3">
              <h3 className="text-lg font-semibold text-white">{activeCap.title}</h3>
              <span className="px-2.5 py-0.5 rounded-full bg-[#389cff]/15 border border-[#389cff]/30 text-[#389cff] text-xs">
                {activeCap.badge}
              </span>
            </div>
            <div className="divide-y divide-white/8 text-xs pt-2">
              {[
                ['Environment:', activeCap.env],
                ['Evidence:', activeCap.evidence],
                ['Live adapters:', activeCap.adapters],
                ['Access:', activeCap.access],
              ].map(([k, v]) => (
                <div key={k} className="py-2.5 flex justify-between">
                  <span className="text-[#9ba5b4]">{k}</span>
                  <span className="font-mono text-white font-medium">{v}</span>
                </div>
              ))}
            </div>
            <div className="flex flex-wrap gap-2 pt-4">
              <button
                type="button"
                onClick={() => {
                  setCloudTab('tools');
                  setActivePage('cloud-app');
                  window.scrollTo({ top: 0, behavior: 'smooth' });
                }}
                className="px-4 py-2 rounded-lg bg-[#389cff] hover:bg-[#2589f5] text-white text-xs font-semibold cursor-pointer"
              >
                Open in Cloud Tools
              </button>
              <button
                type="button"
                onClick={() => {
                  setActivePage('sign-up');
                  window.scrollTo({ top: 0, behavior: 'smooth' });
                }}
                className="px-4 py-2 rounded-lg bg-white/8 hover:bg-white/12 border border-white/15 text-xs text-white cursor-pointer"
              >
                Get access
              </button>
            </div>
          </div>
        </div>

        <div className="rounded-xl bg-[#06090f] border border-white/10 p-6 space-y-4">
          <div>
            <div className="text-sm font-semibold text-white">Schema overview</div>
            <div className="text-xs text-[#9ba5b4]">Core tables and their relationships.</div>
          </div>
          <div className="grid grid-cols-2 gap-4">
            {[
              { name: 'Decision', fields: ['intent', 'rationale', 'options', 'selected'] },
              { name: 'Execution', fields: ['run_id', 'tool', 'input', 'result'] },
              { name: 'Acceptance', fields: ['criteria', 'evaluation', 'status', 'notes'] },
              { name: 'Receipt', fields: ['artifacts', 'evidence', 'hash', 'references'] },
            ].map((tbl) => (
              <div
                key={tbl.name}
                className="p-3.5 rounded-lg bg-[#0c111b] border border-[#389cff]/30 space-y-1.5"
              >
                <div className="text-xs font-semibold text-white flex items-center justify-between">
                  <span>{tbl.name}</span>
                  <span className="w-1.5 h-1.5 rounded-full bg-[#389cff]" />
                </div>
                <div className="font-mono text-[11px] text-[#9ba5b4] space-y-0.5">
                  {tbl.fields.map((f) => (
                    <div key={f}>{f}</div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

function LazySculpture(props: React.ComponentProps<typeof AtomaSculptureCanvas>) {
  return (
    <Suspense fallback={<LabFallback label="Loading 3D visual…" />}>
      <AtomaSculptureCanvas {...props} />
    </Suspense>
  );
}

function LazyRunbookLab() {
  return (
    <Suspense fallback={<LabFallback label="Loading ARES runbook…" />}>
      <AresRunbookLab />
    </Suspense>
  );
}

export const MetaHackathonPage: React.FC = () => (
  <div className="max-w-[1280px] mx-auto px-6 py-16 space-y-20">
    <div className="grid grid-cols-1 lg:grid-cols-[1.1fr_0.9fr] gap-12 items-center">
      <div className="space-y-5">
        <div className="font-mono text-xs uppercase tracking-[0.2em] text-[#389cff]">
          SOLO DEVELOPER TRACK
        </div>
        <h1 className="font-display text-5xl sm:text-6xl font-bold tracking-[-0.035em] leading-[0.98] text-white">
          One builder.
          <br />A bigger operating system for ideas.
        </h1>
        <div className="text-xl font-display text-[#c9cfd7]">ARES Spec Ops Research Lab</div>
        <p className="text-base text-[#9ba5b4] max-w-lg leading-relaxed">
          An experimental workspace for testing agent workflows, developing products, and keeping
          evidence attached to every decision.
        </p>
      </div>

      <LazySculpture
        variant="cube"
        height="h-[420px]"
        label="ARCHITECTURAL CUBE · SOLO DEVELOPER TRACK"
      />
    </div>

    <div className="space-y-4">
      <div className="font-mono text-xs uppercase tracking-[0.2em] text-[#389cff]">
        ILLUSTRATIVE WORKFLOW
      </div>
      <h2 className="font-display text-3xl sm:text-4xl font-bold text-white">
        Evaluate a new runbook loop
      </h2>
      <LazyRunbookLab />
    </div>
  </div>
);

export const ResearchLabPage: React.FC = () => (
  <div className="max-w-[1280px] mx-auto px-6 py-16 space-y-20">
    <div className="grid grid-cols-1 lg:grid-cols-[1.1fr_0.9fr] gap-12 items-center">
      <div className="space-y-5">
        <div className="font-mono text-xs uppercase tracking-[0.2em] text-[#9ba5b4]">
          RESEARCH LAB
        </div>
        <h1 className="font-display text-5xl sm:text-6xl font-bold tracking-[-0.035em] leading-[0.98] text-white">
          ARES Spec Ops
          <br />
          Research Lab.
        </h1>
        <div className="text-xl text-[#c9cfd7] font-medium">
          A place to test the loop.
          <br />
          Build the product. Keep the receipts.
        </div>
        <p className="text-base text-[#9ba5b4] max-w-lg leading-relaxed">
          Explore runbook experiments, product development, and the systems that connect an idea
          to evidence.
        </p>
      </div>

      <LazySculpture
        variant="rings"
        height="h-[420px]"
        label="GYROSCOPIC TITANIUM LOOPS · N³ NUCLEUS"
      />
    </div>

    <LazyRunbookLab />
  </div>
);

export const ContactPage: React.FC = () => {
  const { setActivePage, setCloudTab } = useAtomaStore();
  const [topic, setTopic] = useState('Research');
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [message, setMessage] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [ticket, setTicket] = useState<string | null>(null);

  const submit = () => {
    if (!name.trim() || !/.+@.+\..+/.test(email) || message.trim().length < 10) {
      setError('Add your name, a valid email, and a message of 10+ characters.');
      return;
    }
    setError(null);
    setTicket(`REQ-${Date.now().toString(36).toUpperCase().slice(-6)}`);
  };

  return (
    <div className="max-w-[1280px] mx-auto px-6 py-16 space-y-20">
      <div className="grid grid-cols-1 lg:grid-cols-[1.1fr_0.9fr] gap-12 items-center">
        <div className="space-y-4">
          <div className="font-mono text-xs uppercase tracking-[0.2em] text-[#9ba5b4]">
            CONTACT · PRODUCT CONCEPT
          </div>
          <h1 className="font-display text-5xl sm:text-6xl font-bold text-white leading-[0.98]">
            Good work starts
            <br />
            with a conversation.
          </h1>
          <p className="text-base text-[#9ba5b4] max-w-lg">
            Building something ambitious? Talk to us about research, tooling, collaboration, or
            bringing ARES into your workflow.
          </p>
          <a
            href={`mailto:hello@atoma.engineering?subject=${encodeURIComponent(`Contact · ${topic} · ${name || 'New inquiry'}`)}&body=${encodeURIComponent(`${message}\n\n— ${name} <${email}>`)}`}
            className="text-xs font-mono text-[#389cff] hover:text-white transition"
          >
            Open in your mail client instead →
          </a>
        </div>
        <LazySculpture variant="hero" height="h-[360px]" />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[1.1fr_0.9fr] gap-12">
        <div className="titanium-card rounded-2xl p-8 space-y-5">
          <h2 className="text-2xl font-display font-bold text-white">Get in touch.</h2>
          {ticket ? (
            <div role="status" className="rounded-xl bg-emerald-500/10 border border-emerald-500/30 p-6 space-y-3">
              <div className="font-mono text-xs text-emerald-300">RECEIPT LOGGED · {ticket}</div>
              <p className="text-sm text-white">Thanks, {name}. Your {topic.toLowerCase()} note is queued with ticket {ticket}.</p>
              <div className="flex flex-wrap gap-2 pt-1">
                <button
                  onClick={() => {
                    setCloudTab('research');
                    setActivePage('cloud-app');
                    window.scrollTo({ top: 0, behavior: 'smooth' });
                  }}
                  className="px-4 py-2 rounded-lg bg-[#389cff] hover:bg-[#2589f5] text-white text-xs font-semibold cursor-pointer"
                >
                  Track in Research Hub
                </button>
                <button
                  onClick={() => {
                    setTicket(null);
                    setMessage('');
                  }}
                  className="px-4 py-2 rounded-lg bg-white/8 border border-white/15 text-xs text-white hover:bg-white/12 cursor-pointer"
                >
                  Send another
                </button>
              </div>
            </div>
          ) : (
          <div className="space-y-3 text-xs">
            <div>
              <label htmlFor="contact-name" className="block text-white mb-1.5">Name</label>
              <input
                id="contact-name"
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Your name"
                className="w-full px-3.5 py-2.5 rounded-lg bg-[#06080c] border border-white/12 text-white"
              />
            </div>
            <div>
              <label htmlFor="contact-email" className="block text-white mb-1.5">Email</label>
              <input
                id="contact-email"
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="you@domain.com"
                className="w-full px-3.5 py-2.5 rounded-lg bg-[#06080c] border border-white/12 text-white"
              />
            </div>
            <div>
              <label htmlFor="contact-message" className="block text-white mb-1.5">Message</label>
              <textarea
                id="contact-message"
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                placeholder="What are you building? (10+ characters)"
                rows={4}
                className="w-full px-3.5 py-2.5 rounded-lg bg-[#06080c] border border-white/12 text-white resize-y"
              />
            </div>
            <div>
              <label className="block text-white mb-1.5">What brings you here?</label>
              <div className="flex flex-wrap gap-2" role="group" aria-label="Contact topic">
                {['Research', 'Partnerships', 'Technical questions', 'Contribute'].map((t) => (
                  <button
                    key={t}
                    type="button"
                    onClick={() => setTopic(t)}
                    aria-pressed={topic === t}
                    className={`px-3 py-1.5 rounded-lg border text-xs cursor-pointer ${
                      topic === t
                        ? 'bg-[#389cff]/20 border-[#389cff] text-white'
                        : 'bg-[#06080c] border-white/10 text-[#9ba5b4]'
                    }`}
                  >
                    {t}
                  </button>
                ))}
              </div>
            </div>
            {error && (
              <p role="alert" className="text-xs text-red-300">{error}</p>
            )}
            <button
              type="button"
              onClick={submit}
              className="px-6 py-3 rounded-lg bg-[#389cff] hover:bg-[#2589f5] text-white font-semibold text-xs inline-flex items-center gap-2 cursor-pointer"
            >
              <span>Send message</span>
              <ArrowRight className="w-4 h-4" />
            </button>
          </div>
          )}
        </div>

        <div className="titanium-card rounded-2xl p-8 space-y-4 self-start">
          <h3 className="text-lg font-display font-bold text-white">Prefer the workspace?</h3>
          <p className="text-sm text-[#9ba5b4] leading-relaxed">
            Sign in once and route {topic.toLowerCase()} questions straight into the
            Research Hub scaffolder with your model lane attached.
          </p>
          <button
            onClick={() => {
              try {
                sessionStorage.setItem(
                  'atoma.postLogin',
                  JSON.stringify({ tab: 'research' as const }),
                );
              } catch { /* private mode — login still lands on Code tab */ }
              setActivePage('sign-up');
              window.scrollTo({ top: 0, behavior: 'smooth' });
            }}
            className="px-5 py-2.5 rounded-lg bg-white/8 hover:bg-white/12 border border-white/15 text-sm text-white font-medium cursor-pointer"
          >
            Continue to sign in
          </button>
        </div>
      </div>
    </div>
  );
};

export const SignUpPage: React.FC = () => {
  const {
    setActivePage,
    setCloudTab,
    setIsAuthenticated,
    setUserName,
    setUserHandle,
    selectedCloudModel,
    setSelectedCloudModel,
  } = useAtomaStore();
  const [modelId, setModelId] = useState(selectedCloudModel);
  const [name, setName] = useState('Stewart Barteau');
  const [email, setEmail] = useState('stewart@atoma.engineering');
  const [error, setError] = useState<string | null>(null);
  const scope = useRef<HTMLDivElement>(null);
  const reduced = useReducedMotion();

  useGSAP(
    () => {
      if (reduced || !scope.current) return;
      gsap.from(scope.current.querySelectorAll('[data-signup-rise]'), {
        opacity: 0,
        y: 24,
        duration: 0.6,
        ease: 'power2.out',
        stagger: 0.08,
      });
    },
    { scope, dependencies: [reduced] },
  );

  const handleLogin = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !/.+@.+\..+/.test(email)) {
      setError('Add your name and a valid email to enter the workspace.');
      return;
    }
    setError(null);
    setIsAuthenticated(true);
    setUserName(name.trim());
    setUserHandle(
      name
        .trim()
        .split(/\s+/)
        .map((w) => w[0])
        .join('')
        .slice(0, 2)
        .toUpperCase() || 'AT',
    );
    setSelectedCloudModel(modelId);
    let tab: CloudWorkspaceTab = 'code';
    try {
      const raw = sessionStorage.getItem('atoma.postLogin');
      if (raw) {
        const parsed = JSON.parse(raw) as { tab?: CloudWorkspaceTab };
        if (parsed.tab) tab = parsed.tab;
        sessionStorage.removeItem('atoma.postLogin');
      }
    } catch { /* private mode — fall through to Code tab */ }
    setCloudTab(tab);
    setActivePage('cloud-app');
    window.scrollTo({ top: 0, behavior: 'smooth' });
  };

  return (
    <div className="max-w-[1280px] mx-auto px-6 py-16 space-y-20">
      <div className="grid grid-cols-1 lg:grid-cols-[1.15fr_0.85fr] gap-12 items-start">
        <div className="space-y-6">
          <div className="font-mono text-xs uppercase tracking-[0.2em] text-[#389cff]">
            AUTHENTICATED CLOUD WORKSPACE · EARLY ACCESS
          </div>
          <h1 className="font-display text-5xl sm:text-6xl font-bold text-white leading-[0.96]">
            Your hardware.
            <br />
            Your agents.
            <br />
            Your research hub.
          </h1>
          <p className="text-base text-[#9ba5b4] max-w-lg">
            Sign in below to enter the live ATOMA / ARES Cloud Workspace serving Atoma-N3-70B,
            Nemotron-3.5-30B-MLX, BEAM-1.0-501B, and the full IDE, Swarm Holonomy DAG, Mind Context
            Graph, and Research Hub Scaffolder.
          </p>
          <LazySculpture
            variant="network"
            height="h-[360px]"
            label="CRYSTALLINE N³ KNOWLEDGE NETWORK"
          />
        </div>

        <form onSubmit={handleLogin} className="titanium-card rounded-2xl p-8 space-y-4">
          <div className="font-mono text-[10px] uppercase tracking-[0.2em] text-[#389cff]">
            CLOUD WORKSPACE ACCESS
          </div>
          <h2 className="text-2xl font-display font-bold text-white">Join / Sign in to Atoma</h2>
          <div className="space-y-3 text-xs">
            <div>
              <label className="block text-white mb-1">Full name</label>
              <input
                type="text"
                defaultValue="Stewart Barteau"
                className="w-full px-3.5 py-2.5 rounded-lg bg-[#06080c] border border-white/12 text-white"
              />
            </div>
            <div>
              <label className="block text-white mb-1">Email address</label>
              <input
                type="email"
                defaultValue="stewart@atoma.engineering"
                className="w-full px-3.5 py-2.5 rounded-lg bg-[#06080c] border border-white/12 text-white"
              />
            </div>
            <div>
              <label htmlFor="signup-model" className="block text-white mb-1">Default Cloud / Local Model Lane</label>
              <select
                id="signup-model"
                value={modelId}
                onChange={(e) => setModelId(e.target.value)}
                className="w-full px-3.5 py-2.5 rounded-lg bg-[#06080c] border border-white/12 text-white"
              >
                {CLOUD_MODELS.map((m) => (
                  <option key={m.id} value={m.id}>
                    {m.name} — {m.latency}
                  </option>
                ))}
              </select>
            </div>
            <button
              type="submit"
              className="w-full py-3 rounded-lg bg-[#389cff] hover:bg-[#2589f5] text-white font-semibold text-xs flex items-center justify-center gap-2 shadow-[0_0_25px_rgba(56,156,255,0.5)] cursor-pointer"
            >
              <span>Enter Authenticated Cloud Workspace →</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

export const DownloadsPage: React.FC = () => {
  const { setActivePage, setCloudTab } = useAtomaStore();
  return (
  <div className="max-w-[1280px] mx-auto px-6 py-16 space-y-20">
    <div className="grid grid-cols-1 lg:grid-cols-[1.1fr_0.9fr] gap-12 items-center">
      <div className="space-y-4">
        <div className="font-mono text-xs uppercase tracking-[0.2em] text-[#389cff]">
          PRODUCT CONCEPT · EARLY ACCESS
        </div>
        <h1 className="font-display text-5xl sm:text-6xl font-bold text-white leading-[0.98]">
          A research lab.
          <br />
          On your hardware.
        </h1>
        <div className="text-xl text-[#c9cfd7]">ARES Spec Ops Research Lab</div>
      </div>

      <LazySculpture
        variant="hero"
        height="h-[380px]"
        label="LOCAL HARDWARE PLATFORM · MACOS · LINUX · WINDOWS"
      />

      <div className="titanium-card rounded-2xl p-8 space-y-6">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <div className="font-mono text-xs uppercase tracking-[0.2em] text-[#9ba5b4]">
              ARTIFACTS · VERIFIABLE
            </div>
            <h2 className="font-display text-3xl font-bold text-white mt-2">
              Take the receipts with you.
            </h2>
          </div>
          <button
            type="button"
            onClick={() => {
              setCloudTab('code');
              setActivePage('cloud-app');
              window.scrollTo({ top: 0, behavior: 'smooth' });
            }}
            className="px-4 py-2 rounded-lg bg-[#389cff] hover:bg-[#2589f5] text-white text-xs font-semibold cursor-pointer"
          >
            Open Cloud workspace
          </button>
        </div>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {[
            { title: 'version.json', desc: 'Build stamp — version, commit, time.', href: '/version.json' },
            { title: 'sitemap.xml', desc: 'Crawlable route map.', href: '/sitemap.xml' },
            { title: 'robots.txt', desc: 'Crawler contract.', href: '/robots.txt' },
            { title: 'security.txt', desc: 'Disclosure contact.', href: '/.well-known/security.txt' },
          ].map((a) => (
            <a
              key={a.title}
              href={a.href}
              download
              className="rounded-xl bg-[#06080c] border border-white/10 p-5 space-y-1.5 hover:border-[#389cff]/50 transition group"
            >
              <div className="font-mono text-sm text-white group-hover:text-[#389cff] transition">{a.title}</div>
              <div className="text-xs text-[#9ba5b4]">{a.desc}</div>
            </a>
          ))}
          <button
            type="button"
            onClick={() => {
              setActivePage('research-hub');
              window.scrollTo({ top: 0, behavior: 'smooth' });
            }}
            className="text-left rounded-xl bg-[#06080c] border border-white/10 p-5 space-y-1.5 hover:border-[#389cff]/50 transition cursor-pointer"
          >
            <div className="font-mono text-sm text-white">Research vault</div>
            <div className="text-xs text-[#9ba5b4]">Papers and scaffolder — inside Research Hub.</div>
          </button>
          <button
            type="button"
            onClick={() => {
              setActivePage('tools-mcp');
              window.scrollTo({ top: 0, behavior: 'smooth' });
            }}
            className="text-left rounded-xl bg-[#06080c] border border-white/10 p-5 space-y-1.5 hover:border-[#389cff]/50 transition cursor-pointer"
          >
            <div className="font-mono text-sm text-white">Runbooks &amp; tools</div>
            <div className="text-xs text-[#9ba5b4]">Harness contracts in the Tools catalog.</div>
          </button>
        </div>
      </div>
    </div>
  </div>
  );
};
