import { ChatPanel } from '../components/ChatPanel'

export function Screen01Home() {
  return (
    <div className="screen-01" style={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
      {/* Hero Section */}
      <section style={{
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '40px',
        textAlign: 'center',
      }}>
        <div style={{ maxWidth: '800px' }}>
          <div style={{
            width: '80px',
            height: '80px',
            margin: '0 auto 24px',
            display: 'grid',
            placeItems: 'center',
            border: '1px solid var(--border-subtle)',
            borderRadius: '16px',
            background: 'linear-gradient(135deg, var(--accent-blue-glow), transparent)',
            fontSize: '40px',
            fontWeight: 800,
            color: 'var(--accent-blue)',
          }}>
            A
          </div>
          <h1 style={{
            margin: '0 0 16px',
            fontSize: 'clamp(32px, 5vw, 48px)',
            lineHeight: 1.1,
            letterSpacing: '-0.03em',
            fontWeight: 700,
          }}>
            Atoma Framer
          </h1>
          <p style={{
            margin: '0 0 32px',
            fontSize: '18px',
            lineHeight: 1.6,
            color: 'var(--fg-secondary)',
            maxWidth: '600px',
            marginLeft: 'auto',
            marginRight: 'auto',
          }}>
            A desktop application for agentic engineering — Framer-skinned, Tauri-powered,
            with the full Model Home harness embedded. Multi-turn agent sessions with
            Postgres checkpointing, 47 tools, and MCP integration.
          </p>
          <div style={{
            display: 'flex',
            gap: '12px',
            justifyContent: 'center',
            flexWrap: 'wrap',
          }}>
            <a href="#" style={{
              padding: '12px 24px',
              borderRadius: 'var(--radius-md)',
              background: 'var(--accent-blue)',
              color: 'white',
              fontWeight: 500,
              textDecoration: 'none',
              transition: 'var(--transition-fast)',
            }}>
              Start Agent Session
            </a>
            <a href="#" style={{
              padding: '12px 24px',
              borderRadius: 'var(--radius-md)',
              background: 'transparent',
              border: '1px solid var(--border-subtle)',
              color: 'var(--fg-primary)',
              fontWeight: 500,
              textDecoration: 'none',
              transition: 'var(--transition-fast)',
            }}>
              View Screens
            </a>
          </div>
        </div>
      </section>

      {/* Agent Chat Panel */}
      <section style={{
        height: '45vh',
        minHeight: '350px',
        borderTop: '1px solid var(--border-subtle)',
        background: 'var(--bg-secondary)',
      }}>
        <ChatPanel sessionId="home-session" />
      </section>
    </div>
  )
}
