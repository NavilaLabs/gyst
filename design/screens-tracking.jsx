// Dashboard + Timesheets

const BarChart = ({ data, accent }) => {
  const [hover, setHover] = React.useState(null);
  const max = Math.max(...data.map(d => d.sec), 8 * 3600);
  const W = 720, H = 220;
  const PAD = { l: 36, r: 12, t: 12, b: 28 };
  const innerW = W - PAD.l - PAD.r;
  const innerH = H - PAD.t - PAD.b;
  const barW = innerW / data.length * 0.6;
  const step = innerW / data.length;
  const yTicks = [0, 2, 4, 6, 8];
  const yMax = 8 * 3600;
  return (
    <div className="zk-chart" style={{ position: 'relative' }}>
      <svg viewBox={`0 0 ${W} ${H}`} width="100%" preserveAspectRatio="none" style={{ display: 'block', height: 220 }}>
        {yTicks.map(t => {
          const y = PAD.t + innerH - (t / 8) * innerH;
          return (
            <g key={t}>
              <line className="zk-chart-grid" x1={PAD.l} x2={W - PAD.r} y1={y} y2={y} />
              <text className="zk-chart-axis" x={PAD.l - 8} y={y + 3} textAnchor="end">{t}h</text>
            </g>
          );
        })}
        {data.map((d, i) => {
          const x = PAD.l + i * step + (step - barW) / 2;
          const h = (d.sec / yMax) * innerH;
          const y = PAD.t + innerH - h;
          const isHover = hover === i;
          return (
            <g key={i} onMouseEnter={() => setHover(i)} onMouseLeave={() => setHover(null)} style={{ cursor: 'pointer' }}>
              <rect x={PAD.l + i * step} y={PAD.t} width={step} height={innerH} fill="transparent" />
              <rect className="zk-chart-bar" x={x} y={y} width={barW} height={Math.max(h, 2)} rx="3" opacity={hover != null && !isHover ? 0.4 : 1} />
              <text className="zk-chart-axis" x={x + barW / 2} y={H - 10} textAnchor="middle">{d.day}</text>
            </g>
          );
        })}
      </svg>
      {hover != null && (
        <div className="zk-chart-tooltip" style={{ left: `${((hover + 0.5) / data.length) * 100}%`, top: 24 }}>
          <div style={{ color: 'var(--text-3)', fontSize: 10.5 }}>{data[hover].date}</div>
          <div className="zk-chart-tooltip-time">{formatHM(data[hover].sec)}</div>
        </div>
      )}
    </div>
  );
};

const Donut = ({ activities }) => {
  const total = activities.reduce((s, a) => s + a.tracked, 0);
  let acc = 0;
  const R = 50, C = 60;
  const segs = activities.map(a => {
    const start = acc / total;
    acc += a.tracked;
    const end = acc / total;
    return { ...a, start, end, pct: (a.tracked / total) * 100 };
  });
  const polar = (t) => {
    const ang = t * Math.PI * 2 - Math.PI / 2;
    return [C + R * Math.cos(ang), C + R * Math.sin(ang)];
  };
  return (
    <div className="zk-donut-row">
      <svg viewBox="0 0 120 120" width="140" height="140">
        <circle cx={C} cy={C} r={R} fill="none" stroke="var(--surface-2)" strokeWidth="14" />
        {segs.map((s, i) => {
          const [x1, y1] = polar(s.start);
          const [x2, y2] = polar(s.end);
          const large = s.end - s.start > 0.5 ? 1 : 0;
          return <path key={i} d={`M ${x1} ${y1} A ${R} ${R} 0 ${large} 1 ${x2} ${y2}`} stroke={s.color} strokeWidth="14" fill="none" strokeLinecap="butt" />;
        })}
        <text x={C} y={C - 4} textAnchor="middle" fontSize="9" fill="var(--text-3)" letterSpacing="1.5" style={{ textTransform: 'uppercase', fontWeight: 600 }}>Total</text>
        <text x={C} y={C + 12} textAnchor="middle" fontSize="16" fill="var(--text)" fontFamily="var(--font-mono)" fontWeight="500">{formatHshort(total)}</text>
      </svg>
      <div className="zk-donut-legend">
        {segs.map(s => (
          <div key={s.id} className="zk-donut-legend-row">
            <span className="zk-donut-legend-dot" style={{ background: s.color }} />
            <span className="zk-donut-legend-name">{s.name}</span>
            <span className="zk-donut-legend-time">{formatHM(s.tracked)}</span>
            <span className="zk-donut-legend-pct">{s.pct.toFixed(0)}%</span>
          </div>
        ))}
      </div>
    </div>
  );
};

