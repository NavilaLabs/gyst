// Login + Workspaces screens

const LoginScreen = ({ onLogin }) => {
  const [email, setEmail] = React.useState('jonas@konermann.online');
  const [pw, setPw] = React.useState('••••••••');
  const [showPw, setShowPw] = React.useState(false);
  return (
    <div className="zk-auth" data-screen-label="Login">
      <div className="zk-auth-form-wrap">
        <div className="zk-auth-brand">
          <div className="zk-sidebar-brand-mark" style={{ width: 32, height: 32, fontSize: 20 }}>Z</div>
          <div>
            <div className="zk-sidebar-brand-name" style={{ fontSize: 22 }}>Zeitrak</div>
          </div>
        </div>

        <div className="zk-auth-form">
          <div style={{ marginBottom: 28 }}>
            <h1 className="zk-display" style={{ fontFamily: 'var(--font-display)', fontSize: 34, lineHeight: 1.1, margin: 0, fontWeight: 400, letterSpacing: '-0.015em' }}>
              Welcome back.
            </h1>
            <p style={{ color: 'var(--text-2)', marginTop: 8, fontSize: 14 }}>
              Sign in to continue tracking your time.
            </p>
          </div>

          <div className="zk-stack" style={{ gap: 14 }}>
            <Field label="Work email">
              <div className="zk-input-icon">
                <span className="zk-input-icon-svg"><Icon name="user" size={14} /></span>
                <input className="zk-input" type="email" value={email} onChange={e => setEmail(e.target.value)} style={{ paddingLeft: 34 }} />
              </div>
            </Field>
            <Field label="Password" hint={<a style={{ color: 'var(--accent-2)', cursor: 'pointer' }}>Forgot?</a>}>
              <div className="zk-input-icon">
                <input className="zk-input" type={showPw ? 'text' : 'password'} value={pw} onChange={e => setPw(e.target.value)} style={{ paddingRight: 38 }} />
                <button onClick={() => setShowPw(!showPw)} className="zk-icon-btn" style={{ position: 'absolute', right: 4, top: '50%', transform: 'translateY(-50%)', width: 28, height: 28 }}>
                  <Icon name={showPw ? 'eyeOff' : 'eye'} size={14} />
                </button>
              </div>
            </Field>

            <label style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 12.5, color: 'var(--text-2)', cursor: 'pointer', marginTop: 2 }}>
              <input type="checkbox" defaultChecked style={{ accentColor: 'var(--accent)' }} />
              Keep me signed in for 30 days
            </label>

            <button className="zk-btn zk-btn--primary zk-btn--lg" onClick={onLogin} style={{ justifyContent: 'center', marginTop: 8 }}>
              <span>Sign in</span>
              <Icon name="arrowRight" size={14} />
            </button>

            <div style={{ display: 'flex', alignItems: 'center', gap: 12, color: 'var(--text-3)', fontSize: 11, margin: '8px 0' }}>
              <div style={{ flex: 1, height: 1, background: 'var(--border-soft)' }} />
              <span>OR</span>
              <div style={{ flex: 1, height: 1, background: 'var(--border-soft)' }} />
            </div>

            <button className="zk-btn zk-btn--outline" style={{ justifyContent: 'center', padding: '10px 14px' }}>
              <svg width="14" height="14" viewBox="0 0 18 18"><path fill="#4285f4" d="M17.6 9.2c0-.6 0-1.2-.2-1.8H9v3.4h4.8c-.2 1.1-.8 2-1.8 2.6v2.2h2.9c1.7-1.6 2.7-3.9 2.7-6.4z"/><path fill="#34a853" d="M9 18c2.4 0 4.5-.8 6-2.2l-2.9-2.2c-.8.5-1.8.9-3.1.9-2.4 0-4.4-1.6-5.1-3.8H1v2.3C2.5 16 5.5 18 9 18z"/><path fill="#fbbc05" d="M3.9 10.7C3.7 10.2 3.6 9.6 3.6 9s.1-1.2.3-1.7V5H1C.4 6.2 0 7.6 0 9s.4 2.8 1 4l2.9-2.3z"/><path fill="#ea4335" d="M9 3.6c1.3 0 2.5.5 3.5 1.4l2.6-2.6C13.5.9 11.4 0 9 0 5.5 0 2.5 2 1 5l2.9 2.3C4.6 5.1 6.6 3.6 9 3.6z"/></svg>
              Continue with Google
            </button>
            <button className="zk-btn zk-btn--outline" style={{ justifyContent: 'center', padding: '10px 14px' }}>
              <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M8 0C3.6 0 0 3.6 0 8c0 3.5 2.3 6.5 5.5 7.6.4.1.5-.2.5-.4v-1.4c-2.2.5-2.7-1.1-2.7-1.1-.4-.9-.9-1.2-.9-1.2-.7-.5.1-.5.1-.5.8.1 1.2.8 1.2.8.7 1.2 1.9.9 2.4.7.1-.5.3-.9.5-1.1-1.8-.2-3.6-.9-3.6-4 0-.9.3-1.6.8-2.1-.1-.2-.4-1 .1-2.1 0 0 .7-.2 2.2.8.6-.2 1.3-.3 2-.3s1.4.1 2 .3c1.5-1 2.2-.8 2.2-.8.4 1.1.2 1.9.1 2.1.5.5.8 1.2.8 2.1 0 3.1-1.8 3.7-3.6 3.9.3.3.6.8.6 1.6v2.4c0 .2.1.5.5.4 3.2-1.1 5.5-4.1 5.5-7.6 0-4.4-3.6-8-8-8z"/></svg>
              Continue with GitHub
            </button>

            <div style={{ marginTop: 18, fontSize: 13, color: 'var(--text-3)', textAlign: 'center' }}>
              No account? <a style={{ color: 'var(--text), cursor: pointer', textDecoration: 'underline', textDecorationColor: 'var(--accent)' }}>Create one</a>
            </div>
          </div>
        </div>

        <div className="zk-auth-footer">
          <span>© 2026 Zeitrak</span>
          <div style={{ display: 'flex', gap: 16 }}>
            <a className="zk-text-dim">Privacy</a>
            <a className="zk-text-dim">Terms</a>
            <a className="zk-text-dim">Status</a>
          </div>
        </div>
      </div>

      <div className="zk-auth-art">
        <div style={{ position: 'relative', zIndex: 1 }}>
          <div style={{ display: 'inline-flex', alignItems: 'center', gap: 8, fontSize: 11, letterSpacing: '0.12em', textTransform: 'uppercase', color: 'var(--accent-2)', fontWeight: 600 }}>
            <span style={{ width: 6, height: 6, borderRadius: 3, background: 'var(--accent-2)' }} />
            What's new in Zeitrak
          </div>
          <h2 className="zk-display" style={{ fontFamily: 'var(--font-display)', fontSize: 44, lineHeight: 1.05, letterSpacing: '-0.015em', margin: '14px 0 0', fontWeight: 400, maxWidth: 14 + 'ch' }}>
            A calmer way to track what your day adds up to.
          </h2>
          <p style={{ color: 'var(--text-2)', maxWidth: '46ch', marginTop: 14, fontSize: 14 }}>
            Start a session in one click. Tag it later. Roll it up at week's end. Zeitrak gets out of the way so the work stays the work.
          </p>
        </div>

        <div style={{ position: 'relative', zIndex: 1 }}>
          {/* Mock app preview tile */}
          <div className="zk-card" style={{ padding: 18, background: 'var(--bg)' }}>
            <div className="zk-row-spaced" style={{ marginBottom: 14 }}>
              <div className="zk-row" style={{ gap: 10 }}>
                <div className="zk-runner-dot" style={{ position: 'static' }} />
                <div>
                  <div style={{ fontSize: 12.5, fontWeight: 600 }}>Deep work</div>
                  <div style={{ fontSize: 11, color: 'var(--text-3)' }}>billable · since 09:14</div>
                </div>
              </div>
              <div className="zk-mono" style={{ color: 'var(--accent-2)', fontWeight: 600, fontSize: 14 }}>02:18:44</div>
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', gap: 4, alignItems: 'end', height: 56 }}>
              {[40, 70, 56, 0, 18, 88, 78].map((h, i) => (
                <div key={i} style={{ height: `${h}%`, background: i === 5 ? 'var(--accent)' : 'var(--surface-3)', borderRadius: 3, minHeight: 4 }} />
              ))}
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', gap: 4, marginTop: 6, fontFamily: 'var(--font-mono)', fontSize: 9.5, color: 'var(--text-3)', textAlign: 'center' }}>
              {['W','T','F','S','S','M','T'].map((d, i) => <span key={i}>{d}</span>)}
            </div>
          </div>
          <p style={{ marginTop: 18, fontSize: 12, color: 'var(--text-3)', display: 'flex', alignItems: 'center', gap: 8 }}>
            <Icon name="bolt" size={13} />
            Trusted by 4,200+ teams · SOC2 ready · EU‑hosted
          </p>
        </div>
      </div>
    </div>
  );
};

