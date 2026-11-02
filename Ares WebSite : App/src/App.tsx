import { Suspense, lazy, useEffect, useState } from 'react';
import { useAtomaStore, type PageId, type SimulationTabId } from './store/useAtomaStore';
import { AtomaBrandLockup } from './components/brand/AtomaBrandAssets';
import { ErrorBoundary } from './components/layout/ErrorBoundary';
import { NotFound } from './components/layout/NotFound';
import { useRouteFocus } from './components/layout/Accessibility';
import { useHashRoute, wasUnknownRoute, clearUnknownRoute, routeTitle } from './lib/router';
import { config } from './lib/config';
import { applySeo } from './lib/seo';
import { logger } from './lib/logger';
import {
  ArrowRight,
  Command,
  Menu,
  Search,
  X,
} from './components/ui/Icons';

// ── Code-split: every route shell is lazy so the initial bundle is
//    React + store + nav only. three.js / @xyflow load on demand. ──
const AtomaCloudApp = lazy(() =>
  import('./components/cloud/AtomaCloudApp').then((m) => ({ default: m.AtomaCloudApp })),
);
const JudgeWalkthroughBar = lazy(() =>
  import('./components/cloud/JudgeWalkthroughAndReceipts').then((m) => ({
    default: m.JudgeWalkthroughBar,
  })),
);
const PlatformHomePage = lazy(() =>
  import('./components/pages/AtomaPages').then((m) => ({ default: m.PlatformHomePage })),
);
const ToolsMcpPage = lazy(() =>
  import('./components/pages/AtomaPages').then((m) => ({ default: m.ToolsMcpPage })),
);
const MetaHackathonPage = lazy(() =>
  import('./components/pages/AtomaPages').then((m) => ({ default: m.MetaHackathonPage })),
);
const ResearchLabPage = lazy(() =>
  import('./components/pages/AtomaPages').then((m) => ({ default: m.ResearchLabPage })),
);
const ResearchHubPublicPage = lazy(() =>
  import('./components/pages/AtomaPages').then((m) => ({ default: m.ResearchHubPublicPage })),
);
const ContactPage = lazy(() =>
  import('./components/pages/AtomaPages').then((m) => ({ default: m.ContactPage })),
);
const SignUpPage = lazy(() =>
  import('./components/pages/AtomaPages').then((m) => ({ default: m.SignUpPage })),
);
const DownloadsPage = lazy(() =>
  import('./components/pages/AtomaPages').then((m) => ({ default: m.DownloadsPage })),
);
const SwarmLegacyPage = lazy(() =>
  import('./components/pages/AtomaPages').then((m) => ({ default: m.SwarmLegacyPage })),
);
const InteractiveSimulationShowcase = lazy(() =>
  import('./components/pages/AtomaPages').then((m) => ({
    default: m.InteractiveSimulationShowcase,
  })),
);

function PageFallback() {
  return (
    <div className="max-w-[1360px] mx-auto px-6 py-24" aria-busy="true" aria-label="Loading section">
      <div className="h-8 w-56 rounded-lg bg-white/8 animate-pulse" />
      <div className="mt-4 h-12 w-3/4 rounded-xl bg-white/6 animate-pulse" />
      <div className="mt-8 grid gap-4 md:grid-cols-3">
        <div className="h-44 rounded-2xl bg-white/5 animate-pulse" />
        <div className="h-44 rounded-2xl bg-white/5 animate-pulse" />
        <div className="h-44 rounded-2xl bg-white/5 animate-pulse" />
      </div>
    </div>
  );
}

