// Icons + shared UI atoms

const Icon = ({ name, size = 16, className = '' }) => {
  const paths = {
    home: <path d="M3 10.5L10 4l7 6.5V17a1 1 0 01-1 1h-3v-5h-6v5H4a1 1 0 01-1-1v-6.5z" />,
    clock: <><circle cx="10" cy="10" r="7.25" /><path d="M10 6v4l2.5 2.5" /></>,
    tag: <><path d="M11 3H4a1 1 0 00-1 1v7l8 8 8-8-8-8z" /><circle cx="7.5" cy="7.5" r="1.25" fill="currentColor" /></>,
    hash: <><path d="M5 8h12M4 13h12M9 3l-2 14M14 3l-2 14" /></>,
    settings: <><circle cx="10" cy="10" r="2.5" /><path d="M10 2.5v2M10 15.5v2M2.5 10h2M15.5 10h2M4.7 4.7l1.4 1.4M13.9 13.9l1.4 1.4M4.7 15.3l1.4-1.4M13.9 6.1l1.4-1.4" /></>,
    play: <path d="M6 4l10 6-10 6V4z" fill="currentColor" stroke="none" />,
    stop: <rect x="5" y="5" width="10" height="10" rx="1" fill="currentColor" stroke="none" />,
    pause: <><rect x="6" y="4" width="3" height="12" rx="1" fill="currentColor" stroke="none" /><rect x="11" y="4" width="3" height="12" rx="1" fill="currentColor" stroke="none" /></>,
    plus: <path d="M10 4v12M4 10h12" />,
    minus: <path d="M4 10h12" />,
    edit: <path d="M14 3l3 3-9 9-4 1 1-4 9-9z" />,
    trash: <><path d="M4 6h12M8 6V4a1 1 0 011-1h2a1 1 0 011 1v2M6 6v10a1 1 0 001 1h6a1 1 0 001-1V6" /><path d="M9 9v5M11 9v5" /></>,
    chevronRight: <path d="M8 4l5 6-5 6" />,
    chevronDown: <path d="M5 8l5 5 5-5" />,
    chevronLeft: <path d="M12 4l-5 6 5 6" />,
    chevronUpDown: <><path d="M7 8l3-3 3 3" /><path d="M7 12l3 3 3-3" /></>,
    arrowRight: <><path d="M4 10h12" /><path d="M12 5l5 5-5 5" /></>,
    arrowUp: <><path d="M10 16V4M5 9l5-5 5 5" /></>,
    bolt: <path d="M11 2l-7 9h5l-1 7 7-9h-5l1-7z" fill="currentColor" stroke="none" />,
    spark: <><path d="M10 2v4M10 14v4M2 10h4M14 10h4M4.5 4.5l2.5 2.5M13 13l2.5 2.5M4.5 15.5l2.5-2.5M13 7l2.5-2.5" /></>,
    search: <><circle cx="9" cy="9" r="5.5" /><path d="M13 13l4 4" /></>,
    bell: <><path d="M5 8a5 5 0 0110 0v4l1.5 2H3.5L5 12V8z" /><path d="M8 16a2 2 0 004 0" /></>,
    user: <><circle cx="10" cy="7" r="3" /><path d="M3.5 17a6.5 6.5 0 0113 0" /></>,
    building: <><rect x="4" y="3" width="12" height="14" rx="1" /><path d="M7 7h2M11 7h2M7 10h2M11 10h2M7 13h2M11 13h2" /></>,
    logout: <><path d="M12 4h3a1 1 0 011 1v10a1 1 0 01-1 1h-3" /><path d="M9 13l-3-3 3-3" /><path d="M6 10h8" /></>,
    sidebar: <><rect x="3" y="4" width="14" height="12" rx="1" /><path d="M8 4v12" /></>,
    calendar: <><rect x="3.5" y="4.5" width="13" height="12" rx="1" /><path d="M3.5 8h13" /><path d="M7 3v3M13 3v3" /></>,
    filter: <path d="M3 5h14l-5 7v5l-4-2v-3L3 5z" />,
    download: <><path d="M10 3v10M5 9l5 5 5-5" /><path d="M3 17h14" /></>,
    check: <path d="M4 10l4 4 8-8" />,
    x: <path d="M5 5l10 10M15 5L5 15" />,
    moreHoriz: <><circle cx="5" cy="10" r="1.25" fill="currentColor" stroke="none"/><circle cx="10" cy="10" r="1.25" fill="currentColor" stroke="none"/><circle cx="15" cy="10" r="1.25" fill="currentColor" stroke="none"/></>,
    eye: <><path d="M2 10s3-6 8-6 8 6 8 6-3 6-8 6-8-6-8-6z" /><circle cx="10" cy="10" r="2.5" /></>,
    eyeOff: <><path d="M3 3l14 14" /><path d="M8 5.5A8.5 8.5 0 0110 5c5 0 8 6 8 6a14 14 0 01-2 2.6M6.4 6.4A14 14 0 002 11s3 6 8 6c1.5 0 2.7-.5 3.7-1.2" /></>,
    flame: <path d="M10 2c2 4 5 5 5 9a5 5 0 11-10 0c0-2 1-3 2-4 0 2 2 3 3 1 0-2-1-4 0-6z" />,
    target: <><circle cx="10" cy="10" r="7" /><circle cx="10" cy="10" r="4" /><circle cx="10" cy="10" r="1" fill="currentColor" stroke="none" /></>,
    activity: <path d="M2 10h3l3-7 4 14 3-7h3" />,
    tagFill: <><path d="M11 3H4a1 1 0 00-1 1v7l8 8 8-8-8-8z" fill="currentColor" stroke="none" /><circle cx="7.5" cy="7.5" r="1.1" fill="var(--surface)" stroke="none" /></>,
    save: <><path d="M4 3h9l3 3v11a1 1 0 01-1 1H4a1 1 0 01-1-1V4a1 1 0 011-1z" /><path d="M6 3v5h7V3M6 17v-6h8v6" /></>,
    dashboard: <><rect x="3" y="3" width="7" height="9" rx="1" /><rect x="12" y="3" width="5" height="5" rx="1" /><rect x="12" y="10" width="5" height="7" rx="1" /><rect x="3" y="14" width="7" height="3" rx="1" /></>,
    help: <><circle cx="10" cy="10" r="7.25" /><path d="M8 8a2 2 0 014 0c0 1-2 1.5-2 3M10 14h.01" /></>,
    grid: <><rect x="3" y="3" width="6" height="6" rx="1" /><rect x="11" y="3" width="6" height="6" rx="1" /><rect x="3" y="11" width="6" height="6" rx="1" /><rect x="11" y="11" width="6" height="6" rx="1" /></>,
    list: <><path d="M6 5h11M6 10h11M6 15h11" /><circle cx="3.5" cy="5" r=".75" fill="currentColor" stroke="none" /><circle cx="3.5" cy="10" r=".75" fill="currentColor" stroke="none" /><circle cx="3.5" cy="15" r=".75" fill="currentColor" stroke="none" /></>,
  };
  return (
    <svg className={className} width={size} height={size} viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      {paths[name]}
    </svg>
  );
};

