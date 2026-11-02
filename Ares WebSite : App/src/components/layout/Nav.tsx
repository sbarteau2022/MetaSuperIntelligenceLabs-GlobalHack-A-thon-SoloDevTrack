import { Button } from '../ui/Button'
import { Pill } from '../ui/Pill'

const LINKS = ['Home', 'Mission', 'Apps', 'Harness', 'Tooling', 'Research', 'Contact']

export default function Nav() {
  return (
    <header className="fixed top-0 right-0 left-0 z-50 border-b border-glass-border bg-obsidian-900/70 backdrop-blur-xl">
      <div className="mx-auto flex h-16 max-w-[1200px] items-center justify-between px-6">
        <a href="#" className="flex items-center gap-2.5">
          <svg width="22" height="22" viewBox="0 0 22 22" aria-hidden="true">
            <circle cx="11" cy="11" r="10" fill="none" stroke="#22d3ee" strokeWidth="1.5" />
            <circle cx="11" cy="11" r="4" fill="#22d3ee" />
          </svg>
          <span className="font-display text-[15px] font-semibold text-fg-primary">
            AtomaLabs
          </span>
          <span className="font-mono text-[11px] text-fg-muted">ARES — SPECOPS</span>
        </a>
        <nav className="hidden gap-7 md:flex">
          {LINKS.map((link) => (
            <a
              key={link}
              href={`#${link.toLowerCase()}`}
              className="font-mono text-[12px] text-fg-secondary transition-colors hover:text-cyan-400"
            >
              {link}
            </a>
          ))}
        </nav>
        <div className="flex items-center gap-3">
          <Pill variant="verified">NOV 4</Pill>
          <Button size="sm" variant="primary">
            Download
          </Button>
        </div>
      </div>
    </header>
  )
}
