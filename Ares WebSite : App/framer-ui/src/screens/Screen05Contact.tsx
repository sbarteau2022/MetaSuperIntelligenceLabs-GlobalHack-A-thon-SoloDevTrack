export function Screen05Contact() {
  return (
    <div className="screen-05" style={{ padding: '40px', maxWidth: '800px', margin: '0 auto', height: '100vh', overflow: 'auto', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', textAlign: 'center' }}>
      <div style={{ width: '100%' }}>
        <h1 style={{ fontSize: '32px', fontWeight: 700, marginBottom: '8px' }}>Contact</h1>
        <p style={{ color: 'var(--fg-muted)', fontSize: '16px', marginBottom: '40px' }}>
          Get in touch with the Atoma Engineering team
        </p>

        <form style={{ display: 'flex', flexDirection: 'column', gap: '20px', maxWidth: '500px', margin: '0 auto' }}>
          <div style={{ textAlign: 'left' }}>
            <label style={{ display: 'block', fontSize: '13px', fontWeight: 500, marginBottom: '8px', color: 'var(--fg-secondary)' }}>Name</label>
            <input type="text" placeholder="Your name" style={{ width: '100%', padding: '12px 16px', borderRadius: 'var(--radius-md)', background: 'var(--bg-primary)', border: '1px solid var(--border-subtle)', color: 'var(--fg-primary)', fontSize: '14px' }} />
          </div>
          <div style={{ textAlign: 'left' }}>
            <label style={{ display: 'block', fontSize: '13px', fontWeight: 500, marginBottom: '8px', color: 'var(--fg-secondary)' }}>Email</label>
            <input type="email" placeholder="you@example.com" style={{ width: '100%', padding: '12px 16px', borderRadius: 'var(--radius-md)', background: 'var(--bg-primary)', border: '1px solid var(--border-subtle)', color: 'var(--fg-primary)', fontSize: '14px' }} />
          </div>
          <div style={{ textAlign: 'left' }}>
            <label style={{ display: 'block', fontSize: '13px', fontWeight: 500, marginBottom: '8px', color: 'var(--fg-secondary)' }}>Message</label>
            <textarea placeholder="Your message..." rows={6} style={{ width: '100%', padding: '12px 16px', borderRadius: 'var(--radius-md)', background: 'var(--bg-primary)', border: '1px solid var(--border-subtle)', color: 'var(--fg-primary)', fontSize: '14px', resize: 'vertical', fontFamily: 'inherit' }} />
          </div>
          <button type="submit" style={{ padding: '14px 28px', borderRadius: 'var(--radius-md)', background: 'var(--accent-blue)', color: 'white', border: 'none', fontSize: '14px', fontWeight: 500, cursor: 'pointer', alignSelf: 'flex-start' }}>
            Send Message
          </button>
        </form>

        <div style={{ marginTop: '48px', paddingTop: '24px', borderTop: '1px solid var(--border-subtle)' }}>
          <p style={{ color: 'var(--fg-muted)', fontSize: '14px', marginBottom: '16px' }}>Or reach us directly:</p>
          <div style={{ display: 'flex', gap: '24px', justifyContent: 'center', flexWrap: 'wrap' }}>
            <a href="mailto:engineering@atoma.example" style={{ color: 'var(--accent-blue)', textDecoration: 'none' }}>📧 engineering@atoma.example</a>
            <a href="https://github.com/atoma" style={{ color: 'var(--accent-blue)', textDecoration: 'none' }}>🐙 GitHub</a>
            <a href="https://twitter.com/atoma" style={{ color: 'var(--accent-blue)', textDecoration: 'none' }}>🐦 Twitter</a>
          </div>
        </div>
      </div>
    </div>
  )
}
