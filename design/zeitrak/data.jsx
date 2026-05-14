// Mock data + state hooks

const ACTIVITIES = [
  { id: 'a1', name: 'Deep work',       color: '#22c55e', comment: 'Focused product work',       tracked: 18 * 3600 + 24 * 60, lastUsed: '2h ago' },
  { id: 'a2', name: 'Customer calls',  color: '#3b82f6', comment: 'Discovery + check-ins',     tracked: 6 * 3600 + 12 * 60,  lastUsed: 'Yesterday' },
  { id: 'a3', name: 'Design review',   color: '#a855f7', comment: 'Cross-team critiques',      tracked: 4 * 3600 + 48 * 60,  lastUsed: 'Mon' },
  { id: 'a4', name: 'Admin & email',   color: '#f59e0b', comment: 'Inbox, scheduling, ops',    tracked: 3 * 3600 + 30 * 60,  lastUsed: '3d ago' },
  { id: 'a5', name: 'Documentation',   color: '#06b6d4', comment: 'Specs, READMEs, handoffs',  tracked: 2 * 3600 + 6 * 60,   lastUsed: 'Last week' },
];

const TAGS = [
  { id: 't1', name: 'billable',  color: '#22c55e', count: 28 },
  { id: 't2', name: 'internal',  color: '#a4a4ae', count: 17 },
  { id: 't3', name: 'urgent',    color: '#ef4444', count: 4  },
  { id: 't4', name: 'meeting',   color: '#3b82f6', count: 12 },
  { id: 't5', name: 'research',  color: '#a855f7', count: 7  },
];

const ENTRIES = [
  { id: 'e1', activity: 'Deep work',      activityId: 'a1', tags: ['billable'],          start: 'Today, 09:14',     duration: 2 * 3600 + 18 * 60, desc: 'Onboarding flow refactor' },
  { id: 'e2', activity: 'Design review',  activityId: 'a3', tags: ['internal'],          start: 'Today, 11:45',     duration: 45 * 60,             desc: 'Q2 design crit' },
  { id: 'e3', activity: 'Customer calls', activityId: 'a2', tags: ['billable','meeting'],start: 'Today, 14:20',     duration: 32 * 60,             desc: 'Acme — kickoff' },
  { id: 'e4', activity: 'Admin & email',  activityId: 'a4', tags: [],                    start: 'Yesterday, 08:30', duration: 48 * 60,             desc: '' },
  { id: 'e5', activity: 'Deep work',      activityId: 'a1', tags: ['billable'],          start: 'Yesterday, 10:00', duration: 3 * 3600 + 22 * 60, desc: 'Timesheet API' },
  { id: 'e6', activity: 'Documentation',  activityId: 'a5', tags: ['internal'],          start: 'Mon, 13:45',       duration: 1 * 3600 + 12 * 60, desc: 'API spec for tags' },
  { id: 'e7', activity: 'Customer calls', activityId: 'a2', tags: ['billable'],          start: 'Mon, 09:00',       duration: 28 * 60,             desc: 'Northwind — review' },
];

// Hours per day for last 7 days (in seconds)
const WEEK_DATA = [
  { day: 'Wed', date: 'May 06', sec: 4 * 3600 + 12 * 60 },
  { day: 'Thu', date: 'May 07', sec: 6 * 3600 + 48 * 60 },
  { day: 'Fri', date: 'May 08', sec: 5 * 3600 + 24 * 60 },
  { day: 'Sat', date: 'May 09', sec: 0 },
  { day: 'Sun', date: 'May 10', sec: 1 * 3600 + 30 * 60 },
  { day: 'Mon', date: 'May 11', sec: 7 * 3600 + 18 * 60 },
  { day: 'Tue', date: 'May 12', sec: 6 * 3600 + 50 * 60 },
];

const WORKSPACES = [
  { id: 'main', name: 'main', sub: 'Curated workspace', accent: '#16a34a', members: 4, lastActive: 'Active now', plan: 'Pro' },
  { id: 'side', name: 'Side projects', sub: 'Personal', accent: '#a855f7', members: 1, lastActive: '2d ago', plan: 'Free' },
  { id: 'cli', name: 'Acme client',  sub: 'External · billable', accent: '#3b82f6', members: 7, lastActive: '5h ago', plan: 'Pro' },
];

Object.assign(window, { ACTIVITIES, TAGS, ENTRIES, WEEK_DATA, WORKSPACES });
