import React, { useState } from 'react';
import {
  useAtomaStore,
  RUNBOOK_NODES,
  type RunbookNodeData,
} from '../../store/useAtomaStore';
import {
  CheckCircle2,
  Code2,
  FileCheck2,
  Play,
  ShieldAlert,
} from '../ui/Icons';

export const AresRunbookLab: React.FC = () => {
  const {
    runbookActiveStep,
    setRunbookActiveStep,
    runbookExecuting,
    triggerRunbookLoop,
    runbookCompletedIds,
  } = useAtomaStore();

  const [activeTab, setActiveTab] = useState<'Workflow' | 'Details' | 'Runs' | 'Evidence'>(
    'Workflow'
  );
  const activeNode = RUNBOOK_NODES[runbookActiveStep];

  const orderedSteps: RunbookNodeData['id'][] = [
    'scope',
    'plan',
    'execute',
    'test',
    'review',
    'receipt',
  ];

  return (
    <div className="titanium-shell">
      <div className="rounded-[14px] bg-[#080b10] border border-white/12 overflow-hidden text-[#f5f7fa]">
        <div className="flex flex-wrap items-center justify-between gap-2 px-4 py-3 bg-gradient-to-b from-white/10 to-[#0c1017] border-b border-white/10">
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-1.5">
              <span className="w-2.5 h-2.5 rounded-full bg-[#389cff]" />
              <span className="w-2.5 h-2.5 rounded-full bg-white/25" />
              <span className="w-2.5 h-2.5 rounded-full bg-white/25" />
            </div>
            <span className="font-mono text-xs font-semibold tracking-wider text-white">
              ARES SPEC OPS · RUNBOOK EXPLORER
            </span>
            <span className="px-2 py-0.5 rounded-full bg-[#389cff]/15 border border-[#389cff]/40 text-[#389cff] font-mono text-[10px]">
              N³ SQLite Connected
            </span>
          </div>

          <div className="flex items-center gap-3 text-xs">
            <span className="flex items-center gap-1.5 text-[#9ba5b4]">
              <span className="w-2 h-2 rounded-full bg-[#389cff]" />
              Design preview · Interactive simulation
            </span>
            <button
              onClick={triggerRunbookLoop}
              disabled={runbookExecuting}
              className="px-3.5 py-1.5 rounded-full bg-[#389cff] hover:bg-[#2589f5] disabled:opacity-50 text-white font-medium text-xs flex items-center gap-1.5 transition cursor-pointer"
            >
              <Play className="w-3.5 h-3.5 fill-current" />
              {runbookExecuting ? 'Executing Runbook Loop...' : 'Run Loop Simulation'}
            </button>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-[220px_1fr_320px]">
          <aside className="bg-[#0a0d13] border-r border-white/8 p-4 flex flex-col justify-between">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <span className="font-display font-semibold text-sm text-white">
                  ATOMA <span className="text-[#389cff] font-mono text-xs">ARES</span>
                </span>
                <span className="px-2 py-0.5 rounded bg-[#389cff]/15 text-[#389cff] font-mono text-[10px]">
                  Experimental
                </span>
              </div>

              <div className="space-y-1 text-xs">
                {[
                  { label: 'Runbooks', active: true },
                  { label: 'Research lab', active: false },
                  { label: 'Tools & MCP', active: false },
                  { label: 'Memory (N³ SQLite)', active: false },
                  { label: 'Evidence Receipts', active: false },
                ].map((nav) => (
                  <div
                    key={nav.label}
                    className={`px-3 py-2 rounded-lg font-medium flex items-center justify-between ${
                      nav.active
                        ? 'bg-[#389cff]/15 text-white border border-[#389cff]/30'
                        : 'text-[#9ba5b4]'
                    }`}
                  >
                    <span>{nav.label}</span>
                    {nav.active && <span className="w-1.5 h-1.5 rounded-full bg-[#389cff]" />}
                  </div>
                ))}
              </div>

              <div className="pt-4 border-t border-white/8 space-y-2.5">
                <div className="font-mono text-[10px] uppercase tracking-wider text-[#9ba5b4]">
                  RUNBOOK CONTRACT GATES
                </div>
                {orderedSteps.map((stepId, index) => {
                  const node = RUNBOOK_NODES[stepId];
                  const isCurrent = runbookActiveStep === stepId;
                  const isDone = runbookCompletedIds.includes(stepId);
                  return (
                    <button
                      key={stepId}
                      onClick={() => setRunbookActiveStep(stepId)}
                      className={`w-full p-2.5 rounded-lg text-left border transition ${
                        isCurrent
                          ? 'bg-[#121824] border-[#389cff] shadow-[0_0_20px_-6px_rgba(56,156,255,0.4)]'
                          : 'bg-[#0d1119]/60 border-white/6 hover:border-white/20'
                      }`}
                    >
                      <div className="flex items-center justify-between text-xs font-medium text-white">
                        <span>
                          0{index + 1}. {node.label}
                        </span>
                        <span
                          className={`w-2 h-2 rounded-full ${
                            isCurrent
                              ? 'bg-[#389cff] animate-ping'
                              : isDone
                              ? 'bg-emerald-400'
                              : 'bg-white/20'
                          }`}
                        />
                      </div>
                      <p className="text-[11px] text-[#9ba5b4] mt-0.5 line-clamp-1">
                        {node.subtitle}
                      </p>
                    </button>
                  );
                })}
              </div>
            </div>

            <div className="pt-4 border-t border-white/8 text-[11px] text-[#9ba5b4] leading-relaxed">
              Every step records an inspectable N³ SQLite receipt linking intent, tool calls, and
              human acceptance.
            </div>
          </aside>

          <div className="flex flex-col min-w-0 bg-[#070a0f]">
            <div className="px-6 pt-5 pb-3 border-b border-white/8 flex flex-wrap items-center justify-between gap-4">
              <div>
                <div className="text-[11px] font-mono text-[#9ba5b4]">
                  Runbooks › Evaluate a new runbook loop
                </div>
                <div className="flex items-center gap-3 mt-1">
                  <h3 className="text-lg font-display font-semibold text-white">
                    Product experiment runbook
                  </h3>
                  <span className="px-2.5 py-0.5 rounded-full bg-[#389cff]/15 border border-[#389cff]/30 text-[#389cff] text-[11px] font-medium">
                    In progress
                  </span>
                </div>
              </div>

              <div className="flex items-center gap-1 bg-[#0e131d] p-1 rounded-lg border border-white/8 text-xs">
                {(['Workflow', 'Details', 'Runs', 'Evidence'] as const).map((t) => (
                  <button
                    key={t}
                    onClick={() => setActiveTab(t)}
                    className={`px-3 py-1 rounded-md font-medium transition ${
                      activeTab === t
                        ? 'bg-[#389cff] text-white'
                        : 'text-[#9ba5b4] hover:text-white'
                    }`}
                  >
                    {t}
                  </button>
                ))}
              </div>
            </div>

            <div className="p-6 linear-dot-bg relative">
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 relative z-10">
                {orderedSteps.map((stepId, idx) => {
                  const node = RUNBOOK_NODES[stepId];
                  const isSelected = runbookActiveStep === stepId;
                  const isCompleted = runbookCompletedIds.includes(stepId);

                  return (
                    <button
                      key={stepId}
                      onClick={() => setRunbookActiveStep(stepId)}
                      className={`p-4 rounded-xl text-left transition-all border relative group ${
                        isSelected
                          ? 'bg-[#101826] border-[#389cff] shadow-[0_0_30px_-5px_rgba(56,156,255,0.45)] scale-[1.02]'
                          : 'bg-[#0c1018]/95 border-white/12 hover:border-[#389cff]/50'
                      }`}
                    >
                      <div className="flex items-center justify-between mb-2">
                        <span className="font-mono text-[10px] uppercase tracking-wider px-2 py-0.5 rounded bg-white/6 text-[#c9cfd7]">
                          STEP 0{idx + 1}
                        </span>
                        <span className="font-mono text-[10px] text-[#389cff]">
                          {node.duration}
                        </span>
                      </div>

                      <div className="text-base font-display font-semibold text-white flex items-center justify-between">
                        <span>{node.label}</span>
                        {isCompleted && (
                          <CheckCircle2 className="w-4 h-4 text-[#389cff] shrink-0" />
                        )}
                      </div>

                      <p className="text-xs text-[#9ba5b4] mt-1 leading-relaxed">
                        {node.subtitle}
                      </p>

                      <div className="mt-3 pt-2.5 border-t border-white/8 flex items-center justify-between font-mono text-[10px] text-[#9ba5b4]">
                        <span>{node.agent}</span>
                        <span className="text-[#389cff]">{node.sqlHash}</span>
                      </div>
                    </button>
                  );
                })}
              </div>
            </div>

            <div className="border-t border-white/8 bg-[#090d14] p-5 flex-1 flex flex-col justify-between">
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center gap-2 font-mono text-xs text-white">
                  <Code2 className="w-4 h-4 text-[#389cff]" />
                  <span>
                    Active Artifact — <strong>{activeNode.label}</strong> ({activeNode.sqlHash})
                  </span>
                </div>
                <span className="font-mono text-[11px] text-[#9ba5b4]">
                  Agent: <strong className="text-[#c9cfd7]">{activeNode.agent}</strong>
                </span>
              </div>

              <pre className="p-4 rounded-xl bg-[#05070b] border border-white/10 font-mono text-xs text-[#dbe4f0] overflow-x-auto leading-relaxed">
                {activeNode.codeSnippet}
              </pre>
            </div>
          </div>

          <aside className="bg-[#0a0d13] border-l border-white/8 p-5 flex flex-col justify-between space-y-5">
            <div className="space-y-4">
              <div className="flex items-center justify-between border-b border-white/8 pb-3">
                <div>
                  <div className="text-xs font-mono uppercase tracking-wider text-[#9ba5b4]">
                    EVIDENCE INSPECTOR
                  </div>
                  <h4 className="text-sm font-semibold text-white mt-0.5">
                    Runbook: Product experiment
                  </h4>
                </div>
                <span className="px-2 py-1 rounded bg-white/6 font-mono text-[11px] text-[#c9cfd7]">
                  Run #3
                </span>
              </div>

              <div className="space-y-2 text-xs border-b border-white/8 pb-4">
                <div className="flex items-center justify-between">
                  <span className="text-[#9ba5b4]">Status</span>
                  <span className="inline-flex items-center gap-1.5 text-white font-medium">
                    <span className="w-2 h-2 rounded-full bg-[#389cff]" /> Design preview
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-[#9ba5b4]">Acceptance</span>
                  <span className="inline-flex items-center gap-1.5 text-amber-300 font-medium">
                    <span className="w-2 h-2 rounded-full bg-amber-400" /> Human review required
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-[#9ba5b4]">Traceability</span>
                  <span className="inline-flex items-center gap-1.5 text-emerald-300 font-mono text-[11px]">
                    N³ SQLite ({activeNode.sqlHash})
                  </span>
                </div>
              </div>

              <div className="space-y-2">
                <div className="text-xs font-semibold text-white">
                  Step Verification: {activeNode.label}
                </div>
                <p className="text-xs text-[#9ba5b4] leading-relaxed">{activeNode.details}</p>
              </div>

              <div className="p-3.5 rounded-xl bg-[#0e131d] border border-white/8 space-y-2">
                <div className="text-[11px] font-mono uppercase tracking-wider text-[#9ba5b4]">
                  VERIFIED CHECKS
                </div>
                {activeNode.checks.map((check, i) => (
                  <div key={i} className="flex items-center gap-2 text-xs text-[#dbe4f0]">
                    <CheckCircle2 className="w-3.5 h-3.5 text-[#389cff] shrink-0" />
                    <span>{check}</span>
                  </div>
                ))}
              </div>

              <div className="p-3.5 rounded-xl bg-amber-500/8 border border-amber-500/25 space-y-1">
                <div className="flex items-center gap-1.5 text-xs font-semibold text-amber-300">
                  <ShieldAlert className="w-3.5 h-3.5" /> Limitations
                </div>
                <p className="text-[11px] text-[#9ba5b4] leading-relaxed">
                  Experimental workflow. Results require explicit human review before promotion.
                </p>
              </div>
            </div>

            <div className="space-y-2 pt-2">
              <button
                onClick={() => {
                  const idx = orderedSteps.indexOf(runbookActiveStep);
                  const next = orderedSteps[(idx + 1) % orderedSteps.length];
                  setRunbookActiveStep(next);
                }}
                className="w-full py-2.5 px-4 rounded-lg bg-[#389cff] hover:bg-[#2589f5] text-white font-medium text-xs flex items-center justify-center gap-2 transition cursor-pointer"
              >
                <span>Inspect next step →</span>
              </button>
              <button
                onClick={() => setRunbookActiveStep('receipt')}
                className="w-full py-2.5 px-4 rounded-lg bg-white/6 hover:bg-white/12 border border-white/10 text-white font-medium text-xs flex items-center justify-center gap-2 transition cursor-pointer"
              >
                <FileCheck2 className="w-3.5 h-3.5 text-[#389cff]" />
                <span>View N³ receipt structure</span>
              </button>
            </div>
          </aside>
        </div>
      </div>
    </div>
  );
};
