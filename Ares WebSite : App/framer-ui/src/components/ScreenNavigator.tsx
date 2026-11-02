import { useState, useCallback } from 'react'
import { ReactNode } from 'react'

interface ScreenMeta {
  id: string
  label: string
  icon: string
}

const SCREEN_META: ScreenMeta[] = [
  { id: 'home', label: 'Home', icon: '🏠' },
  { id: 'tools-mcp', label: 'Tools & MCP', icon: '🔧' },
  { id: 'meta-hackathon', label: 'Meta Hackathon', icon: '🏆' },
  { id: 'research-lab', label: 'Research Lab', icon: '🧪' },
  { id: 'contact', label: 'Contact', icon: '📧' },
  { id: 'sign-up', label: 'Sign Up', icon: '📝' },
  { id: 'downloads', label: 'Downloads', icon: '⬇️' },
  { id: 'ide-chat', label: 'IDE + Chat', icon: '💻' },
  { id: 'swarm-holonomy', label: 'Swarm Holonomy', icon: '🔮' },
  { id: 'mind-context-graph', label: 'Mind Context Graph', icon: '🧠' },
  { id: 'research-hub-reader', label: 'Research Hub Reader', icon: '📚' },
]

interface ScreenNavigatorProps {
  screens: ReactNode[]
}

export function ScreenNavigator({ screens }: ScreenNavigatorProps) {
  const [currentIndex, setCurrentIndex] = useState(0)

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'ArrowLeft') {
      setCurrentIndex(i => Math.max(0, i - 1))
    } else if (e.key === 'ArrowRight') {
      setCurrentIndex(i => Math.min(SCREEN_META.length - 1, i + 1))
    }
  }, [])

  return (
    <div
      className="screen-navigator"
      style={{
        display: 'flex',
        height: '100vh',
        background: 'var(--bg-primary)',
      }}
      onKeyDown={handleKeyDown}
      tabIndex={0}
    >
      {/* Sidebar */}
      <aside
        style={{
          width: 'var(--sidebar-width)',
          background: 'var(--bg-secondary)',
          borderRight: '1px solid var(--border-subtle)',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        <div style={{ padding: '16px', borderBottom: '1px solid var(--border-subtle)' }}>
          <h2 style={{
            margin: 0,
            fontSize: '13px',
            fontWeight: 600,
            textTransform: 'uppercase',
            letterSpacing: '0.5px',
            color: 'var(--fg-muted)',
          }}>
            Screens
          </h2>
        </div>
        <nav style={{ flex: 1, overflowY: 'auto', padding: '8px' }}>
          {SCREEN_META.map((screen, i) => (
            <button
              key={screen.id}
              onClick={() => setCurrentIndex(i)}
              style={{
                width: '100%',
                display: 'flex',
                alignItems: 'center',
                gap: '10px',
                padding: '10px 12px',
                borderRadius: 'var(--radius-sm)',
                background: i === currentIndex ? 'var(--accent-blue-glow)' : 'transparent',
                border: 'none',
                color: i === currentIndex ? 'var(--accent-blue)' : 'var(--fg-primary)',
                fontSize: '13px',
                fontWeight: i === currentIndex ? 600 : 400,
                cursor: 'pointer',
                textAlign: 'left',
                transition: 'var(--transition-fast)',
              }}
              onMouseEnter={e => {
                if (i !== currentIndex) {
                  e.currentTarget.style.background = 'var(--bg-tertiary)'
                }
              }}
              onMouseLeave={e => {
                if (i !== currentIndex) {
                  e.currentTarget.style.background = 'transparent'
                }
              }}
            >
              <span style={{ fontSize: '16px' }}>{screen.icon}</span>
              {screen.label}
            </button>
          ))}
        </nav>
        <div style={{ padding: '12px', borderTop: '1px solid var(--border-subtle)', fontSize: '11px', color: 'var(--fg-muted)' }}>
          Navigate with <kbd style={{
            background: 'var(--bg-primary)',
            border: '1px solid var(--border-subtle)',
            borderRadius: '3px',
            padding: '1px 4px',
            margin: '0 2px',
            fontFamily: 'var(--font-mono)',
            fontSize: '10px',
          }}>←</kbd> <kbd style={{
            background: 'var(--bg-primary)',
            border: '1px solid var(--border-subtle)',
            borderRadius: '3px',
            padding: '1px 4px',
            margin: '0 2px',
            fontFamily: 'var(--font-mono)',
            fontSize: '10px',
          }}>→</kbd> or click a screen
        </div>
      </aside>

      {/* Main Content */}
      <main style={{ flex: 1, overflow: 'auto', position: 'relative' }}>
        {screens[currentIndex]}
      </main>
    </div>
  )
}
