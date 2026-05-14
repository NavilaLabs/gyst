// Sidebar + Topbar + main app shell

const Sidebar = ({ route, setRoute, workspaceId, onSwitchWorkspace, timer, onStartQuick, onStop, collapsed }) => {
  const elapsed = timer.running ? Math.floor((Date.now() - timer.startedAt) / 1000) : 0;
  const ws = WORKSPACES.find(w => w.id === workspaceId) || WORKSPACES[0];
  const items = [
    { group: 'Track', list: [
      { id: 'dashboard', label: 'Dashboard', icon: 'dashboard' },
      { id: 'timesheets', label: 'Timesheets', icon: 'clock' },
    ]},
    { group: 'Library', list: [
      { id: 'activities', label: 'Activities', icon: 'grid', count: ACTIVITIES.length },
      { id: 'tags', label: 'Tags', icon: 'hash', count: TAGS.length },
    ]},
    { group: 'Workspace', list: [
      { id: 'settings', label: 'Settings', icon: 'settings' },
    ]},
  ];
  return (
    <aside className="zk-sidebar" data-sidebar={collapsed ? 'icons' : 'full'}>
      <div className="zk-sidebar-brand">
        <div className="zk-sidebar-brand-mark">Z</div>
        <div className="zk-sidebar-brand-text">
          <div className="zk-sidebar-brand-name">Zeitrak</div>
          <div className="zk-sidebar-brand-sub">v 2.4 · stable</div>
        </div>
      </div>

      <button className="zk-sidebar-ws" onClick={onSwitchWorkspace}>
        <span className="zk-sidebar-ws-avatar" style={{ background: `linear-gradient(135deg, ${ws.accent}, ${ws.accent}aa)` }}>{ws.name[0].toUpperCase()}</span>
        <span className="zk-sidebar-ws-text">
          <div className="zk-sidebar-ws-name">{ws.name}</div>
          <div className="zk-sidebar-ws-sub">{ws.plan} · {ws.members} members</div>
        </span>
        <span className="zk-sidebar-ws-chev"><Icon name="chevronDown" size={12} /></span>
      </button>

      <nav className="zk-nav">
        {items.map(g => (
          <React.Fragment key={g.group}>
            <div className="zk-nav-section-title">{g.group}</div>
            {g.list.map(it => (
              <button
                key={it.id}
                className="zk-nav-item"
                data-active={route === it.id}
                onClick={() => setRoute(it.id)}
                title={collapsed ? it.label : undefined}>
                <span className="zk-nav-icon"><Icon name={it.icon} size={15} /></span>
                <span className="zk-nav-label">{it.label}</span>
                {it.count != null && <span className="zk-nav-badge">{it.count}</span>}
              </button>
            ))}
          </React.Fragment>
        ))}
      </nav>

      <div className="zk-sidebar-bottom">
        {timer.running ? (
          <div className="zk-runner">
            <span className="zk-runner-dot" />
            <div className="zk-runner-content">
              <div className="zk-runner-name">{timer.activity}</div>
              <div className="zk-runner-time">{formatHMS(elapsed)}</div>
            </div>
            <button className="zk-runner-stop" onClick={onStop} title="Stop">
              <Icon name="stop" size={11} />
            </button>
          </div>
        ) : (
          <button className="zk-start-pill" onClick={onStartQuick}>
            <Icon name="play" size={12} />
            <span>Quick start</span>
          </button>
        )}

        <div className="zk-sidebar-user">
          <div className="zk-sidebar-user-avatar">JK</div>
          <div className="zk-sidebar-user-meta">
            <div className="zk-sidebar-user-name">Jonas K.</div>
            <div className="zk-sidebar-user-email">jonas@konermann.online</div>
          </div>
        </div>
      </div>
    </aside>
  );
};

const Topbar = ({ route }) => {
  const titles = {
    dashboard: 'Dashboard', timesheets: 'Timesheets', activities: 'Activities', tags: 'Tags', settings: 'Settings'
  };
  return (
    <div className="zk-topbar">
      <div className="zk-row" style={{ gap: 10, color: 'var(--text-3)', fontSize: 12.5 }}>
        <span>main</span>
        <Icon name="chevronRight" size={11} />
        <span style={{ color: 'var(--text)', fontWeight: 600 }}>{titles[route]}</span>
      </div>
      <div className="zk-row" style={{ gap: 8 }}>
        <div className="zk-input-icon" style={{ width: 280 }}>
          <span className="zk-input-icon-svg"><Icon name="search" size={13} /></span>
          <input className="zk-input" placeholder="Search activities, tags, entries…" style={{ height: 32, fontSize: 12.5 }} />
          <span style={{ position: 'absolute', right: 8, top: '50%', transform: 'translateY(-50%)', fontSize: 10.5, color: 'var(--text-4)', fontFamily: 'var(--font-mono)', padding: '2px 5px', border: '1px solid var(--border-soft)', borderRadius: 4 }}>⌘K</span>
        </div>
        <button className="zk-icon-btn" title="Notifications"><Icon name="bell" size={14} /></button>
        <button className="zk-icon-btn" title="Help"><Icon name="help" size={14} /></button>
      </div>
    </div>
  );
};

const TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "accent": "#22c55e",
  "density": "comfortable",
  "displayFont": "Fraunces",
  "sidebarCollapsed": false,
  "showRunningTimer": true
}/*EDITMODE-END*/;

