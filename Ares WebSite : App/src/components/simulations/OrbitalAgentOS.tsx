import React, { useState } from 'react';
import { useAtomaStore } from '../../store/useAtomaStore';
import {
  CheckCircle2,
  Cpu,
  GitMerge,
  Play,
  Pause,
  RotateCw,
} from '../ui/Icons';

interface OrbitalNode {
  id: string;
  title: string;
  role: string;
  row: number;
  col: number;
  meta1: string;
  meta2: string;
  meta3: string;
  status: 'active' | 'synced' | 'ready';
}

const ORBITAL_DAG_NODES: OrbitalNode[] = [
  {
    id: 'researcher-05',
    title: 'researcher-05',
    role: 'Web Search + Docs',
    row: 0,
    col: 0,
    meta1: 'Query: autonomous systems safety',
    meta2: 'Sources: web, arxiv, github',
    meta3: 'Results: 12 sources',
    status: 'active',
  },
  {
    id: 'planner-01',
    title: 'planner-01',
    role: 'Task Planning',
    row: 0,
    col: 1,
    meta1: 'Plan: 6 steps',
    meta2: 'Estimated time: 4m 30s',
    meta3: 'Complexity: medium',
    status: 'active',
  },
  {
    id: 'coder-02',
    title: 'coder-02',
    role: 'Code Generation',
    row: 0,
    col: 2,
    meta1: 'Language: Rust + C++',
    meta2: 'Files: 8',
    meta3: 'Tests: 14',
    status: 'active',
  },
  {
    id: 'memory-08',
    title: 'memory-08',
    role: 'Long-term Memory',
    row: 1,
    col: 0,
    meta1: 'Store: research, code, results',
    meta2: 'Vector DB: 2,450vecs',
    meta3: 'Status: synced',
    status: 'synced',
  },
  {
    id: 'critic-06',
    title: 'critic-06',
    role: 'Code Review',
    row: 1,
    col: 1,
    meta1: 'Issues: 2 (minor)',
    meta2: 'Quality: 94/100',
    meta3: 'Approved: true',
    status: 'active',
  },
  {
    id: 'executor-03',
    title: 'executor-03',
    role: 'Build & Run',
    row: 1,
    col: 2,
    meta1: 'Build: success',
    meta2: 'Tests: 14/14',
    meta3: 'Runtime: 38s',
    status: 'active',
  },
  {
    id: 'navigator-07',
    title: 'navigator-07',
    role: 'Tool Use',
    row: 2,
    col: 0,
    meta1: 'Tools: github, terminal, browser',
    meta2: 'Actions: 6',
    meta3: 'Success: 100%',
    status: 'active',
  },
  {
    id: 'optimizer-09',
    title: 'optimizer-09',
    role: 'Performance Tuning',
    row: 2,
    col: 1,
    meta1: 'Optimizations: 3',
    meta2: 'Speedup: 18%',
    meta3: 'Status: complete',
    status: 'ready',
  },
  {
    id: 'deployer-10',
    title: 'deployer-10',
    role: 'Deployment',
    row: 2,
    col: 2,
    meta1: 'Target: k8s + local N³',
    meta2: 'Environment: staging',
    meta3: 'Status: ready',
    status: 'ready',
  },
];

const AGENT_ROSTER = [
  { id: 'planner-01', role: 'Task Planning', cpu: '12%', ram: '1.2 GB' },
  { id: 'coder-02', role: 'Code Generation', cpu: '18%', ram: '2.4 GB' },
  { id: 'executor-03', role: 'Build & Run', cpu: '9%', ram: '0.8 GB' },
  { id: 'verifier-04', role: 'Testing & Validation', cpu: '14%', ram: '1.6 GB' },
  { id: 'researcher-05', role: 'Knowledge Retrieval', cpu: '7%', ram: '0.9 GB' },
  { id: 'critic-06', role: 'Code Review', cpu: '11%', ram: '1.1 GB' },
  { id: 'navigator-07', role: 'Web / Tool Use', cpu: '16%', ram: '1.8 GB' },
  { id: 'memory-08', role: 'Long-term Memory', cpu: '8%', ram: '0.7 GB' },
  { id: 'optimizer-09', role: 'Performance Tuning', cpu: '13%', ram: '1.5 GB' },
  { id: 'deployer-10', role: 'Deployment', cpu: '5%', ram: '0.6 GB' },
];

