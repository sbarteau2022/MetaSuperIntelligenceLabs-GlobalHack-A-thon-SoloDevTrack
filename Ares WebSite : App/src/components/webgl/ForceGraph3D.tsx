import React, { useMemo, useRef, useState } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import * as THREE from 'three';
import { useAtomaStore } from '../../store/useAtomaStore';
import { Search, Sliders, Sparkles, Star, X, RotateCcw, Folder, Tag } from '../ui/Icons';

interface GraphCloudProps {
  scheme: 'Galaxy ✨' | 'Nebula ✨' | 'Obsidian Vault' | 'Neural Cortex';
  nodeSize: number;
  showLinks: boolean;
  linkOpacity: number;
  repulsion: number;
  pullToCenter: number;
  seed: number;
  autoRotate: boolean;
  dragRotation: React.MutableRefObject<{ x: number; y: number; zoom: number }>;
}

function PointCloudNetwork({
  scheme,
  nodeSize,
  showLinks,
  linkOpacity,
  repulsion,
  pullToCenter,
  seed,
  autoRotate,
  dragRotation,
}: GraphCloudProps) {
  const groupRef = useRef<THREE.Group>(null);

  const { pointGeo, lineGeo } = useMemo(() => {
    const count = 2400;
    const pos = new Float32Array(count * 3);
    const col = new Float32Array(count * 3);

    const palettes: Record<GraphCloudProps['scheme'], string[]> = {
      'Galaxy ✨': ['#ff6bd6', '#389cff', '#a855f7', '#38bdf8', '#fde047', '#ffffff'],
      'Nebula ✨': ['#06b6d4', '#ec4899', '#8b5cf6', '#eab308', '#3b82f6', '#10b981'],
      'Obsidian Vault': ['#d946ef', '#8b5cf6', '#38bdf8', '#facc15', '#34d399', '#fb7185'],
      'Neural Cortex': ['#00f0ff', '#389cff', '#0284c7', '#7dd3fc', '#e0f2fe', '#60a5fa'],
    };

    const palette = palettes[scheme].map((hex) => new THREE.Color(hex));
    const spreadScale = (repulsion / 9) * (1 / Math.max(0.12, pullToCenter * 3.5));

    let s = seed * 9973 + 12345;
    const rand = () => {
      s = (s * 16807) % 2147483647;
      return (s - 1) / 2147483646;
    };

    for (let i = 0; i < count; i++) {
      let x = 0,
        y = 0,
        z = 0;

      if (scheme === 'Nebula ✨') {
        const clusterIdx = i % 6;
        const centers = [
          [0.4, 0.9, 0.2],
          [-0.6, -0.7, 0.3],
          [1.6, -0.2, -0.5],
          [-1.5, 0.6, -0.3],
          [0.0, 0.0, 0.0],
          [-0.2, -1.4, -0.4],
        ];
        const [cx, cy, cz] = centers[clusterIdx];
        const r = Math.pow(rand(), 0.7) * 1.15;
        const theta = rand() * Math.PI * 2;
        const phi = Math.acos(2 * rand() - 1);
        x = (cx + r * Math.sin(phi) * Math.cos(theta)) * spreadScale;
        y = (cy + r * Math.sin(phi) * Math.sin(theta)) * spreadScale;
        z = (cz + r * Math.cos(phi)) * spreadScale;
      } else if (scheme === 'Neural Cortex') {
        const hemi = i % 2 === 0 ? -1 : 1;
        const u = rand() * Math.PI;
        const v = rand() * Math.PI * 2;
        const r = 1.65 + 0.25 * Math.sin(u * 6) * Math.cos(v * 5);
        x = hemi * (0.18 + Math.abs(Math.sin(u) * Math.cos(v)) * r * 0.85) * spreadScale;
        y = Math.cos(u) * r * 0.78 * spreadScale;
        z = Math.sin(u) * Math.sin(v) * r * 1.05 * spreadScale;
      } else {
        const r = Math.pow(rand(), 0.58) * 2.35 * spreadScale;
        const theta = rand() * Math.PI * 2;
        const phi = Math.acos(2 * rand() - 1);
        x = r * Math.sin(phi) * Math.cos(theta);
        y = r * Math.sin(phi) * Math.sin(theta);
        z = r * Math.cos(phi);
      }

      pos[i * 3] = x;
      pos[i * 3 + 1] = y;
      pos[i * 3 + 2] = z;

      const c = palette[i % palette.length];
      const coreBoost = Math.max(0, 1 - Math.sqrt(x * x + y * y + z * z) / 2.6);
      col[i * 3] = Math.min(1, c.r + coreBoost * 0.35);
      col[i * 3 + 1] = Math.min(1, c.g + coreBoost * 0.35);
      col[i * 3 + 2] = Math.min(1, c.b + coreBoost * 0.35);
    }

    const maxLinks = 1500;
    const lPos = new Float32Array(maxLinks * 6);
    const lCol = new Float32Array(maxLinks * 6);
    let linkIdx = 0;

    for (let i = 0; i < 650 && linkIdx < maxLinks; i++) {
      const target = (i * 7 + 3) % count;
      const ax = pos[i * 3],
        ay = pos[i * 3 + 1],
        az = pos[i * 3 + 2];
      const bx = pos[target * 3],
        by = pos[target * 3 + 1],
        bz = pos[target * 3 + 2];
      const dist = Math.hypot(ax - bx, ay - by, az - bz);
      if (dist < 2.2 * spreadScale) {
        lPos[linkIdx * 6] = ax;
        lPos[linkIdx * 6 + 1] = ay;
        lPos[linkIdx * 6 + 2] = az;
        lPos[linkIdx * 6 + 3] = bx;
        lPos[linkIdx * 6 + 4] = by;
        lPos[linkIdx * 6 + 5] = bz;

        lCol[linkIdx * 6] = col[i * 3];
        lCol[linkIdx * 6 + 1] = col[i * 3 + 1];
        lCol[linkIdx * 6 + 2] = col[i * 3 + 2];
        lCol[linkIdx * 6 + 3] = col[target * 3];
        lCol[linkIdx * 6 + 4] = col[target * 3 + 1];
        lCol[linkIdx * 6 + 5] = col[target * 3 + 2];
        linkIdx++;
      }
    }

    const pGeo = new THREE.BufferGeometry();
    pGeo.setAttribute('position', new THREE.BufferAttribute(pos, 3));
    pGeo.setAttribute('color', new THREE.BufferAttribute(col, 3));

    const lnGeo = new THREE.BufferGeometry();
    lnGeo.setAttribute('position', new THREE.BufferAttribute(lPos.slice(0, linkIdx * 6), 3));
    lnGeo.setAttribute('color', new THREE.BufferAttribute(lCol.slice(0, linkIdx * 6), 3));

    return { pointGeo: pGeo, lineGeo: lnGeo };
  }, [scheme, repulsion, pullToCenter, seed]);

  useFrame((state) => {
    const t = state.clock.getElapsedTime();
    if (groupRef.current) {
      groupRef.current.rotation.y = (autoRotate ? t * 0.08 : 0) + dragRotation.current.y;
      groupRef.current.rotation.x = Math.sin(t * 0.15) * 0.08 + dragRotation.current.x;
      groupRef.current.scale.setScalar(dragRotation.current.zoom);
    }
  });

  return (
    <group ref={groupRef}>
      {showLinks && (
        <lineSegments geometry={lineGeo}>
          <lineBasicMaterial
            vertexColors
            transparent
            opacity={linkOpacity}
            blending={THREE.AdditiveBlending}
          />
        </lineSegments>
      )}

      <points geometry={pointGeo}>
        <pointsMaterial
          size={nodeSize * 0.095}
          vertexColors
          transparent
          opacity={0.92}
          sizeAttenuation
          blending={THREE.AdditiveBlending}
        />
      </points>
    </group>
  );
}

