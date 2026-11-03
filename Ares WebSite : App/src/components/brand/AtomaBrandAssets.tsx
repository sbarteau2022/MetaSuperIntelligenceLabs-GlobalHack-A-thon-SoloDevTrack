import React from 'react';

/**
 * Signature Atoma Geodesic Truss + Interlocking Gyroscopic Rings Logo Mark
 * Inspired directly by QGh3P.jpg: machined octahedral/triangular truss core
 * encircled by three interlocking brushed-titanium gyroscopic rings with a amber rim arc.
 */
export const AtomaLogoMark: React.FC<{ className?: string; glow?: boolean }> = ({
  className = 'w-8 h-8',
  glow = true,
}) => (
  <svg
    viewBox="0 0 48 48"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    className={`transition-transform duration-300 ${className}`}
    aria-hidden="true"
  >
    <defs>
      <linearGradient id="atoma-gyro-silver" x1="6" y1="4" x2="42" y2="44" gradientUnits="userSpaceOnUse">
        <stop offset="0%" stopColor="#ffffff" />
        <stop offset="48%" stopColor="#a1a1aa" />
        <stop offset="100%" stopColor="#52525b" />
      </linearGradient>
      <linearGradient id="atoma-gyro-amber" x1="20" y1="4" x2="44" y2="26" gradientUnits="userSpaceOnUse">
        <stop offset="0%" stopColor="#fbbf24" />
        <stop offset="50%" stopColor="#f59e0b" />
        <stop offset="100%" stopColor="#71717a" />
      </linearGradient>
    </defs>

    {/* 3 Interlocking Gyroscopic Orbital Rings */}
    <ellipse
      cx="24"
      cy="24"
      rx="20"
      ry="9.5"
      transform="rotate(-34 24 24)"
      stroke="url(#atoma-gyro-amber)"
      strokeWidth="2.4"
    />
    <ellipse
      cx="24"
      cy="24"
      rx="20"
      ry="9.5"
      transform="rotate(34 24 24)"
      stroke="url(#atoma-gyro-silver)"
      strokeWidth="2.2"
    />
    <ellipse
      cx="24"
      cy="26"
      rx="18.5"
      ry="7.5"
      stroke="#71717a"
      strokeWidth="1.8"
      strokeOpacity="0.85"
    />

    {/* Central Geodesic Triangular Truss Core ('A' Apex) */}
    <polygon
      points="24,8 12,31 36,31"
      fill="#26262b"
      stroke="url(#atoma-gyro-silver)"
      strokeWidth="2.2"
      strokeLinejoin="round"
    />
    {/* Inverted inner triangle forming the octahedral truss */}
    <polygon
      points="17.5,20 30.5,20 24,33"
      fill="#34343a"
      stroke="#e4e4e7"
      strokeWidth="1.6"
      strokeLinejoin="round"
    />

    {/* Glowing Amber Rim Highlight & Core */}
    {glow && (
      <circle
        cx="24"
        cy="23.5"
        r="2.6"
        fill="#f59e0b"
        style={{ filter: 'drop-shadow(0 0 6px rgba(245, 158, 11, 0.95))' }}
      />
    )}
  </svg>
);

export const AtomaAgentBadge: React.FC<{ className?: string }> = ({ className = 'w-6 h-6' }) => (
  <span
    className={`inline-flex items-center justify-center rounded-full bg-[#2e2e33] border border-[#f59e0b]/40 shadow-[0_0_12px_rgba(245,158,11,0.35)] ${className}`}
  >
    <AtomaLogoMark className="w-4 h-4" glow={false} />
  </span>
);

export const AtomaBrandLockup: React.FC<{
  onClick?: () => void;
  subtitle?: string;
}> = ({ onClick, subtitle = 'ENGINEERING' }) => (
  <button
    onClick={onClick}
    className="inline-flex items-center gap-3 text-left group focus:outline-none cursor-pointer"
  >
    <AtomaLogoMark className="w-9 h-9 transition-transform duration-300 group-hover:rotate-12 group-hover:scale-105" />
    <div className="flex flex-col leading-none">
      <span className="font-display font-bold text-[15px] tracking-[0.22em] text-white">
        ATOMA
      </span>
      <span className="font-mono text-[9px] tracking-[0.28em] text-[#71717a] mt-1">
        {subtitle}
      </span>
    </div>
  </button>
);

