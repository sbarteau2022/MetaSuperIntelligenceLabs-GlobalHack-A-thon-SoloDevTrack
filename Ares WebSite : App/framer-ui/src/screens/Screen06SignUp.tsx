export function Screen06SignUp() {
  return (
    <div className="screen-06" style={{ padding: '40px', maxWidth: '500px', margin: '0 auto', height: '100vh', overflow: 'auto', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', textAlign: 'center' }}>
      <div style={{ width: '100%' }}>
        <h1 style={{ fontSize: '32px', fontWeight: 700, marginBottom: '8px' }}>Sign Up</h1>
        <p style={{ color: 'var(--fg-muted)', fontSize: '16px', marginBottom: '40px' }}>
          Create your Atoma Engineering account for early access
        </p>

        <form style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
          <div style={{ textAlign: 'left' }}>
            <label style={{ display: 'block', fontSize: '13px', fontWeight: 500', marginBottom: '8px', color: 'var(--fg-secondary)' }}>Full Name</label>
            <input type="text" placeholder="John Doe" style={{ width: '100%', padding: '12px 16px', borderRadius: 'var(--radius-md)', background: 'var(--bg-primary)', border: '1px solid var(--border-subtle)', color: 'var(--fg-primary)', fontSize: '14px' }} />
          </div>
          <div style={{ textAlign: 'left' }}>
            <label style={{ display: 'block', fontSize: '13px', fontWeight: 500', marginBottom: '8px', color: 'var(--fg-secondary)' }}>Email</label>
            <input type="email" placeholder="you@example.com" style={{ width: '100%', padding: '12px 16px', borderRadius: 'var(--radius-md)', background: 'var(--bg-primary)', border: '1px solid var(--border-subtle)', color: 'var(--fg-primary)', fontSize: '14px' }} />
          </div>
          <div style={{ textAlign: 'left' }}>
            <label style={{ display: 'block', fontSize: '13px', fontWeight: 500', marginBottom: '8px', color: 'var(--fg-secondary)' }}>Password</label>
            <input type="password" placeholder="••••••••" style={{ width: '100%', padding: '12px 16px', borderRadius: 'var(--radius-md)', background: 'var(--bg-primary)', border: '1px solid var(--border-subtle)', color: 'var(--fg-primary)', fontSize: '14px' }} />
          </div>
          <div style={{ textAlign: 'left' }}>
            <label style={{ display: 'block', fontSize: '13px', fontWeight: 500', marginBottom: '8px', color: 'var(--fg-secondary)' }}>Confirm Password</label>
            <input type="password" placeholder="••••••••" style={{ width: '100%', padding: '12px 16px', borderRadius: 'var(--radius-md)', background: 'var(--bg-primary)', border: '1px solid var(--border-subtle)', color: 'var(--fg-primary)', fontSize: '14px' }} />
          </div>
          <label style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '13px', color: 'var(--fg-secondary)', cursor: 'pointer' }}>
            <input type="checkbox" style={{ accentColor: 'var(--accent-blue)' }} />
            I agree to the Terms of Service and Privacy Policy
          </label>
          <button type="submit" style={{ padding: '14px 28px', borderRadius: 'var(--radius-md)', background: 'var(--accent-blue)', color: 'white', border: 'none', fontSize: '14px', fontWeight: 500, cursor: 'pointer', marginTop: '8px' }}>
            Create Account
          </button>
        </form>

        <p style={{ marginTop: '24px', color: 'var(--fg-muted)', fontSize: '14px' }}>
          Already have an account? <a href="#" style={{ color: 'var(--accent-blue)' }}>Sign in</a>
        </p>
      </div>
    </div>
  )
}