const DashboardScreen = ({ timer, startTimer, stopTimer, recent }) => {
  const [picked, setPicked] = React.useState(ACTIVITIES[0].id);
  const todaySec = recent.filter(e => e.start.startsWith('Today')).reduce((s, e) => s + e.duration, 0);
  const weekSec = WEEK_DATA.reduce((s, d) => s + d.sec, 0);
  const elapsed = timer.running ? Math.floor((Date.now() - timer.startedAt) / 1000) : 0;
  const billableSec = recent.filter(e => e.tags.includes('billable')).reduce((s, e) => s + e.duration, 0);
  const streak = 5;
  return (
    <div data-screen-label="Dashboard">
      <div className="zk-page-header">
        <div>
          <div className="zk-page-eyebrow">Tuesday · May 12</div>
          <h1 className="zk-page-title">Good afternoon, <em>Jonas</em>.</h1>
          <p className="zk-page-subtitle">Here's where your week stands so far.</p>
        </div>
        <div className="zk-row" style={{ gap: 8 }}>
          <button className="zk-btn zk-btn--outline zk-btn--sm"><Icon name="download" size={13} /> Export week</button>
          <button className="zk-btn zk-btn--outline zk-btn--sm"><Icon name="calendar" size={13} /> This week</button>
        </div>
      </div>

      <div className="zk-stack-lg">
        {/* Quick start / running session */}
        <Card flush>
          {timer.running ? (
            <div style={{ padding: 'var(--pad-card)', display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 24, flexWrap: 'wrap' }}>
              <div className="zk-row" style={{ gap: 18 }}>
                <div style={{ width: 44, height: 44, borderRadius: 12, background: 'var(--accent-soft)', display: 'flex', alignItems: 'center', justifyContent: 'center', position: 'relative' }}>
                  <div className="zk-runner-dot" style={{ width: 10, height: 10 }} />
                </div>
                <div>
                  <div style={{ fontSize: 11, letterSpacing: '0.1em', textTransform: 'uppercase', color: 'var(--text-3)', fontWeight: 600 }}>Currently running</div>
                  <div style={{ fontSize: 18, fontWeight: 600, marginTop: 4 }}>{timer.activity}</div>
                </div>
              </div>
              <div className="zk-big-timer">
                {formatHMS(elapsed).split(':').map((p, i, arr) => (
                  <React.Fragment key={i}>
                    <span>{p}</span>
                    {i < arr.length - 1 && <span style={{ color: 'var(--text-4)' }}>:</span>}
                  </React.Fragment>
                ))}
              </div>
              <div className="zk-row" style={{ gap: 6 }}>
                <button className="zk-btn zk-btn--outline"><Icon name="pause" size={13} /> Pause</button>
                <button className="zk-btn zk-btn--danger" onClick={stopTimer}><Icon name="stop" size={13} /> Stop</button>
              </div>
            </div>
          ) : (
            <div style={{ padding: 'var(--pad-card)', display: 'grid', gridTemplateColumns: '1fr auto', gap: 16, alignItems: 'end' }}>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16 }}>
                <Field label="Activity">
                  <select className="zk-select" value={picked} onChange={e => setPicked(e.target.value)}>
                    {ACTIVITIES.map(a => <option key={a.id} value={a.id}>{a.name}</option>)}
                  </select>
                </Field>
                <Field label="Tags" hint="optional">
                  <input className="zk-input" placeholder="billable, customer-acme…" />
                </Field>
              </div>
              <button className="zk-btn zk-btn--primary zk-btn--lg" onClick={() => startTimer(ACTIVITIES.find(a => a.id === picked).name)}>
                <Icon name="play" size={13} /> Start session
              </button>
            </div>
          )}
        </Card>

        {/* Stats row */}
        <div className="zk-grid zk-grid-4">
          <Stat label="Today" value={formatHshort(todaySec).replace('h','')} unit="h" delta={12} deltaLabel="vs avg" />
          <Stat label="This week" value={formatHshort(weekSec).replace('h','')} unit="h" delta={-4} deltaLabel="vs last" />
          <Stat label="Billable" value={formatHshort(billableSec).replace('h','')} unit="h" deltaLabel="64% of tracked" />
          <Stat label="Streak" value={streak} unit="days" deltaLabel="best: 12 days" />
        </div>

        {/* Chart + Donut */}
        <div className="zk-grid" style={{ gridTemplateColumns: '1.6fr 1fr', gap: 16 }}>
          <Card title="Hours per day" icon="activity" action={<Segmented value="week" onChange={()=>{}} options={[{value:'week',label:'Week'},{value:'month',label:'Month'},{value:'year',label:'Year'}]} />}>
            <BarChart data={WEEK_DATA} />
          </Card>
          <Card title="Activity mix" icon="tag" action={<span className="zk-text-dim" style={{ fontSize: 11.5 }}>last 7 days</span>}>
            <Donut activities={ACTIVITIES} />
          </Card>
        </div>

        {/* Recent entries */}
        <Card title="Recent entries" icon="clock" action={<button className="zk-btn zk-btn--ghost zk-btn--sm">View all <Icon name="arrowRight" size={11} /></button>} flush>
          <div>
            {recent.slice(0, 6).map(e => {
              const a = ACTIVITIES.find(x => x.id === e.activityId);
              return (
                <div key={e.id} className="zk-entry">
                  <div className="zk-entry-color" style={{ background: a?.color || 'var(--accent)' }} />
                  <div className="zk-entry-main">
                    <div className="zk-entry-name">{e.activity}</div>
                    <div className="zk-entry-meta">
                      <span>{e.start}</span>
                      {e.desc && <><span>·</span><span>{e.desc}</span></>}
                      {e.tags.map(t => {
                        const tag = TAGS.find(x => x.name === t);
                        return <span key={t} className="zk-tag"><span className="zk-tag-dot" style={{ background: tag?.color }} />{t}</span>;
                      })}
                    </div>
                  </div>
                  <div className="zk-entry-time">{formatHM(e.duration)}</div>
                  <div className="zk-entry-actions">
                    <button className="zk-icon-action" title="Edit"><Icon name="edit" size={13} /></button>
                    <button className="zk-icon-action" title="Resume"><Icon name="play" size={11} /></button>
                  </div>
                </div>
              );
            })}
          </div>
        </Card>
      </div>
    </div>
  );
};

