import { Button } from '../ui/Button'
import { Pill } from '../ui/Pill'
import { AtomaBrandLockup } from '../brand/AtomaBrandAssets'

const LINKS = ['Home', 'Mission', 'Apps', 'Harness', 'Tooling', 'Research', 'Contact']

export default function Nav() {
  return (
    <header className="fixed top-0 right-0 left-0 z-50 border-b border-glass-border bg-[#2e2e33]/90 backdrop-blur-xl">
      <div className="mx-auto flex h-16 max-w-[1200px] items-center justify-between px-6">
        <AtomaBrandLockup subtitle="ARES — SPECOPS" />
        <nav className="hidden gap-7 md:flex">
          {LINKS.map((link) => (
            <a
              key={link}
              href={`#${link.toLowerCase()}`}
              className="font-mono text-[12px] text-fg-secondary transition-colors hover:text-white"
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