export const ForceGraph3D: React.FC = () => {
  const {
    graphColorScheme,
    setGraphColorScheme,
    graphSizeMode,
    setGraphSizeMode,
    graphNodeSize,
    setGraphNodeSize,
    graphShowLinks,
    setGraphShowLinks,
    graphLinkThickness,
    setGraphLinkThickness,
    graphLinkOpacity,
    setGraphLinkOpacity,
    graphRepulsion,
    setGraphRepulsion,
    graphLinkLength,
    setGraphLinkLength,
    graphPullToCenter,
    setGraphPullToCenter,
    graphCloudSeed,
    reformGraphCloud,
  } = useAtomaStore();

  const [searchFilter, setSearchFilter] = useState('word, path: tag: content: opened:>10 -exc');
  const [glowMode, setGlowMode] = useState<'All links' | 'Outbound links'>('All links');
  const [smoothness, setSmoothness] = useState(0.6);
  const [elasticity, setElasticity] = useState(0.45);
  const [linkStrength, setLinkStrength] = useState(0.9);
  const [freeLayout, setFreeLayout] = useState(false);

  const dragRotation = useRef({ x: 0, y: 0, zoom: 1 });
  const isDragging = useRef(false);
  const lastPos = useRef({ x: 0, y: 0 });

  const isLightVault = graphColorScheme === 'Obsidian Vault';

  return (
    <div
      className={`relative w-full h-[680px] rounded-xl overflow-hidden border border-white/10 select-none ${
        isLightVault ? 'bg-[#ebe4d8] text-[#1c1917]' : 'bg-[#040509] text-[#f5f7fa]'
      }`}
    >
      {/* Top Floating Filter & Search Pill */}
      <div className="absolute top-3 left-1/2 -translate-x-1/2 z-20 flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-[#14161c]/95 border border-white/12 text-xs shadow-2xl backdrop-blur-md">
        <button className="px-2 py-1 rounded bg-white/8 hover:bg-white/14 text-[#e2e8f0] font-medium flex items-center gap-1">
          <Tag className="w-3 h-3 text-[#389cff]" /> Tags
        </button>
        <button className="px-2 py-1 rounded bg-white/8 hover:bg-white/14 text-[#e2e8f0] font-medium flex items-center gap-1">
          <Folder className="w-3 h-3 text-[#a855f7]" /> Folders
        </button>
        <div className="flex items-center gap-1.5 px-2.5 py-1 rounded bg-black/50 border border-white/8 w-72">
          <Search className="w-3 h-3 text-[#9ba5b4]" />
          <input
            type="text"
            value={searchFilter}
            onChange={(e) => setSearchFilter(e.target.value)}
            className="bg-transparent text-[11px] text-[#cbd5e1] focus:outline-none w-full font-mono"
          />
        </div>
        <div className="px-2.5 py-1 rounded bg-black/40 border border-white/8 text-[11px] text-[#9ba5b4] w-44 flex items-center justify-between">
          <span>Filters...</span>
          <span className="text-[10px]">⌄</span>
        </div>
        <button
          onClick={reformGraphCloud}
          title="Star current preset"
          className="p-1 rounded hover:bg-white/10 text-[#9ba5b4] hover:text-amber-400"
        >
          <Star className="w-3.5 h-3.5" />
        </button>
        <button
          onClick={() => setSearchFilter('')}
          title="Clear filter"
          className="p-1 rounded hover:bg-white/10 text-[#9ba5b4] hover:text-white"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* Left Advanced Graph View Inspector */}
      <div className="absolute top-3 left-3 bottom-3 w-64 z-20 rounded-lg bg-[#121318]/95 border border-white/10 backdrop-blur-xl flex flex-col overflow-hidden text-[#d4d8e2] shadow-2xl">
        <div className="flex items-center gap-2 px-3 py-2 border-b border-white/8 bg-[#181a20]">
          <span className="px-1.5 py-0.5 rounded bg-[#389cff] text-white text-[10px] font-semibold">
            N³
          </span>
          <Sliders className="w-3.5 h-3.5 text-[#9ba5b4]" />
          <Sparkles className="w-3.5 h-3.5 text-[#389cff]" />
          <span className="text-[11px] font-semibold ml-auto text-[#9ba5b4]">3D Vault</span>
        </div>

        <div className="px-3 py-2 border-b border-white/8 flex items-center justify-between">
          <span className="text-xs font-semibold text-white">Advanced Graph View</span>
          <span className="text-xs text-[#9ba5b4]">−</span>
        </div>

        <div className="flex-1 overflow-y-auto p-3 space-y-4 text-[11px]">
          <div className="space-y-2">
            <div className="text-[10px] font-mono uppercase tracking-wider text-[#8e96a8]">
              ▾ VIEW PRESETS
            </div>
            <div className="grid grid-cols-2 gap-1.5">
              {(['Galaxy ✨', 'Nebula ✨', 'Obsidian Vault', 'Neural Cortex'] as const).map((preset) => (
                <button
                  key={preset}
                  onClick={() => setGraphColorScheme(preset)}
                  className={`px-2 py-1.5 rounded text-[11px] font-medium border text-left transition ${
                    graphColorScheme === preset
                      ? 'bg-[#389cff]/20 border-[#389cff] text-white'
                      : 'bg-white/5 border-white/8 text-[#9ba5b4] hover:text-white'
                  }`}
                >
                  {preset}
                </button>
              ))}
            </div>
            <div className="grid grid-cols-2 gap-1.5 pt-0.5">
              <button
                onClick={reformGraphCloud}
                className="py-1 px-2 rounded bg-white/8 hover:bg-white/14 text-white font-medium text-center"
              >
                Save current
              </button>
              <button
                onClick={reformGraphCloud}
                className="py-1 px-2 rounded bg-white/5 hover:bg-white/10 text-[#9ba5b4] text-center"
              >
                Reset
              </button>
            </div>
          </div>

          <div className="space-y-2 pt-2 border-t border-white/8">
            <div className="text-[10px] font-mono uppercase tracking-wider text-[#8e96a8]">
              ▾ APPEARANCE
            </div>

            <div className="flex items-center justify-between">
              <span className="text-[#9ba5b4]">Size</span>
              <select
                value={graphSizeMode}
                onChange={(e) => setGraphSizeMode(e.target.value as any)}
                className="bg-[#1e2028] border border-white/10 rounded px-2 py-0.5 text-[11px] text-white"
              >
                <option value="PageRank">PageRank</option>
                <option value="Inbound links">Inbound links</option>
                <option value="Betweenness">Betweenness</option>
              </select>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-[#9ba5b4]">Color</span>
              <span className="bg-[#1e2028] border border-white/10 rounded px-2.5 py-0.5 text-[11px] text-white">
                Cluster ⌄
              </span>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-[#9ba5b4]">Glow</span>
              <select
                value={glowMode}
                onChange={(e) => setGlowMode(e.target.value as any)}
                className="bg-[#1e2028] border border-white/10 rounded px-2 py-0.5 text-[11px] text-white"
              >
                <option value="All links">All links</option>
                <option value="Outbound links">Outbound links</option>
              </select>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-[#9ba5b4]">Color scheme</span>
              <select
                value={graphColorScheme}
                onChange={(e) => setGraphColorScheme(e.target.value as any)}
                className="bg-[#1e2028] border border-white/10 rounded px-2 py-0.5 text-[11px] text-white"
              >
                <option value="Galaxy ✨">Galaxy ✨</option>
                <option value="Nebula ✨">Nebula ✨</option>
                <option value="Obsidian Vault">Obsidian Vault</option>
                <option value="Neural Cortex">Neural Cortex</option>
              </select>
            </div>

            <div className="space-y-1 pt-1">
              <div className="flex justify-between">
                <span className="text-[#9ba5b4]">Node size</span>
                <span className="font-mono text-white">{graphNodeSize.toFixed(3)}</span>
              </div>
              <input
                type="range"
                min={0.15}
                max={1.2}
                step={0.05}
                value={graphNodeSize}
                onChange={(e) => setGraphNodeSize(parseFloat(e.target.value))}
                className="w-full accent-[#389cff]"
              />
            </div>
          </div>

          <div className="space-y-2 pt-2 border-t border-white/8">
            <div className="text-[10px] font-mono uppercase tracking-wider text-[#8e96a8]">
              ▾ LINKS
            </div>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={graphShowLinks}
                onChange={(e) => setGraphShowLinks(e.target.checked)}
                className="accent-[#389cff] rounded"
              />
              <span>Show links</span>
            </label>

            <div className="space-y-1">
              <div className="flex justify-between">
                <span className="text-[#9ba5b4]">Thickness</span>
                <span className="font-mono text-white">{graphLinkThickness.toFixed(3)}</span>
              </div>
              <input
                type="range"
                min={0.1}
                max={2.0}
                step={0.1}
                value={graphLinkThickness}
                onChange={(e) => setGraphLinkThickness(parseFloat(e.target.value))}
                className="w-full accent-[#389cff]"
              />
            </div>

            <div className="space-y-1">
              <div className="flex justify-between">
                <span className="text-[#9ba5b4]">Opacity</span>
                <span className="font-mono text-white">{graphLinkOpacity.toFixed(3)}</span>
              </div>
              <input
                type="range"
                min={0.02}
                max={0.6}
                step={0.02}
                value={graphLinkOpacity}
                onChange={(e) => setGraphLinkOpacity(parseFloat(e.target.value))}
                className="w-full accent-[#389cff]"
              />
            </div>
          </div>

          <div className="space-y-2 pt-2 border-t border-white/8">
            <div className="text-[10px] font-mono uppercase tracking-wider text-[#8e96a8]">
              ▾ PHYSICS
            </div>

            <div className="space-y-1">
              <div className="flex justify-between">
                <span className="text-[#9ba5b4]">Node spread (repulsion)</span>
                <span className="font-mono text-white">{graphRepulsion}</span>
              </div>
              <input
                type="range"
                min={3}
                max={18}
                step={1}
                value={graphRepulsion}
                onChange={(e) => setGraphRepulsion(parseInt(e.target.value, 10))}
                className="w-full accent-[#389cff]"
              />
            </div>

            <div className="space-y-1">
              <div className="flex justify-between">
                <span className="text-[#9ba5b4]">Link length</span>
                <span className="font-mono text-white">{graphLinkLength}</span>
              </div>
              <input
                type="range"
                min={80}
                max={320}
                step={5}
                value={graphLinkLength}
                onChange={(e) => setGraphLinkLength(parseInt(e.target.value, 10))}
                className="w-full accent-[#389cff]"
              />
            </div>

            <div className="space-y-1">
              <div className="flex justify-between">
                <span className="text-[#9ba5b4]">Pull to center</span>
                <span className="font-mono text-white">{graphPullToCenter.toFixed(3)}</span>
              </div>
              <input
                type="range"
                min={0.12}
                max={0.5}
                step={0.005}
                value={graphPullToCenter}
                onChange={(e) => setGraphPullToCenter(parseFloat(e.target.value))}
                className="w-full accent-[#389cff]"
              />
            </div>

            <div className="space-y-1">
              <div className="flex justify-between">
                <span className="text-[#9ba5b4]">Link strength</span>
                <span className="font-mono text-white">{linkStrength.toFixed(1)}</span>
              </div>
              <input
                type="range"
                min={0.1}
                max={1.5}
                step={0.1}
                value={linkStrength}
                onChange={(e) => setLinkStrength(parseFloat(e.target.value))}
                className="w-full accent-[#389cff]"
              />
            </div>

            <div className="space-y-1">
              <div className="flex justify-between">
                <span className="text-[#9ba5b4]">Smoothness</span>
                <span className="font-mono text-white">{smoothness.toFixed(1)}</span>
              </div>
              <input
                type="range"
                min={0.1}
                max={1.0}
                step={0.1}
                value={smoothness}
                onChange={(e) => setSmoothness(parseFloat(e.target.value))}
                className="w-full accent-[#389cff]"
              />
            </div>

            <div className="space-y-1">
              <div className="flex justify-between">
                <span className="text-[#9ba5b4]">Link elasticity</span>
                <span className="font-mono text-white">{elasticity.toFixed(2)}</span>
              </div>
              <input
                type="range"
                min={0.1}
                max={1.0}
                step={0.05}
                value={elasticity}
                onChange={(e) => setElasticity(parseFloat(e.target.value))}
                className="w-full accent-[#389cff]"
              />
            </div>

            <label className="flex items-center gap-2 pt-1 cursor-pointer">
              <input
                type="checkbox"
                checked={freeLayout}
                onChange={(e) => setFreeLayout(e.target.checked)}
                className="accent-[#389cff]"
              />
              <span>Free layout</span>
            </label>

            <button
              onClick={reformGraphCloud}
              className="w-full mt-2 py-2 px-3 rounded bg-white/10 hover:bg-[#389cff] hover:text-white text-white font-medium flex items-center justify-center gap-1.5 transition"
            >
              <RotateCcw className="w-3.5 h-3.5" /> Re-form the cloud
            </button>
          </div>
        </div>
      </div>

      {/* Interactive 3D WebGL Viewport */}
      <div
        className="w-full h-full cursor-grab active:cursor-grabbing"
        onPointerDown={(e) => {
          isDragging.current = true;
          lastPos.current = { x: e.clientX, y: e.clientY };
        }}
        onPointerMove={(e) => {
          if (!isDragging.current) return;
          const dx = e.clientX - lastPos.current.x;
          const dy = e.clientY - lastPos.current.y;
          dragRotation.current.y += dx * 0.006;
          dragRotation.current.x = Math.max(-1.2, Math.min(1.2, dragRotation.current.x + dy * 0.006));
          lastPos.current = { x: e.clientX, y: e.clientY };
        }}
        onPointerUp={() => {
          isDragging.current = false;
        }}
        onPointerLeave={() => {
          isDragging.current = false;
        }}
        onWheel={(e) => {
          dragRotation.current.zoom = Math.max(
            0.55,
            Math.min(1.8, dragRotation.current.zoom - e.deltaY * 0.0008)
          );
        }}
      >
        <Canvas
          camera={{ position: [0, 0, 5.2], fov: 50 }}
          dpr={[1, 1.5]}
          gl={{ antialias: false, alpha: true, powerPreference: 'high-performance' }}
        >
          <ambientLight intensity={0.8} />
          <PointCloudNetwork
            scheme={graphColorScheme}
            nodeSize={graphNodeSize}
            showLinks={graphShowLinks}
            linkOpacity={graphLinkOpacity}
            repulsion={graphRepulsion}
            pullToCenter={graphPullToCenter}
            seed={graphCloudSeed}
            autoRotate={!freeLayout}
            dragRotation={dragRotation}
          />
        </Canvas>
      </div>

      <div className="absolute bottom-3 right-4 z-10 flex items-center gap-4 px-3 py-1.5 rounded-full bg-black/70 border border-white/10 font-mono text-[11px] text-[#9ba5b4]">
        <span>2,400 N³ NODES</span>
        <span>·</span>
        <span className="text-[#389cff]">{graphColorScheme.toUpperCase()}</span>
        <span>·</span>
        <span>DRAG TO ORBIT / SCROLL TO ZOOM</span>
      </div>
    </div>
  );
};