const WorkspacesScreen = ({ onChoose, onLogout }) => (
  <div className="zk-ws-screen" data-screen-label="Workspaces">
    <div className="zk-ws-screen-head">
      <div className="zk-row" style={{ gap: 10 }}>
        <div className="zk-sidebar-brand-mark" style={{ width: 28, height: 28, fontSize: 17 }}>Z</div>
        <span style={{ fontFamily: 'var(--font-display)', fontSize: 20, fontWeight: 400 }}>Zeitrak</span>
      </div>
      <div className="zk-row" style={{ gap: 8 }}>
        <button className="zk-btn zk-btn--ghost zk-btn--sm" onClick={onLogout}>Sign out</button>
        <button className="zk-icon-btn"><Icon name="settings" size={15} /></button>
      </div>
    </div>

    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', padding: '48px 24px' }}>
      <div style={{ width: '100%', maxWidth: 580 }}>
        <div style={{ fontSize: 11, letterSpacing: '0.12em', textTransform: 'uppercase', color: 'var(--accent-2)', fontWeight: 700, display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ width: 6, height: 6, borderRadius: 3, background: 'var(--accent-2)' }} />
          Signed in as Jonas
        </div>
        <h1 className="zk-display" style={{ fontFamily: 'var(--font-display)', fontSize: 48, fontWeight: 400, letterSpacing: '-0.015em', lineHeight: 1.05, margin: '14px 0 6px' }}>
          Choose a <em style={{ color: 'var(--accent-2)' }}>workspace</em>.
        </h1>
        <p style={{ color: 'var(--text-2)', fontSize: 14, marginBottom: 28 }}>
          Pick where you'd like to track time today. You can switch from the sidebar at any moment.
        </p>

        <div className="zk-ws-list">
          {WORKSPACES.map(ws => (
            <div key={ws.id} className="zk-ws-card" onClick={() => onChoose(ws.id)}>
              <div className="zk-ws-avatar" style={{ background: `linear-gradient(135deg, ${ws.accent}, ${ws.accent}aa)` }}>
                {ws.name[0].toUpperCase()}
              </div>
              <div className="zk-ws-meta">
                <div className="zk-ws-name">{ws.name}</div>
                <div className="zk-ws-sub">{ws.sub}</div>
              </div>
              <div className="zk-ws-stats">
                <div><strong>{ws.members}</strong> members</div>
                <div>{ws.lastActive}</div>
                <div className="zk-pill" style={{ padding: '2px 7px', fontSize: 10.5 }}>{ws.plan}</div>
              </div>
              <Icon name="arrowRight" size={15} className="zk-text-dim" />
            </div>
          ))}

          <button className="zk-ws-card" style={{ background: 'transparent', borderStyle: 'dashed', justifyContent: 'center', color: 'var(--text-2)' }}>
            <Icon name="plus" size={15} />
            <span style={{ fontWeight: 500, fontSize: 13.5 }}>Create new workspace</span>
          </button>
        </div>
      </div>
    </div>
  </div>
);

Object.assign(window, { LoginScreen, WorkspacesScreen });
