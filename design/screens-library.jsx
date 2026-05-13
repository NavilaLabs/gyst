// Activities + Tags screens

const ActivitiesScreen = () => {
  const [name, setName] = React.useState('');
  const [comment, setComment] = React.useState('');
  const [color, setColor] = React.useState('#22c55e');
  const palette = ['#22c55e','#3b82f6','#a855f7','#f59e0b','#06b6d4','#ef4444','#ec4899','#84cc16'];
  return (
    <div data-screen-label="Activities">
      <div className="zk-page-header">
        <div>
          <div className="zk-page-eyebrow">Library</div>
          <h1 className="zk-page-title">Activities</h1>
          <p className="zk-page-subtitle">The categories you track time against. Each activity gets a color to make charts and entries scannable.</p>
        </div>
      </div>

      <div className="zk-grid" style={{ gridTemplateColumns: '380px 1fr', gap: 16, alignItems: 'start' }}>
        <Card title="New activity" icon="plus">
          <div className="zk-stack" style={{ gap: 14 }}>
            <Field label="Name">
              <input className="zk-input" placeholder="e.g. Deep work" value={name} onChange={e => setName(e.target.value)} />
            </Field>
            <Field label="Color">
              <div className="zk-row" style={{ gap: 8, flexWrap: 'wrap' }}>
                {palette.map(c => (
                  <button key={c} className="zk-color-swatch" data-selected={color === c} style={{ background: c, width: 24, height: 24 }} onClick={() => setColor(c)} />
                ))}
              </div>
            </Field>
            <Field label="Comment" hint="optional">
              <textarea className="zk-textarea" placeholder="What kind of work falls under this?" value={comment} onChange={e => setComment(e.target.value)} />
            </Field>
            <div className="zk-row" style={{ gap: 8 }}>
              <button className="zk-btn zk-btn--primary"><Icon name="plus" size={13} /> Create</button>
              <button className="zk-btn zk-btn--ghost">Reset</button>
            </div>
          </div>
        </Card>

        <Card title={`All activities · ${ACTIVITIES.length}`} icon="grid" flush
          action={
            <div className="zk-row" style={{ gap: 8 }}>
              <div className="zk-input-icon" style={{ width: 200 }}>
                <span className="zk-input-icon-svg"><Icon name="search" size={13} /></span>
                <input className="zk-input" placeholder="Search…" style={{ height: 30, fontSize: 12.5 }} />
              </div>
            </div>
          }>
          <table className="zk-table">
            <thead>
              <tr>
                <th>Activity</th>
                <th>Description</th>
                <th style={{ textAlign: 'right' }}>Tracked</th>
                <th>Last used</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {ACTIVITIES.map(a => (
                <tr key={a.id}>
                  <td>
                    <div className="zk-row" style={{ gap: 10 }}>
                      <span style={{ width: 10, height: 10, borderRadius: 3, background: a.color }} />
                      <span style={{ fontWeight: 600 }}>{a.name}</span>
                    </div>
                  </td>
                  <td className="zk-text-2" style={{ maxWidth: 320 }}>{a.comment}</td>
                  <td className="zk-cell-num" style={{ textAlign: 'right', fontWeight: 600 }}>{formatHM(a.tracked)}</td>
                  <td className="zk-text-dim">{a.lastUsed}</td>
                  <td className="zk-cell-actions">
                    <button className="zk-icon-action" title="Edit"><Icon name="edit" size={13} /></button>
                    <button className="zk-icon-action zk-icon-action--danger" title="Delete"><Icon name="trash" size={13} /></button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </Card>
      </div>
    </div>
  );
};

const TagsScreen = () => {
  const [name, setName] = React.useState('');
  const [color, setColor] = React.useState('#22c55e');
  const palette = ['#22c55e','#3b82f6','#a855f7','#f59e0b','#06b6d4','#ef4444','#ec4899','#84cc16','#a4a4ae'];
  return (
    <div data-screen-label="Tags">
      <div className="zk-page-header">
        <div>
          <div className="zk-page-eyebrow">Library</div>
          <h1 className="zk-page-title">Tags</h1>
          <p className="zk-page-subtitle">Lightweight labels for slicing your time — billable status, clients, priorities. Tag entries inline or in bulk.</p>
        </div>
      </div>

      <div className="zk-grid" style={{ gridTemplateColumns: '380px 1fr', gap: 16, alignItems: 'start' }}>
        <Card title="New tag" icon="hash">
          <div className="zk-stack" style={{ gap: 14 }}>
            <Field label="Name">
              <input className="zk-input" placeholder="e.g. urgent" value={name} onChange={e => setName(e.target.value)} />
            </Field>
            <Field label="Color">
              <div className="zk-row" style={{ gap: 8, flexWrap: 'wrap' }}>
                {palette.map(c => (
                  <button key={c} className="zk-color-swatch" data-selected={color === c} style={{ background: c, width: 24, height: 24 }} onClick={() => setColor(c)} />
                ))}
              </div>
            </Field>
            {name && (
              <div style={{ padding: '12px 14px', background: 'var(--surface-2)', borderRadius: 8, border: '1px solid var(--border-soft)' }}>
                <div style={{ fontSize: 11, color: 'var(--text-3)', marginBottom: 6, letterSpacing: '0.06em', textTransform: 'uppercase', fontWeight: 600 }}>Preview</div>
                <span className="zk-tag"><span className="zk-tag-dot" style={{ background: color }} />{name}</span>
              </div>
            )}
            <div className="zk-row" style={{ gap: 8 }}>
              <button className="zk-btn zk-btn--primary"><Icon name="plus" size={13} /> Create</button>
              <button className="zk-btn zk-btn--ghost">Reset</button>
            </div>
          </div>
        </Card>

        <Card title={`All tags · ${TAGS.length}`} icon="hash" flush>
          {TAGS.length === 0 ? (
            <Empty
              icon="hash"
              title="No tags yet"
              desc="Tags help you slice tracked time later — billable, internal, by-client, by-priority. Add your first one on the left."
            />
          ) : (
            <table className="zk-table">
              <thead>
                <tr>
                  <th>Tag</th>
                  <th style={{ textAlign: 'right' }}>Used in</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {TAGS.map(t => (
                  <tr key={t.id}>
                    <td>
                      <span className="zk-tag" style={{ fontSize: 12.5, padding: '4px 10px' }}>
                        <span className="zk-tag-dot" style={{ background: t.color }} />
                        {t.name}
                      </span>
                    </td>
                    <td className="zk-cell-num zk-text-2" style={{ textAlign: 'right' }}>{t.count} entries</td>
                    <td className="zk-cell-actions">
                      <button className="zk-icon-action" title="Edit"><Icon name="edit" size={13} /></button>
                      <button className="zk-icon-action zk-icon-action--danger" title="Delete"><Icon name="trash" size={13} /></button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </Card>
      </div>
    </div>
  );
};

const SettingsScreen = () => {
  const [tab, setTab] = React.useState('me');
  return (
    <div data-screen-label="Settings">
      <div className="zk-page-header">
        <div>
          <div className="zk-page-eyebrow">Preferences</div>
          <h1 className="zk-page-title">Settings</h1>
          <p className="zk-page-subtitle">Tweak how Zeitrak behaves for you and for the workspace.</p>
        </div>
      </div>

      <Tabs value={tab} onChange={setTab} options={[
        { value: 'me', label: 'My settings', icon: 'user' },
        { value: 'ws', label: 'Workspace', icon: 'building' },
        { value: 'team', label: 'Team', icon: 'user' },
        { value: 'billing', label: 'Billing', icon: 'tag' },
      ]} />

      {tab === 'me' && (
        <div className="zk-grid" style={{ gridTemplateColumns: '1fr 1fr', gap: 16 }}>
          <Card title="Profile" icon="user">
            <div className="zk-stack" style={{ gap: 14 }}>
              <div className="zk-row" style={{ gap: 14 }}>
                <div style={{ width: 56, height: 56, borderRadius: 12, background: 'var(--accent-soft-2)', color: 'var(--accent-2)', fontSize: 22, fontWeight: 700, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>JK</div>
                <button className="zk-btn zk-btn--outline zk-btn--sm">Upload photo</button>
              </div>
              <Field label="Full name"><input className="zk-input" defaultValue="Jonas Konermann" /></Field>
              <Field label="Email"><input className="zk-input" defaultValue="jonas@konermann.online" /></Field>
            </div>
          </Card>

          <Card title="Localization" icon="settings">
            <div className="zk-stack" style={{ gap: 14 }}>
              <Field label="Timezone">
                <select className="zk-select" defaultValue="Europe/Berlin">
                  <option>Europe/Berlin</option><option>Europe/London</option><option>America/New_York</option>
                </select>
              </Field>
              <Field label="Date format">
                <select className="zk-select" defaultValue="iso"><option value="iso">2026-04-10 (ISO 8601)</option><option>04/10/2026</option><option>10.04.2026</option></select>
              </Field>
              <Field label="Language">
                <select className="zk-select"><option>English</option><option>Deutsch</option></select>
              </Field>
              <Field label="Theme">
                <Segmented value="dark" onChange={()=>{}} options={[{value:'light',label:'Light'},{value:'dark',label:'Dark'},{value:'auto',label:'Auto'}]} />
              </Field>
            </div>
          </Card>

          <Card title="Notifications" icon="bell">
            <div className="zk-stack" style={{ gap: 12 }}>
              {[
                ['Daily digest', 'Summary of yesterday\'s tracked time, every morning at 9.', true],
                ['Idle reminder', 'Nudge me when a timer\'s been running > 4h.', true],
                ['Weekly review', 'Friday afternoon recap with billable totals.', false],
              ].map(([t, d, on]) => (
                <div key={t} className="zk-row-spaced" style={{ padding: '6px 0' }}>
                  <div>
                    <div style={{ fontSize: 13, fontWeight: 600 }}>{t}</div>
                    <div className="zk-text-dim" style={{ fontSize: 12 }}>{d}</div>
                  </div>
                  <span className="zk-pill" style={{ color: on ? 'var(--accent-2)' : 'var(--text-3)' }}>
                    <span className="zk-pill-dot" style={{ background: on ? 'var(--accent-2)' : 'var(--text-4)' }} />
                    {on ? 'On' : 'Off'}
                  </span>
                </div>
              ))}
            </div>
          </Card>

          <Card title="Security" icon="user">
            <div className="zk-stack" style={{ gap: 12 }}>
              <div className="zk-row-spaced">
                <div>
                  <div style={{ fontSize: 13, fontWeight: 600 }}>Password</div>
                  <div className="zk-text-dim" style={{ fontSize: 12 }}>Last changed 4 months ago</div>
                </div>
                <button className="zk-btn zk-btn--outline zk-btn--sm">Change</button>
              </div>
              <div className="zk-row-spaced">
                <div>
                  <div style={{ fontSize: 13, fontWeight: 600 }}>Two-factor auth</div>
                  <div className="zk-text-dim" style={{ fontSize: 12 }}>Authenticator app · enabled</div>
                </div>
                <span className="zk-pill" style={{ color: 'var(--accent-2)' }}><span className="zk-pill-dot"/>Active</span>
              </div>
              <div className="zk-row-spaced">
                <div>
                  <div style={{ fontSize: 13, fontWeight: 600 }}>Sessions</div>
                  <div className="zk-text-dim" style={{ fontSize: 12 }}>2 active devices</div>
                </div>
                <button className="zk-btn zk-btn--outline zk-btn--sm">Manage</button>
              </div>
            </div>
          </Card>

          <div style={{ gridColumn: 'span 2' }}>
            <button className="zk-btn zk-btn--primary"><Icon name="save" size={13} /> Save changes</button>
          </div>
        </div>
      )}

      {tab === 'ws' && (
        <div className="zk-grid" style={{ gridTemplateColumns: '1fr 1fr', gap: 16 }}>
          <Card title="Workspace" icon="building">
            <div className="zk-stack" style={{ gap: 14 }}>
              <Field label="Workspace name"><input className="zk-input" defaultValue="main" /></Field>
              <Field label="Slug" hint="zeitrak.app/main"><input className="zk-input" defaultValue="main" /></Field>
              <Field label="Timezone"><select className="zk-select"><option>Europe/Berlin</option></select></Field>
              <Field label="Week starts on"><select className="zk-select"><option>Monday</option><option>Sunday</option></select></Field>
            </div>
          </Card>

          <Card title="Billing & rates" icon="tag">
            <div className="zk-stack" style={{ gap: 14 }}>
              <Field label="Currency"><select className="zk-select"><option>EUR — Euro</option><option>USD — Dollar</option></select></Field>
              <Field label="Default hourly rate"><input className="zk-input" defaultValue="€ 120" /></Field>
              <Field label="Rounding"><select className="zk-select"><option>None</option><option>Nearest 5 min</option><option>Nearest 15 min</option></select></Field>
              <div className="zk-row-spaced" style={{ paddingTop: 6, borderTop: '1px solid var(--border-soft)' }}>
                <div>
                  <div style={{ fontSize: 13, fontWeight: 600 }}>Plan: Pro</div>
                  <div className="zk-text-dim" style={{ fontSize: 12 }}>Renews May 28 · €15/month</div>
                </div>
                <button className="zk-btn zk-btn--outline zk-btn--sm">Manage</button>
              </div>
            </div>
          </Card>

          <Card title="Danger zone" icon="trash" style={{ borderColor: 'rgba(239,68,68,0.18)' }}>
            <div className="zk-stack" style={{ gap: 12 }}>
              <div className="zk-row-spaced">
                <div>
                  <div style={{ fontSize: 13, fontWeight: 600 }}>Export all data</div>
                  <div className="zk-text-dim" style={{ fontSize: 12 }}>Download a CSV of every entry, activity, and tag.</div>
                </div>
                <button className="zk-btn zk-btn--outline zk-btn--sm"><Icon name="download" size={12} /> Export</button>
              </div>
              <div className="zk-row-spaced">
                <div>
                  <div style={{ fontSize: 13, fontWeight: 600, color: 'var(--danger)' }}>Delete workspace</div>
                  <div className="zk-text-dim" style={{ fontSize: 12 }}>Permanent. All entries and members will be removed.</div>
                </div>
                <button className="zk-btn zk-btn--danger zk-btn--sm"><Icon name="trash" size={12} /> Delete</button>
              </div>
            </div>
          </Card>

          <div style={{ gridColumn: 'span 2' }}>
            <button className="zk-btn zk-btn--primary"><Icon name="save" size={13} /> Save workspace</button>
          </div>
        </div>
      )}

      {tab === 'team' && (
        <Card title="Members" icon="user" action={<button className="zk-btn zk-btn--primary zk-btn--sm"><Icon name="plus" size={12}/> Invite</button>} flush>
          <table className="zk-table">
            <thead><tr><th>Name</th><th>Role</th><th>Tracked this week</th><th></th></tr></thead>
            <tbody>
              {[
                ['Jonas K.','jonas@konermann.online','Owner','31h 48m'],
                ['Mira S.','mira@studio.io','Member','24h 02m'],
                ['Tomás L.','tomas@studio.io','Member','18h 12m'],
                ['Eva R.','eva@studio.io','Viewer','—'],
              ].map(r => (
                <tr key={r[0]}>
                  <td>
                    <div className="zk-row" style={{ gap: 10 }}>
                      <div style={{ width: 28, height: 28, borderRadius: '50%', background: 'var(--surface-3)', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 11, fontWeight: 600 }}>{r[0].split(' ').map(p => p[0]).join('')}</div>
                      <div>
                        <div style={{ fontWeight: 600 }}>{r[0]}</div>
                        <div className="zk-text-dim" style={{ fontSize: 11.5 }}>{r[1]}</div>
                      </div>
                    </div>
                  </td>
                  <td><span className="zk-pill">{r[2]}</span></td>
                  <td className="zk-cell-num">{r[3]}</td>
                  <td className="zk-cell-actions"><button className="zk-icon-action"><Icon name="moreHoriz" size={14}/></button></td>
                </tr>
              ))}
            </tbody>
          </table>
        </Card>
      )}

      {tab === 'billing' && (
        <Card title="Plan & invoices" icon="tag">
          <div className="zk-text-dim">Billing details, invoice history and seat management would live here.</div>
        </Card>
      )}
    </div>
  );
};

Object.assign(window, { ActivitiesScreen, TagsScreen, SettingsScreen });
