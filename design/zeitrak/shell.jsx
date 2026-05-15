// Sidebar + Topbar + AppShell

const NAV = [
  { section: 'Tracking',     items: [
    { id: 'dashboard',  label: 'Dashboard',  icon: 'home' },
    { id: 'timesheets', label: 'Timesheets', icon: 'clock' },
  ]},
  { section: 'Library',      items: [
    { id: 'activities', label: 'Activities', icon: 'tag', badge: ACTIVITIES.length },
    { id: 'tags',       label: 'Tags',       icon: 'hash', badge: TAGS.length },
  ]},
  { section: 'Preferences',  items: [
    { id: 'settings',   label: 'Settings',   icon: 'settings' },
  ]},
];

const Sidebar = ({ page, setPage, timer, stopTimer, toggleSidebar, sidebarMode, onLogout, onSwitchWs }) => {
  const elapsed = timer.running ? Math.floor((Date.now() - timer.startedAt) / 1000) : 0;
  return (
    <aside className="zk-sidebar">
      <div className="zk-sidebar-brand">
        <div className="zk-sidebar-brand-mark">Z</div>
        <div className="zk-sidebar-brand-text">
          <div className="zk-sidebar-brand-name">Zeitrak</div>
          <div className="zk-sidebar-brand-sub">Time, well kept</div>
        </div>
      </div>

      <div className="zk-sidebar-ws" onClick={onSwitchWs} title="Switch workspace">
        <div className="zk-sidebar-ws-avatar">M</div>
        <div className="zk-sidebar-ws-text">
          <div className="zk-sidebar-ws-name">main</div>
          <div className="zk-sidebar-ws-sub">Pro · 4 members</div>
        </div>
        <div className="zk-sidebar-ws-chev"><Icon name="chevronUpDown" size={14} /></div>
      </div>

      <nav className="zk-nav">
        {NAV.map(sec => (
          <div key={sec.section}>
            <div className="zk-nav-section-title">{sec.section}</div>
            {sec.items.map(item => (
              <button
                key={item.id}
                className="zk-nav-item"
                data-active={page === item.id}
                onClick={() => setPage(item.id)}>
                <span className="zk-nav-icon"><Icon name={item.icon} size={16} /></span>
                <span className="zk-nav-label">{item.label}</span>
                {item.badge != null && <span className="zk-nav-badge">{item.badge}</span>}
              </button>
            ))}
          </div>
        ))}
      </nav>

      <div className="zk-sidebar-bottom">
        {timer.running && (
          <div className="zk-runner" title="Running session">
            <div className="zk-runner-dot" />
            <div className="zk-runner-content">
              <div className="zk-runner-name">{timer.activity}</div>
              <div className="zk-runner-time">{formatHMS(elapsed)}</div>
            </div>
            <button className="zk-runner-stop" onClick={stopTimer} title="Stop timer">
              <Icon name="stop" size={11} />
            </button>
          </div>
        )}
        {!timer.running && (
          <button className="zk-start-pill" onClick={() => setPage('timesheets')}>
            <Icon name="play" size={11} />
            <span>Start timer</span>
          </button>
        )}

        <div className="zk-sidebar-user">
          <div className="zk-sidebar-user-avatar">JK</div>
          <div className="zk-sidebar-user-meta">
            <div className="zk-sidebar-user-name">Jonas K.</div>
            <div className="zk-sidebar-user-email">jonas@konermann.online</div>
          </div>
          <button className="zk-icon-btn" onClick={() => setPage('settings')} title="Settings"><Icon name="settings" size={14} /></button>
        </div>
        <button className="zk-logout" onClick={onLogout}>
          <Icon name="logout" size={14} />
          <span>Sign out</span>
        </button>
      </div>
    </aside>
  );
};

const Topbar = ({ page, query, setQuery }) => {
  const labels = {
    dashboard: 'Dashboard',
    timesheets: 'Timesheets',
    activities: 'Activities',
    tags: 'Tags',
    settings: 'Settings',
  };
  return (
    <div className="zk-topbar">
      <div className="zk-breadcrumb">
        <span className="zk-breadcrumb-crumb">main</span>
        <span className="zk-breadcrumb-sep">/</span>
        <span className="zk-breadcrumb-current">{labels[page]}</span>
      </div>
      <div className="zk-topbar-actions">
        <div className="zk-input-icon" style={{ width: 240 }}>
          <span className="zk-input-icon-svg"><Icon name="search" size={14} /></span>
          <input className="zk-input" placeholder="Search entries, activities…" value={query} onChange={e => setQuery(e.target.value)} style={{ height: 32, fontSize: 12.5 }} />
        </div>
        <span className="zk-kbd"><kbd>⌘</kbd><kbd>K</kbd></span>
        <button className="zk-icon-btn" title="Notifications"><Icon name="bell" size={15} /></button>
        <button className="zk-icon-btn" title="Help"><Icon name="spark" size={15} /></button>
      </div>
    </div>
  );
};

Object.assign(window, { Sidebar, Topbar, NAV });