const RUST_MAIN_CODE = `use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, error};

pub async fn run_agent(config: AgentConfig) -> Result<()> {
    let (tx, mut rx) = mpsc::channel(32);
    let model = load_model(&config.model_path).await?;
    let mut executor = CodeExecutor::new(config).await?;

    loop {
        tokio::select! {
            Some(task) = rx.recv() => {
                let result = executor.execute(task).await?;
                tx.send(result).await?;
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                info!("agent heartbeat · N3 receipt synced");
            }
        }
    }
}`;

const CPP_WEBGPU_CODE = `#include <iostream>
#include <memory>
#include <vector>
#include <webgpu/webgpu.hpp>

class GPUEngine {
public:
    explicit GPUEngine() {
        init_webgpu();
    }

    void run_inference(const std::vector<float>& input) {
        wgpu::ComputePassEncoder pass = encoder.BeginComputePass();
        pass.SetPipeline(pipeline_);
        pass.DispatchWorkgroups((input.size() + 63) / 64, 1, 1);
        pass.EndPass();
    }

private:
    void init_webgpu() {
        // Initialize WebGPU device and pipeline
        std::cout << "WebGPU initialized" << std::endl;
    }
};`;

export const OrbitalAgentOS: React.FC = () => {
  const {
    orbitalRunning,
    setOrbitalRunning,
    orbitalSelectedNode,
    setOrbitalSelectedNode,
  } = useAtomaStore();

  const [rustCode, setRustCode] = useState(RUST_MAIN_CODE);
  const [cppCode, setCppCode] = useState(CPP_WEBGPU_CODE);
  const [buildCount] = useState(14);
  const [lastBuildTime, setLastBuildTime] = useState('38.2s');

  const triggerRebuild = () => {
    setLastBuildTime((36 + Math.random() * 3).toFixed(1) + 's');
  };

  return (
    <div className="w-full rounded-xl overflow-hidden border border-cyan-500/25 bg-[#05080d] text-[#e2e8f0] shadow-[0_0_60px_-15px_rgba(6,182,212,0.25)] select-none">
      <div className="flex items-center justify-between px-4 py-2.5 bg-[#090d14] border-b border-cyan-500/15">
        <div className="flex items-center gap-2">
          <span className="w-3 h-3 rounded-full bg-rose-500/70" />
          <span className="w-3 h-3 rounded-full bg-amber-500/70" />
          <span className="w-3 h-3 rounded-full bg-emerald-500/70" />
          <span className="ml-3 font-display font-bold tracking-wider text-xs text-white flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-cyan-400 animate-pulse" />
            ORBITAL <span className="text-[10px] font-mono text-cyan-400/80">AGENT OS</span>
          </span>
        </div>

        <div className="px-4 py-1 rounded-md bg-[#111722] border border-cyan-500/20 font-mono text-xs text-cyan-200">
          brigade://pipeline/orchestrator
        </div>

        <div className="flex items-center gap-3 text-xs font-mono">
          <span className="text-[#9ba5b4] hidden md:inline">Cluster: prod-eu-1</span>
          <span className="text-cyan-300 hidden sm:inline">Agents: 12/16</span>
          <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-emerald-500/15 border border-emerald-500/30 text-emerald-300 text-[11px]">
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-ping" /> LIVE
          </span>
        </div>
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-[210px_1fr_290px]">
        <aside className="bg-[#070b11] border-r border-cyan-500/15 p-3 flex flex-col justify-between space-y-4">
          <div className="space-y-4">
            <div className="space-y-1 text-xs">
              {['Home', 'Agents', 'Workflows', 'Code', 'Telemetry', 'Compute', 'Storage'].map(
                (item) => (
                  <div
                    key={item}
                    className={`px-2.5 py-1.5 rounded flex items-center justify-between cursor-pointer transition ${
                      item === 'Workflows'
                        ? 'bg-cyan-500/15 text-cyan-300 border border-cyan-500/30 font-medium'
                        : 'text-[#8e96a8] hover:text-white hover:bg-white/5'
                    }`}
                  >
                    <span>{item}</span>
                    <span className="text-[10px]">›</span>
                  </div>
                )
              )}
            </div>

            <div className="pt-2 border-t border-white/8">
              <div className="flex items-center justify-between text-[10px] font-mono uppercase tracking-wider text-[#8e96a8] mb-2">
                <span>AGENTS (12)</span>
                <span className="text-cyan-400">ONLINE</span>
              </div>
              <div className="space-y-1 max-h-64 overflow-y-auto pr-1">
                {AGENT_ROSTER.map((agent) => {
                  const selected = orbitalSelectedNode === agent.id;
                  return (
                    <button
                      key={agent.id}
                      onClick={() => setOrbitalSelectedNode(agent.id)}
                      className={`w-full p-1.5 rounded text-left transition flex items-center justify-between ${
                        selected
                          ? 'bg-cyan-500/15 border border-cyan-400/40'
                          : 'hover:bg-white/5 border border-transparent'
                      }`}
                    >
                      <div className="min-w-0">
                        <div className="flex items-center gap-1.5 text-[11px] font-mono text-white truncate">
                          <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 shrink-0" />
                          {agent.id}
                        </div>
                        <div className="text-[10px] text-[#64748b] truncate pl-3">{agent.role}</div>
                      </div>
                      <div className="text-right font-mono text-[9px] text-[#9ba5b4] shrink-0">
                        <div>CPU {agent.cpu}</div>
                        <div>{agent.ram}</div>
                      </div>
                    </button>
                  );
                })}
              </div>
            </div>
          </div>

          <div className="pt-3 border-t border-white/8">
            <div className="text-[10px] font-mono uppercase tracking-wider text-[#8e96a8] mb-2">
              SYSTEM RESOURCES
            </div>
            <div className="grid grid-cols-4 gap-1.5 text-center font-mono">
              {[
                { label: 'CPU', val: '42%', color: 'border-cyan-400 text-cyan-300' },
                { label: 'GPU', val: '68%', color: 'border-purple-400 text-purple-300' },
                { label: 'RAM', val: '31%', color: 'border-blue-400 text-blue-300' },
                { label: 'DISK', val: '12%', color: 'border-emerald-400 text-emerald-300' },
              ].map((g) => (
                <div key={g.label} className="flex flex-col items-center">
                  <div
                    className={`w-10 h-10 rounded-full border-2 ${g.color} flex items-center justify-center text-[10px] font-semibold bg-black/50`}
                  >
                    {g.val}
                  </div>
                  <span className="text-[9px] text-[#64748b] mt-1">{g.label}</span>
                </div>
              ))}
            </div>
          </div>
        </aside>

        <div className="flex flex-col min-w-0">
          <div className="flex flex-wrap items-center justify-between gap-2 px-4 py-2 bg-[#0a0f18] border-b border-cyan-500/15 text-xs">
            <div className="flex items-center gap-2.5">
              <GitMerge className="w-4 h-4 text-cyan-400" />
              <span className="font-mono font-semibold text-white">autonomous-research-flow</span>
              <span className="px-2 py-0.5 rounded-full bg-emerald-500/15 border border-emerald-500/30 text-emerald-300 text-[10px] font-mono">
                ● {orbitalRunning ? 'Running' : 'Paused'}
              </span>
              <span className="text-[11px] font-mono text-[#64748b]">v2.4.1</span>
            </div>

            <div className="flex items-center gap-2">
              <button
                onClick={() => setOrbitalRunning(!orbitalRunning)}
                className="px-2.5 py-1 rounded bg-cyan-500/15 hover:bg-cyan-500/25 border border-cyan-400/30 text-cyan-200 flex items-center gap-1 text-[11px] font-mono transition"
              >
                {orbitalRunning ? (
                  <>
                    <Pause className="w-3 h-3" /> Pause
                  </>
                ) : (
                  <>
                    <Play className="w-3 h-3" /> Resume
                  </>
                )}
              </button>
              <div className="flex rounded bg-[#111824] p-0.5 border border-white/8 text-[11px]">
                <span className="px-2.5 py-0.5 rounded bg-cyan-500/20 text-cyan-300 font-medium">
                  Graph
                </span>
                <span className="px-2.5 py-0.5 text-[#8e96a8]">Code</span>
                <span className="px-2.5 py-0.5 text-[#8e96a8]">Logs</span>
                <span className="px-2.5 py-0.5 text-[#8e96a8]">Metrics</span>
              </div>
            </div>
          </div>

          <div className="p-4 linear-dot-bg bg-[#060a10] border-b border-cyan-500/15">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-3 relative">
              {ORBITAL_DAG_NODES.map((node) => {
                const isSelected = orbitalSelectedNode === node.id;
                return (
                  <button
                    key={node.id}
                    onClick={() => setOrbitalSelectedNode(node.id)}
                    className={`p-3 rounded-lg text-left transition-all border relative ${
                      isSelected
                        ? 'bg-[#0d1826] border-cyan-400 shadow-[0_0_25px_-5px_rgba(6,182,212,0.45)]'
                        : 'bg-[#0a111a]/90 border-cyan-500/20 hover:border-cyan-400/50'
                    }`}
                  >
                    <div className="flex items-center justify-between mb-1">
                      <span className="font-mono text-xs font-semibold text-white flex items-center gap-1.5">
                        <Cpu className="w-3.5 h-3.5 text-cyan-400" />
                        {node.title}
                      </span>
                      <span
                        className={`w-2 h-2 rounded-full ${
                          orbitalRunning ? 'bg-emerald-400 animate-pulse' : 'bg-amber-400'
                        }`}
                      />
                    </div>
                    <div className="text-[11px] text-cyan-300/90 font-medium mb-2">
                      {node.role}
                    </div>
                    <div className="space-y-0.5 font-mono text-[10px] text-[#9ba5b4]">
                      <div className="truncate">{node.meta1}</div>
                      <div className="truncate">{node.meta2}</div>
                      <div className="text-cyan-400">{node.meta3}</div>
                    </div>
                  </button>
                );
              })}
            </div>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-[1fr_1fr_250px] bg-[#070b12] flex-1">
            <div className="border-r border-white/8 flex flex-col">
              <div className="flex items-center justify-between px-3 py-1.5 border-b border-white/8 bg-[#0b1018] font-mono text-[11px]">
                <span className="text-amber-400 font-semibold">🦀 main.rs</span>
                <span className="text-[10px] text-[#64748b]">agents / coder-02 / src / main.rs</span>
              </div>
              <textarea
                value={rustCode}
                onChange={(e) => setRustCode(e.target.value)}
                spellCheck={false}
                className="w-full flex-1 min-h-[230px] p-3 bg-transparent font-mono text-[11px] leading-relaxed text-[#cbd5e1] focus:outline-none resize-none"
              />
            </div>

            <div className="border-r border-white/8 flex flex-col">
              <div className="flex items-center justify-between px-3 py-1.5 border-b border-white/8 bg-[#0b1018] font-mono text-[11px]">
                <span className="text-cyan-400 font-semibold">⚡ agent.cpp</span>
                <span className="text-[10px] text-[#64748b]">
                  agents / executor-03 / src / agent.cpp
                </span>
              </div>
              <textarea
                value={cppCode}
                onChange={(e) => setCppCode(e.target.value)}
                spellCheck={false}
                className="w-full flex-1 min-h-[230px] p-3 bg-transparent font-mono text-[11px] leading-relaxed text-[#cbd5e1] focus:outline-none resize-none"
              />
            </div>

            <div className="p-3 flex flex-col justify-between font-mono text-[11px] bg-[#06090f]">
              <div className="space-y-2.5">
                <div className="flex items-center justify-between border-b border-white/8 pb-1.5">
                  <span className="text-white font-semibold">Build &amp; Run</span>
                  <button
                    onClick={triggerRebuild}
                    className="px-2 py-0.5 rounded bg-cyan-500/20 hover:bg-cyan-500/30 text-cyan-300 text-[10px] flex items-center gap-1"
                  >
                    <RotateCw className="w-2.5 h-2.5" /> Re-run
                  </button>
                </div>

                <div className="p-2 rounded bg-emerald-500/10 border border-emerald-500/25">
                  <div className="text-emerald-400 font-semibold flex items-center gap-1.5">
                    <CheckCircle2 className="w-3.5 h-3.5" /> Success
                  </div>
                  <div className="text-[10px] text-[#9ba5b4]">
                    Build completed in {lastBuildTime}
                  </div>
                </div>

                <div className="space-y-1 text-[10px] text-emerald-300/90">
                  <div>✓ Compiling agent-executor</div>
                  <div>✓ Compiling dependencies</div>
                  <div>✓ Linking WebGPU shaders</div>
                  <div>✓ Running tests ({buildCount}/14)</div>
                </div>

                <div className="pt-2 border-t border-white/8 space-y-1 text-[10px] text-[#9ba5b4]">
                  <div className="text-white">$ cargo build --release</div>
                  <div>Initializing WebGPU...</div>
                  <div className="text-cyan-300">GPU: NVIDIA GeForce RTX 4090</div>
                  <div>Compute shaders: enabled</div>
                  <div>Memory bandwidth: 0.92 TB/s</div>
                  <div className="text-emerald-400">Inference completed in 1.8s</div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <aside className="bg-[#070b11] border-l border-cyan-500/15 p-3 space-y-4 font-mono">
          <div className="flex items-center justify-between border-b border-white/8 pb-2">
            <span className="text-xs font-semibold text-white">TELEMETRY</span>
            <div className="flex gap-1 text-[10px]">
              <span className="px-2 py-0.5 rounded bg-cyan-500/20 text-cyan-300">Real-time</span>
              <span className="px-2 py-0.5 text-[#64748b]">Traces</span>
            </div>
          </div>

          <div className="space-y-1.5">
            <div className="flex items-center justify-between text-[10px] text-[#9ba5b4]">
              <span>AGENT PERFORMANCE</span>
              <span className="text-cyan-300">CPU · RAM · GPU</span>
            </div>
            <div className="h-24 rounded bg-[#0a1018] border border-white/8 p-2 flex items-end">
              <svg viewBox="0 0 240 70" className="w-full h-full overflow-visible">
                <path
                  d="M0,52 L25,44 L50,48 L75,30 L100,36 L125,22 L150,28 L175,18 L200,26 L240,14 L240,70 L0,70 Z"
                  fill="rgba(6, 182, 212, 0.18)"
                />
                <path
                  d="M0,52 L25,44 L50,48 L75,30 L100,36 L125,22 L150,28 L175,18 L200,26 L240,14"
                  fill="none"
                  stroke="#06b6d4"
                  strokeWidth="2"
                />
                <path
                  d="M0,60 L25,54 L50,58 L75,45 L100,48 L125,38 L150,42 L175,33 L200,39 L240,29"
                  fill="none"
                  stroke="#a855f7"
                  strokeWidth="1.5"
                />
              </svg>
            </div>
          </div>

          <div className="space-y-1.5 pt-1 border-t border-white/8">
            <div className="text-[10px] text-[#9ba5b4]">WORKFLOW METRICS</div>
            <div className="grid grid-cols-3 gap-2">
              <div className="p-2 rounded bg-[#0b111a] border border-white/8">
                <div className="text-[9px] text-[#64748b]">Tasks/s</div>
                <div className="text-sm font-bold text-white">4.8</div>
                <div className="text-[9px] text-emerald-400">↑ 12%</div>
              </div>
              <div className="p-2 rounded bg-[#0b111a] border border-white/8">
                <div className="text-[9px] text-[#64748b]">Success</div>
                <div className="text-sm font-bold text-white">99.2%</div>
                <div className="text-[9px] text-emerald-400">↑ 0.4%</div>
              </div>
              <div className="p-2 rounded bg-[#0b111a] border border-white/8">
                <div className="text-[9px] text-[#64748b]">Latency</div>
                <div className="text-sm font-bold text-white">1.3s</div>
                <div className="text-[9px] text-emerald-400">↓ 18%</div>
              </div>
            </div>
          </div>

          <div className="space-y-2 pt-2 border-t border-white/8">
            <div className="flex items-center justify-between text-[10px]">
              <span className="text-[#9ba5b4]">HARDWARE ACCEL (WebGPU)</span>
              <span className="text-emerald-400">● Active</span>
            </div>

            {[
              { label: 'Compute Shaders', pct: 82 },
              { label: 'Memory Bandwidth', pct: 76 },
              { label: 'VRAM Usage', pct: 58 },
            ].map((bar) => (
              <div key={bar.label} className="space-y-1 text-[10px]">
                <div className="flex justify-between text-[#cbd5e1]">
                  <span>{bar.label}</span>
                  <span>{bar.pct}%</span>
                </div>
                <div className="h-1.5 rounded-full bg-white/10 overflow-hidden">
                  <div
                    className="h-full bg-gradient-to-r from-cyan-400 to-blue-500 rounded-full"
                    style={{ width: `${bar.pct}%` }}
                  />
                </div>
              </div>
            ))}

            <div className="flex justify-between text-[10px] pt-1">
              <span className="text-[#9ba5b4]">Acceleration Gain</span>
              <span className="text-cyan-300 font-bold">3.8x</span>
            </div>
          </div>

          <div className="space-y-1.5 pt-2 border-t border-white/8 text-[10px]">
            <div className="text-[#9ba5b4]">SYSTEM LOGS</div>
            <div className="space-y-1 text-[#9ba5b4]">
              <div>
                <span className="text-[#64748b]">12:18:42</span>{' '}
                <span className="text-cyan-300">optimizer-09</span> Optimization (+18%)
              </div>
              <div>
                <span className="text-[#64748b]">12:18:37</span>{' '}
                <span className="text-emerald-300">executor-03</span> Tests passed (14/14)
              </div>
              <div>
                <span className="text-[#64748b]">12:18:32</span>{' '}
                <span className="text-purple-300">critic-06</span> Code review approved
              </div>
              <div>
                <span className="text-[#64748b]">12:18:25</span>{' '}
                <span className="text-blue-300">deployer-10</span> Staging ready
              </div>
            </div>
          </div>
        </aside>
      </div>
    </div>
  );
};