// ---------- Card ----------
const Card = ({ title, icon, eyebrow, action, flush, children, style }) => (
  <div className="zk-card" style={style}>
    {(title || eyebrow || action) && (
      <div className="zk-card-head">
        {title ? (
          <div className="zk-card-title">
            {icon && <span className="zk-card-title-icon"><Icon name={icon} size={15} /></span>}
            <span>{title}</span>
          </div>
        ) : eyebrow ? (
          <div className="zk-card-eyebrow">{eyebrow}</div>
        ) : <span />}
        {action}
      </div>
    )}
    <div className={flush ? 'zk-card-flush' : 'zk-card-body'}>{children}</div>
  </div>
);

// ---------- Field ----------
const Field = ({ label, hint, children }) => (
  <div className="zk-field">
    {label && <label className="zk-label">{label}{hint && <span className="zk-label-hint">{hint}</span>}</label>}
    {children}
  </div>
);

// ---------- Stat ----------
const Stat = ({ label, value, unit, delta, deltaLabel }) => (
  <div className="zk-stat">
    <div className="zk-stat-label">{label}</div>
    <div className="zk-display zk-stat-value">
      {value}
      {unit && <span style={{ fontSize: '0.5em', color: 'var(--text-3)', fontFamily: 'var(--font-body)', marginLeft: 4, fontWeight: 500 }}>{unit}</span>}
    </div>
    {(delta != null || deltaLabel) && (
      <div className="zk-stat-meta">
        {delta != null && (
          <span className={`zk-stat-delta zk-stat-delta--${delta >= 0 ? 'up' : 'down'}`}>
            <Icon name="arrowUp" size={11} style={{ transform: delta < 0 ? 'rotate(180deg)' : 'none' }} />
            {Math.abs(delta)}%
          </span>
        )}
        <span>{deltaLabel}</span>
      </div>
    )}
  </div>
);

// ---------- Segmented ----------
const Segmented = ({ value, onChange, options }) => (
  <div className="zk-seg">
    {options.map(o => (
      <button key={o.value} className="zk-seg-item" data-active={value === o.value} onClick={() => onChange(o.value)}>
        {o.icon && <Icon name={o.icon} size={13} />}
        {o.label}
      </button>
    ))}
  </div>
);

// ---------- Tabs ----------
const Tabs = ({ value, onChange, options }) => (
  <div className="zk-tabs">
    {options.map(o => (
      <button key={o.value} className="zk-tab" data-active={value === o.value} onClick={() => onChange(o.value)}>
        {o.icon && <Icon name={o.icon} size={14} />}
        {o.label}
      </button>
    ))}
  </div>
);

// ---------- Empty ----------
const Empty = ({ icon = 'tag', title, desc, action }) => (
  <div className="zk-empty">
    <div className="zk-empty-icon"><Icon name={icon} size={20} /></div>
    <div className="zk-empty-title">{title}</div>
    {desc && <div className="zk-empty-desc">{desc}</div>}
    {action}
  </div>
);

// ---------- Helpers ----------
const formatHMS = (sec) => {
  const h = Math.floor(sec / 3600);
  const m = Math.floor((sec % 3600) / 60);
  const s = sec % 60;
  return [h, m, s].map(v => String(v).padStart(2, '0')).join(':');
};
const formatHM = (sec) => {
  const h = Math.floor(sec / 3600);
  const m = Math.floor((sec % 3600) / 60);
  return `${h}h ${String(m).padStart(2, '0')}m`;
};
const formatHshort = (sec) => {
  const h = sec / 3600;
  return h >= 10 ? `${h.toFixed(0)}h` : `${h.toFixed(1)}h`;
};

Object.assign(window, { Icon, Card, Field, Stat, Segmented, Tabs, Empty, formatHMS, formatHM, formatHshort });