export default function App() {
  const {
    activePage,
    setActivePage,
    setCloudTab,
    setActiveSimTab,
    commandPaletteOpen,
    setCommandPaletteOpen,
  } = useAtomaStore();
  const [mobileNavOpen, setMobileNavOpen] = useState(false);

  // Clear a stale 404 flag whenever the user lands on a known page.
  useEffect(() => {
    clearUnknownRoute();
    setMobileNavOpen(false);
  }, [activePage]);

  // CONTRACT: hash is the shareable route (#/platform …). Store stays source of truth.
  useHashRoute();
  useRouteFocus(activePage);
  const unknownRoute = wasUnknownRoute();

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setCommandPaletteOpen(!commandPaletteOpen);
      }
      if (e.key === 'Escape') {
        setCommandPaletteOpen(false);
        setMobileNavOpen(false);
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [commandPaletteOpen, setCommandPaletteOpen]);

  // Focus-trap + scroll-lock for command palette (a11y gate).
  useEffect(() => {
    if (!commandPaletteOpen) return;
    const prev = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Tab') {
        const root = document.getElementById('cmd-palette');
        if (!root) return;
        const items = Array.from(root.querySelectorAll<HTMLElement>('button, [href], input, [tabindex]:not([tabindex="-1"])')).filter((el) => !el.hasAttribute('disabled'));
        if (items.length === 0) return;
        const first = items[0] as HTMLElement;
        const last = items[items.length - 1] as HTMLElement;
        if (e.shiftKey && document.activeElement === first) {
          e.preventDefault();
          last.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    };
    window.addEventListener('keydown', onKey);
    // Initial focus: first actionable element.
    const t = window.setTimeout(() => {
      document.getElementById('cmd-palette')?.querySelector<HTMLElement>('button')?.focus();
    }, 30);
    return () => {
      document.body.style.overflow = prev;
      window.removeEventListener('keydown', onKey);
      window.clearTimeout(t);
    };
  }, [commandPaletteOpen]);

  // Per-route SEO contract — one fact per line, no duplication.
  useEffect(() => {
    try {
      applySeo({
        title: routeTitle(activePage),
        description: seoDescription(activePage),
        path: `/${activePage}`,
        noindex: activePage === 'cloud-app' || activePage === 'sign-up',
      });
    } catch (err) {
      logger.warn('seo', String(err).slice(0, 160));
    }
  }, [activePage]);

  // When inside the authenticated Cloud Workspace ("behind the login wall"), render the full-screen Cloud OS
  if (activePage === 'cloud-app') {
    return (
      <ErrorBoundary lane="cloud-app">
        <Suspense fallback={<PageFallback />}>
          <AtomaCloudApp />
          <JudgeWalkthroughBar />
        </Suspense>
      </ErrorBoundary>
    );
  }

  const navItems: Array<{ id: PageId; label: string }> = [
    { id: 'platform', label: 'Platform' },
    { id: 'tools-mcp', label: 'Tools & MCP' },
    { id: 'research-lab', label: 'Research Lab' },
    { id: 'research-hub', label: 'Research Hub' },
    { id: 'meta-hackathon', label: 'Meta Hack-a-thon' },
    { id: 'ide-workbench', label: 'IDE & DAG Lab' },
    { id: 'contact', label: 'Contact' },
    { id: 'downloads', label: 'Downloads' },
  ];

  const jumpToSim = (sim: SimulationTabId) => {
    setActiveSimTab(sim);
    setActivePage('ide-workbench');
    setCommandPaletteOpen(false);
    window.scrollTo({ top: 0, behavior: 'smooth' });
  };

  return (
    <div className="min-h-screen bg-black text-[#f5f7fa] flex flex-col pb-24">
      <header className="sticky top-0 z-40 bg-black/85 backdrop-blur-xl border-b border-white/8">
        <div className="max-w-[1360px] mx-auto px-6 h-[72px] flex items-center justify-between gap-4">
          <AtomaBrandLockup onClick={() => setActivePage('platform')} />

          <nav aria-label="Primary" className="hidden lg:flex items-center gap-6 text-[13px]">
            {navItems.map((item) => {
              const isActive = activePage === item.id;
              return (
                <button
                  key={item.id}
                  onClick={() => {
                    setActivePage(item.id);
                    window.scrollTo({ top: 0, behavior: 'smooth' });
                  }}
                  aria-current={isActive ? 'page' : undefined}
                  className={`relative py-2 font-medium transition cursor-pointer ${
                    isActive ? 'text-white' : 'text-[#9ba5b4] hover:text-white'
                  }`}
                >
                  {item.label}
                  {isActive && (
                    <span
                      className="absolute left-0 right-0 -bottom-[19px] h-[2px] bg-[#389cff]"
                      aria-hidden="true"
                    />
                  )}
                </button>
              );
            })}
          </nav>

          <div className="flex items-center gap-2.5">
            <button
              onClick={() => setMobileNavOpen((v) => !v)}
              aria-expanded={mobileNavOpen}
              aria-label={mobileNavOpen ? 'Close navigation menu' : 'Open navigation menu'}
              className="lg:hidden inline-flex items-center justify-center w-9 h-9 rounded-lg bg-[#0c0f14] hover:bg-white/8 border border-white/12 text-[#9ba5b4] hover:text-white transition cursor-pointer"
            >
              {mobileNavOpen ? <X className="w-4 h-4" /> : <Menu className="w-4 h-4" />}
            </button>
            <button
              onClick={() => setCommandPaletteOpen(true)}
              className="hidden sm:inline-flex items-center gap-2 px-3 py-1.5 rounded-lg bg-[#0c0f14] hover:bg-white/8 border border-white/12 text-xs text-[#9ba5b4] hover:text-white transition cursor-pointer"
              title="Open Linear Command Palette (⌘K)"
            >
              <Command className="w-3.5 h-3.5 text-[#389cff]" />
              <span className="font-mono text-[11px]">⌘K</span>
            </button>

            <button
              onClick={() => setActivePage('sign-up')}
              className="hidden sm:inline-flex px-3.5 py-1.5 rounded-lg bg-white/6 hover:bg-white/12 border border-white/12 text-xs font-medium text-white transition cursor-pointer"
            >
              Sign in
            </button>

            <button
              onClick={() => {
                setCloudTab('code');
                setActivePage('cloud-app');
                window.scrollTo({ top: 0, behavior: 'smooth' });
              }}
              className="px-4 py-2 rounded-lg bg-[#389cff] hover:bg-[#2589f5] text-white text-xs font-semibold inline-flex items-center gap-1.5 transition shadow-[0_0_20px_-4px_rgba(56,156,255,0.6)] cursor-pointer"
            >
              <span>Launch Cloud App</span>
              <ArrowRight className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      </header>

      {/* Mobile drawer — renders nav + auth when the hamburger (lg:hidden)
          toggle fires. Closes on select / Escape (handled above). */}
      {mobileNavOpen && (
        <div
          className="lg:hidden fixed inset-0 z-30 bg-black/70 backdrop-blur-sm"
          onClick={() => setMobileNavOpen(false)}
          aria-hidden="true"
        >
          <nav
            aria-label="Mobile"
            onClick={(e) => e.stopPropagation()}
            className="absolute top-[72px] left-0 right-0 bg-[#0d1118] border-b border-white/15 px-6 py-4 max-h-[70vh] overflow-y-auto shadow-2xl"
          >
            <div className="grid gap-1">
              {navItems.map((item) => {
                const isActive = activePage === item.id;
                return (
                  <button
                    key={item.id}
                    onClick={() => {
                      setActivePage(item.id);
                      setMobileNavOpen(false);
                      window.scrollTo({ top: 0, behavior: 'smooth' });
                    }}
                    aria-current={isActive ? 'page' : undefined}
                    className={`text-left px-3 py-2.5 rounded-lg text-sm font-medium transition cursor-pointer ${
                      isActive ? 'bg-white/10 text-white' : 'text-[#9ba5b4] hover:text-white hover:bg-white/5'
                    }`}
                  >
                    {item.label}
                  </button>
                );
              })}
              <button
                onClick={() => {
                  setActivePage('sign-up');
                  setMobileNavOpen(false);
                  window.scrollTo({ top: 0, behavior: 'smooth' });
                }}
                className="mt-2 text-left px-3 py-2.5 rounded-lg bg-white/6 border border-white/12 text-sm font-medium text-white cursor-pointer"
              >
                Sign in
              </button>
            </div>
          </nav>
        </div>
      )}

      <main className="flex-1" id="main" tabIndex={-1} aria-label={routeTitle(activePage)}>
        <div key={activePage} className="page-enter">
          <ErrorBoundary lane={`page:${activePage}`} key={activePage}>
            <Suspense fallback={<PageFallback />}>
              {unknownRoute && <NotFound />}
              {!unknownRoute && activePage === 'platform' && <PlatformHomePage />}
              {!unknownRoute && activePage === 'tools-mcp' && <ToolsMcpPage />}
              {!unknownRoute && activePage === 'meta-hackathon' && <MetaHackathonPage />}
              {!unknownRoute && activePage === 'research-lab' && <ResearchLabPage />}
              {!unknownRoute && activePage === 'research-hub' && <ResearchHubPublicPage />}
              {!unknownRoute && activePage === 'ide-workbench' && <InteractiveSimulationShowcase />}
              {!unknownRoute && activePage === 'swarm-legacy' && <SwarmLegacyPage />}
              {!unknownRoute && activePage === 'contact' && <ContactPage />}
              {!unknownRoute && activePage === 'sign-up' && <SignUpPage />}
              {!unknownRoute && activePage === 'downloads' && <DownloadsPage />}
            </Suspense>
          </ErrorBoundary>
        </div>
        <style>{`
          .page-enter { animation: page-in 0.22s cubic-bezier(0.22,1,0.36,1); }
          @keyframes page-in { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: none; } }
          @media (prefers-reduced-motion: reduce) { .page-enter { animation: none; } }
        `}</style>
      </main>

      <footer className="border-t border-white/10 bg-black py-10">
        <div className="max-w-[1360px] mx-auto px-6 flex flex-col md:flex-row items-start md:items-center justify-between gap-6">
          <div className="flex flex-wrap items-center gap-6">
            <AtomaBrandLockup onClick={() => setActivePage('platform')} />
            <span className="hidden sm:inline-block h-5 w-[1px] bg-white/12" />
            <span className="text-xs text-[#9ba5b4]">
              Principled intelligence with receipts.
            </span>
          </div>

          <div className="flex flex-wrap items-center gap-6 text-xs text-[#9ba5b4]">
            {navItems.map((item) => (
              <button
                key={item.id}
                onClick={() => {
                  setActivePage(item.id);
                  window.scrollTo({ top: 0, behavior: 'smooth' });
                }}
                className="hover:text-white transition cursor-pointer"
              >
                {item.label}
              </button>
            ))}
          </div>

          <div className="flex flex-col gap-2 text-xs text-[#9ba5b4] font-mono sm:text-right">
            <span>
              v{config.appVersion} · {config.commitSha.slice(0, 8)}
            </span>
            <button
              onClick={() => {
                setCloudTab('tools');
                setActivePage('cloud-app');
                window.scrollTo({ top: 0, behavior: 'smooth' });
              }}
              className="hover:text-white transition cursor-pointer"
            >
              Open Cloud Tools →
            </button>
            <span>Independent by design. © 2026 Atoma Engineering.</span>
          </div>
        </div>
      </footer>

      {commandPaletteOpen && (
        <div
          onClick={() => setCommandPaletteOpen(false)}
          className="fixed inset-0 z-50 bg-black/75 backdrop-blur-md flex items-start justify-center pt-24 px-4"
        >
          <div
            id="cmd-palette"
            role="dialog"
            aria-modal="true"
            aria-label="Command palette — jump to workspace or simulation"
            onClick={(e) => e.stopPropagation()}
            onKeyDown={(e) => {
              if (e.key === 'Escape') setCommandPaletteOpen(false);
            }}
            className="w-full max-w-xl rounded-2xl bg-[#0d1118] border border-white/15 shadow-2xl overflow-hidden"
          >
            <div className="flex items-center gap-3 px-4 py-3.5 border-b border-white/10">
              <Search className="w-4 h-4 text-[#389cff]" />
              <span className="text-xs font-mono text-[#9ba5b4]">
                Jump to Cloud Workspace, Research Hub Scaffolder, or any Atoma simulation...
              </span>
              <button
                onClick={() => setCommandPaletteOpen(false)}
                className="ml-auto text-[#9ba5b4] hover:text-white"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="p-3 space-y-3 text-xs max-h-[420px] overflow-y-auto">
              <div>
                <div className="px-2 py-1 font-mono text-[10px] uppercase tracking-wider text-[#389cff]">
                  Authenticated Cloud Workspace (Behind Login)
                </div>
                <div className="grid grid-cols-2 gap-1.5 mt-1">
                  {[
                    { tab: 'code' as const, label: '08 · Code IDE & Agent Command' },
                    { tab: 'swarm' as const, label: '09 · Swarm Holonomy & DAG' },
                    { tab: 'mind' as const, label: '10 · Mind 3,482-Node Graph' },
                    { tab: 'research' as const, label: '11 · Research Hub & Scaffolder' },
                  ].map((item) => (
                    <button
                      key={item.tab}
                      onClick={() => {
                        setCloudTab(item.tab);
                        setActivePage('cloud-app');
                        setCommandPaletteOpen(false);
                      }}
                      className="px-3 py-2.5 rounded-lg bg-[#389cff]/15 hover:bg-[#389cff]/25 border border-[#389cff]/30 text-left text-white font-semibold transition cursor-pointer"
                    >
                      {item.label}
                    </button>
                  ))}
                </div>
              </div>

              <div className="pt-2 border-t border-white/8">
                <div className="px-2 py-1 font-mono text-[10px] uppercase tracking-wider text-[#9ba5b4]">
                  Interactive Public Simulations
                </div>
                <div className="space-y-1 mt-1">
                  {[
                    {
                      id: 'flare-dag' as const,
                      title: 'Flare Codebase Dependency DAG & IDE',
                      desc: '98 nodes · 220 edges · Activity/Risk/Hotspots lenses + PowerShell agent terminal',
                    },
                    {
                      id: 'orbital-os' as const,
                      title: 'Orbital Agent OS (brigade://pipeline/orchestrator)',
                      desc: '12 autonomous agents · Rust + C++ WebGPU dual editors · Live GPU telemetry',
                    },
                    {
                      id: 'ares-runbook' as const,
                      title: 'ARES Spec Ops Runbook Loop Explorer',
                      desc: 'Scope → Plan → Execute → Test → Human review → N³ SQLite Receipt',
                    },
                    {
                      id: 'galaxy-3d' as const,
                      title: '3D Galaxy / Nebula / Obsidian Vault Force Graph',
                      desc: '2,400 WebGL nodes · PageRank size · Repulsion & Pull physics sliders',
                    },
                  ].map((sim) => (
                    <button
                      key={sim.id}
                      onClick={() => jumpToSim(sim.id)}
                      className="w-full px-3 py-2 rounded-lg hover:bg-white/8 flex items-center justify-between text-left transition cursor-pointer"
                    >
                      <div>
                        <div className="font-semibold text-white">{sim.title}</div>
                        <div className="text-[11px] text-[#9ba5b4]">{sim.desc}</div>
                      </div>
                      <ArrowRight className="w-4 h-4 text-[#389cff]" />
                    </button>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      <ErrorBoundary lane="judge-bar">
        <Suspense fallback={null}>
          <JudgeWalkthroughBar />
        </Suspense>
      </ErrorBoundary>
    </div>
  );
}

function seoDescription(page: PageId): string {
  switch (page) {
    case 'cloud-app':
      return 'Authenticated Atoma Cloud workspace — code IDE, swarm DAG, mind graph and research hub (sign-in required).';
    case 'tools-mcp':
      return 'MCP tool registry, seatbelt execution lanes and protocol contracts for the Atoma agent OS.';
    case 'research-lab':
      return 'ARES Spec Ops research lab — runbook loops, receipts and evaluation harnesses.';
    case 'research-hub':
      return 'Atoma research hub — papers, scaffolder and evidence-linked engineering notes.';
    case 'meta-hackathon':
      return 'Meta hack-a-thon track — solo-builder operating system for agent ideas.';
    case 'ide-workbench':
      return 'Interactive IDE & DAG lab — Flare dependency graph, orbital agent OS, runbook explorer and 3D galaxy vault.';
    case 'contact':
      return 'Contact Atoma Engineering — partnerships, research and deployment inquiries.';
    case 'sign-up':
      return 'Sign in to the Atoma Cloud workspace.';
    case 'downloads':
      return 'Download Atoma artifacts — CLIs, receipts and release bundles.';
    case 'swarm-legacy':
      return 'Legacy swarm telemetry view — superseded by the Cloud swarm holonomy tab.';
    default:
      return 'Atoma Engineering ARES Spec Ops — hyperbolic memory, live swarm telemetry and N³ receipts.';
  }
}