const App = () => {
  const [stage, setStage] = React.useState('login'); // login | workspaces | app
  const [route, setRoute] = React.useState('dashboard');
  const [workspaceId, setWorkspaceId] = React.useState('main');
  const [t, setTweak] = useTweaks(TWEAK_DEFAULTS);
  const [collapsed, setCollapsed] = React.useState(t.sidebarCollapsed);
  const [, force] = React.useReducer(x => x + 1, 0);
  const [timer, setTimer] = React.useState(() => t.showRunningTimer
    ? { running: true, activity: 'Deep work', startedAt: Date.now() - 1000 * 60 * 47 }
    : { running: false });

  // Tick clock for running timer
  React.useEffect(() => {
    if (!timer.running) return;
    const id = setInterval(force, 1000);
    return () => clearInterval(id);
  }, [timer.running]);

  // Apply tweaks as CSS vars
  React.useEffect(() => {
    const r = document.documentElement;
    r.style.setProperty('--accent', t.accent);
    // derive a softer hover variant by drawing alpha over surface; use rgba helper:
    r.style.setProperty('--accent-soft', t.accent + '1f');
    r.style.setProperty('--accent-soft-2', t.accent + '12');
    // accent-2 stays slightly desaturated
    r.style.setProperty('--font-display', `'${t.displayFont}', Georgia, serif`);
    r.dataset.density = t.density;
  }, [t.accent, t.displayFont, t.density]);

  const startTimer = (activity = 'Deep work') => setTimer({ running: true, activity, startedAt: Date.now() });
  const stopTimer = () => setTimer({ running: false });

  if (stage === 'login') {
    return (
      <>
        <LoginScreen onLogin={() => setStage('workspaces')} />
        <TweaksPanel title="Tweaks">
          <TweakSection label="Brand">
            <TweakColor label="Accent color" value={t.accent} onChange={v => setTweak('accent', v)} options={['#22c55e', '#3b82f6', '#f59e0b', '#ec4899']} />
            <TweakSelect label="Display typeface" value={t.displayFont} onChange={v => setTweak('displayFont', v)}
              options={[
                { value: 'Fraunces', label: 'Fraunces (serif)' },
                { value: 'Instrument Serif', label: 'Instrument Serif' },
                { value: 'Crimson Pro', label: 'Crimson Pro' },
                { value: 'Geist', label: 'Geist (sans)' },
              ]} />
          </TweakSection>
        </TweaksPanel>
      </>
    );
  }
  if (stage === 'workspaces') {
    return (
      <WorkspacesScreen
        onChoose={id => { setWorkspaceId(id); setStage('app'); setRoute('dashboard'); }}
        onLogout={() => setStage('login')}
      />
    );
  }

  const recent = ENTRIES;
  return (
    <>
      <div className="zk-app">
        <Sidebar
          route={route}
          setRoute={setRoute}
          workspaceId={workspaceId}
          onSwitchWorkspace={() => setStage('workspaces')}
          timer={timer}
          onStartQuick={() => startTimer('Deep work')}
          onStop={stopTimer}
          collapsed={collapsed}
          onToggleCollapse={() => { const v = !collapsed; setCollapsed(v); setTweak('sidebarCollapsed', v); }}
        />
        <main className="zk-main">
          <Topbar route={route} />
          <div className="zk-page">
            {route === 'dashboard' && <DashboardScreen timer={timer} startTimer={startTimer} stopTimer={stopTimer} recent={recent} />}
            {route === 'timesheets' && <TimesheetsScreen timer={timer} startTimer={startTimer} stopTimer={stopTimer} recent={recent} />}
            {route === 'activities' && <ActivitiesScreen />}
            {route === 'tags' && <TagsScreen />}
            {route === 'settings' && <SettingsScreen />}
          </div>
        </main>
      </div>

      <TweaksPanel title="Tweaks">
        <TweakSection label="Brand">
          <TweakColor label="Accent color" value={t.accent} onChange={v => setTweak('accent', v)} options={['#22c55e', '#3b82f6', '#f59e0b', '#ec4899', '#a855f7']} />
          <TweakSelect label="Display typeface" value={t.displayFont} onChange={v => setTweak('displayFont', v)}
            options={[
              { value: 'Fraunces', label: 'Fraunces (serif)' },
              { value: 'Instrument Serif', label: 'Instrument Serif' },
              { value: 'Crimson Pro', label: 'Crimson Pro' },
              { value: 'Geist', label: 'Geist (sans)' },
            ]} />
        </TweakSection>
        <TweakSection label="Layout">
          <TweakRadio label="Density" value={t.density} onChange={v => setTweak('density', v)}
            options={[{ value: 'compact', label: 'Compact' }, { value: 'comfortable', label: 'Comfortable' }]} />
          <TweakToggle label="Sidebar collapsed" value={collapsed} onChange={v => { setCollapsed(v); setTweak('sidebarCollapsed', v); }} />
        </TweakSection>
        <TweakSection label="Demo state">
          <TweakToggle label="Show running timer"
            value={timer.running}
            onChange={v => v ? startTimer('Deep work') : stopTimer()} />
          <TweakButton label="Jump to login" onClick={() => setStage('login')} />
          <TweakButton label="Switch workspace" onClick={() => setStage('workspaces')} />
        </TweakSection>
      </TweaksPanel>
    </>
  );
};

const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(<App />);
