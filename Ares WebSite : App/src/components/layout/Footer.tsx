const LINKS = ['GitHub', 'Discord', 'License', 'Privacy']

export default function Footer() {
  return (
    <footer className="border-t border-glass-border py-12" data-reveal>
      <div className="mx-auto flex max-w-[1200px] flex-col gap-4 px-6 sm:flex-row sm:items-center sm:justify-between">
        <p className="font-mono text-[12px] text-fg-muted">
          AtomaLabs © 2026 — research-grade engineering lab.
        </p>
        <nav className="flex gap-6">
          {LINKS.map((link) => (
            <a
              key={link}
              href="#"
              className="font-mono text-[12px] text-fg-muted transition-colors hover:text-cyan-400"
            >
              {link}
            </a>
          ))}
        </nav>
      </div>
    </footer>
  )
}