const TimesheetsScreen = ({ timer, startTimer, stopTimer, recent }) => {
  const [mode, setMode] = React.useState('timer');
  const [activity, setActivity] = React.useState(ACTIVITIES[0].id);
  const [desc, setDesc] = React.useState('');
  const [filter, setFilter] = React.useState('week');
  const elapsed = timer.running ? Math.floor((Date.now() - timer.startedAt) / 1000) : 0;
  return (
    <div data-screen-label="Timesheets">
      <div className="zk-page-header">
        <div>
          <div className="zk-page-eyebrow">Tracking</div>
          <h1 className="zk-page-title">Timesheets</h1>
          <p className="zk-page-subtitle">Run a live timer or log entries by hand. Everything rolls up to your week.</p>
        </div>
        <div className="zk-row" style={{ gap: 8 }}>
          <button className="zk-btn zk-btn--outline zk-btn--sm"><Icon name="filter" size={13} /> Filter</button>
          <button className="zk-btn zk-btn--outline zk-btn--sm"><Icon name="download" size={13} /> Export CSV</button>
        </div>
      </div>

      <div className="zk-stack-lg">
        <Card
          title={mode === 'timer' ? (timer.running ? 'Session in progress' : 'New session') : 'Manual entry'}
          icon={mode === 'timer' ? 'play' : 'plus'}
          action={<Segmented value={mode} onChange={setMode} options={[{value:'timer',label:'Timer',icon:'clock'},{value:'manual',label:'Manual',icon:'plus'}]} />}>
          {mode === 'timer' ? (
            timer.running ? (
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 24, flexWrap: 'wrap' }}>
                <div>
                  <div className="zk-row" style={{ gap: 8, marginBottom: 6 }}>
                    <div className="zk-runner-dot" style={{ position: 'static' }} />
                    <span style={{ fontSize: 12, color: 'var(--text-3)' }}>RUNNING</span>
                  </div>
                  <div style={{ fontSize: 18, fontWeight: 600, marginBottom: 2 }}>{timer.activity}</div>
                  <div style={{ fontSize: 12, color: 'var(--text-3)' }}>started at 09:14 · billable</div>
                </div>
                <div className="zk-big-timer">{formatHMS(elapsed)}</div>
                <div className="zk-row" style={{ gap: 6 }}>
                  <button className="zk-btn zk-btn--outline"><Icon name="pause" size={13} /> Pause</button>
                  <button className="zk-btn zk-btn--danger" onClick={stopTimer}><Icon name="stop" size={13} /> Stop & save</button>
                </div>
              </div>
            ) : (
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr auto', gap: 14, alignItems: 'end' }}>
                <Field label="Activity">
                  <select className="zk-select" value={activity} onChange={e => setActivity(e.target.value)}>
                    {ACTIVITIES.map(a => <option key={a.id} value={a.id}>{a.name}</option>)}
                  </select>
                </Field>
                <Field label="Description" hint="optional">
                  <input className="zk-input" placeholder="What are you working on?" value={desc} onChange={e => setDesc(e.target.value)} />
                </Field>
                <button className="zk-btn zk-btn--primary zk-btn--lg" onClick={() => startTimer(ACTIVITIES.find(a => a.id === activity).name)}>
                  <Icon name="play" size={13} /> Start
                </button>
              </div>
            )
          ) : (
            <div className="zk-stack" style={{ gap: 14 }}>
              <div className="zk-grid zk-grid-2">
                <Field label="Start"><input type="datetime-local" className="zk-input" defaultValue="2026-05-12T09:14" /></Field>
                <Field label="End"><input type="datetime-local" className="zk-input" defaultValue="2026-05-12T11:32" /></Field>
              </div>
              <div className="zk-grid zk-grid-2">
                <Field label="Activity">
                  <select className="zk-select"><option>Deep work</option></select>
                </Field>
                <Field label="Tags">
                  <input className="zk-input" placeholder="billable, internal…" />
                </Field>
              </div>
              <Field label="Description"><textarea className="zk-textarea" placeholder="Optional notes" /></Field>
              <div className="zk-row" style={{ gap: 8 }}>
                <button className="zk-btn zk-btn--primary"><Icon name="plus" size={13} /> Save entry</button>
                <button className="zk-btn zk-btn--ghost">Cancel</button>
              </div>
            </div>
          )}
        </Card>

        <Card title="Recent" icon="list" flush
          action={
            <div className="zk-row" style={{ gap: 8 }}>
              <Segmented value={filter} onChange={setFilter} options={[{value:'today',label:'Today'},{value:'week',label:'Week'},{value:'month',label:'Month'}]} />
            </div>
          }>
          <table className="zk-table">
            <thead>
              <tr>
                <th>Activity</th>
                <th>Description</th>
                <th>Tags</th>
                <th>Start</th>
                <th style={{ textAlign: 'right' }}>Duration</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {recent.map(e => {
                const a = ACTIVITIES.find(x => x.id === e.activityId);
                return (
                  <tr key={e.id}>
                    <td>
                      <div className="zk-row" style={{ gap: 8 }}>
                        <span style={{ width: 8, height: 8, borderRadius: 2, background: a?.color }} />
                        <span style={{ fontWeight: 500 }}>{e.activity}</span>
                      </div>
                    </td>
                    <td className="zk-text-2" style={{ maxWidth: 280, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                      {e.desc || <span className="zk-text-dim" style={{ fontStyle: 'italic' }}>—</span>}
                    </td>
                    <td>
                      <div className="zk-row" style={{ gap: 4, flexWrap: 'wrap' }}>
                        {e.tags.length === 0 && <span className="zk-text-dim">—</span>}
                        {e.tags.map(t => {
                          const tag = TAGS.find(x => x.name === t);
                          return <span key={t} className="zk-tag"><span className="zk-tag-dot" style={{ background: tag?.color }} />{t}</span>;
                        })}
                      </div>
                    </td>
                    <td className="zk-cell-num zk-text-2">{e.start}</td>
                    <td className="zk-cell-num" style={{ textAlign: 'right', fontWeight: 600 }}>{formatHM(e.duration)}</td>
                    <td className="zk-cell-actions">
                      <button className="zk-icon-action" title="Edit"><Icon name="edit" size={13} /></button>
                      <button className="zk-icon-action" title="Tag"><Icon name="tag" size={13} /></button>
                      <button className="zk-icon-action zk-icon-action--danger" title="Delete"><Icon name="trash" size={13} /></button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </Card>
      </div>
    </div>
  );
};

Object.assign(window, { DashboardScreen, TimesheetsScreen });