export const BranchingWorkflowConceptSVG: React.FC<{
  activeStep: string;
  onSelectStep: (step: 'scope' | 'plan' | 'execute' | 'test' | 'review' | 'receipt') => void;
}> = ({ activeStep, onSelectStep }) => {
  const steps = [
    {
      id: 'scope' as const,
      title: 'Objective',
      desc: 'Define goals,\ncontext and\nsuccess criteria.',
      x: 40,
      y: 28,
    },
    {
      id: 'plan' as const,
      title: 'Plan',
      desc: 'Break it down\ninto a testable\nrunbook.',
      x: 138,
      y: 102,
    },
    {
      id: 'execute' as const,
      title: 'Build',
      desc: 'Execute with\nthe right tools\nand context.',
      x: 236,
      y: 176,
    },
    {
      id: 'review' as const,
      title: 'Review',
      desc: 'Assess results\nwith human\njudgment.',
      x: 334,
      y: 250,
    },
    {
      id: 'receipt' as const,
      title: 'Receipt',
      desc: 'Keep evidence\nconnected\nto the work.',
      x: 432,
      y: 314,
    },
  ];

  return (
    <div className="relative w-full rounded-2xl bg-[#06080c] border border-white/10 p-6 overflow-hidden linear-grid-bg">
      <div className="flex items-center justify-between text-[11px] font-mono text-[#9ba5b4] mb-2">
        <span className="uppercase tracking-widest text-[#c9cfd7]">ARES · WORKFLOW TOPOLOGY</span>
        <span>Workflow concept</span>
      </div>

      <svg viewBox="0 0 580 400" className="w-full h-auto overflow-visible select-none">
        <defs>
          <linearGradient id="ares-flow-line" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stopColor="#f59e0b" stopOpacity="0.95" />
            <stop offset="50%" stopColor="#60a5fa" stopOpacity="0.8" />
            <stop offset="100%" stopColor="#f59e0b" stopOpacity="0.95" />
          </linearGradient>
          <radialGradient id="node-blue-glow" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="#f59e0b" stopOpacity="0.35" />
            <stop offset="100%" stopColor="#f59e0b" stopOpacity="0" />
          </radialGradient>
        </defs>

        <circle
          cx="390"
          cy="190"
          r="155"
          fill="none"
          stroke="rgba(201,207,215,0.08)"
          strokeDasharray="3 3"
        />
        <circle cx="390" cy="190" r="105" fill="none" stroke="rgba(56,156,255,0.1)" />

        <path
          d="M 68 56 L 68 128 Q 68 140 80 140 L 138 140 M 166 156 L 166 202 Q 166 214 178 214 L 236 214 M 264 230 L 264 276 Q 264 288 276 288 L 334 288 M 362 304 L 362 340 Q 362 352 374 352 L 432 352"
          fill="none"
          stroke="url(#ares-flow-line)"
          strokeWidth="2.2"
        />

        {steps.map((s, idx) => {
          const isSelected = activeStep === s.id;
          return (
            <g key={s.id} onClick={() => onSelectStep(s.id)} className="cursor-pointer group">
              {isSelected && (
                <circle cx={s.x + 28} cy={s.y + 28} r="44" fill="url(#node-blue-glow)" />
              )}
              <rect
                x={s.x}
                y={s.y}
                width="56"
                height="56"
                rx="14"
                fill="#0c1017"
                stroke={isSelected ? '#f59e0b' : 'rgba(201,207,215,0.28)'}
                strokeWidth={isSelected ? '2' : '1.2'}
              />
              {idx === 0 && (
                <circle cx={s.x + 28} cy={s.y + 28} r="12" fill="#c9cfd7" fillOpacity="0.85" />
              )}
              {idx === 1 && (
                <path
                  d={`M ${s.x + 18} ${s.y + 24} L ${s.x + 28} ${s.y + 18} L ${s.x + 38} ${
                    s.y + 24
                  } L ${s.x + 28} ${s.y + 30} Z M ${s.x + 18} ${s.y + 31} L ${s.x + 28} ${
                    s.y + 37
                  } L ${s.x + 38} ${s.y + 31}`}
                  stroke="#c9cfd7"
                  strokeWidth="1.6"
                  fill="none"
                />
              )}
              {idx === 2 && (
                <rect
                  x={s.x + 19}
                  y={s.y + 19}
                  width="18"
                  height="18"
                  rx="3"
                  stroke="#f59e0b"
                  strokeWidth="1.8"
                  fill="rgba(56,156,255,0.15)"
                />
              )}
              {idx === 3 && (
                <circle
                  cx={s.x + 26}
                  cy={s.y + 26}
                  r="8"
                  stroke="#c9cfd7"
                  strokeWidth="1.8"
                  fill="none"
                />
              )}
              {idx === 4 && (
                <rect
                  x={s.x + 20}
                  y={s.y + 17}
                  width="16"
                  height="22"
                  rx="2"
                  stroke="#f59e0b"
                  strokeWidth="1.8"
                  fill="rgba(56,156,255,0.15)"
                />
              )}

              <text
                x={s.x + 70}
                y={s.y + 16}
                fill="#f5f7fa"
                fontSize="13"
                fontWeight="600"
                fontFamily="Space Grotesk, sans-serif"
              >
                {s.title}
              </text>
              {s.desc.split('\n').map((line, lIdx) => (
                <text
                  key={lIdx}
                  x={s.x + 70}
                  y={s.y + 33 + lIdx * 14}
                  fill="#9ba5b4"
                  fontSize="11"
                  fontFamily="JetBrains Mono, monospace"
                >
                  {line}
                </text>
              ))}
            </g>
          );
        })}
      </svg>
    </div>
  );
};
