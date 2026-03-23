export function timeAgo(dateStr: string | null, verbose = false): string {
  if (!dateStr) return '';
  const diff = Date.now() - new Date(dateStr).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return verbose ? 'just now' : 'now';
  const suffix = verbose ? ' ago' : '';
  if (mins < 60) return `${mins}m${suffix}`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h${suffix}`;
  const days = Math.floor(hours / 24);
  if (days < 7) return `${days}d${suffix}`;
  return verbose ? new Date(dateStr).toLocaleDateString() : `${days}d`;
}

export function timeUntil(dateStr: string | null): string {
  if (!dateStr) return '';
  const diff = new Date(dateStr).getTime() - Date.now();
  if (diff <= 0) return 'now';
  const mins = Math.floor(diff / 60000);
  if (mins < 60) return `in ${mins}m`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `in ${hours}h`;
  const days = Math.floor(hours / 24);
  return `in ${days}d`;
}

export function reminderIsoString(date: Date): string {
  return date.toISOString().split('.')[0];
}

function offsetFromNow(n: number, unit: string): Date {
  const d = new Date();
  if (unit === 'm') d.setMinutes(d.getMinutes() + n);
  else if (unit === 'h') d.setHours(d.getHours() + n);
  else if (unit === 'd') d.setDate(d.getDate() + n);
  else if (unit === 'w') d.setDate(d.getDate() + n * 7);
  return d;
}

const UNIT_PATTERN = '(m|min|mins|minutes?|h|hr|hrs|hours?|d|days?|w|wk|wks|weeks?)';
const relativeRe = new RegExp(`^(?:in\\s+)?(\\d+)\\s*${UNIT_PATTERN}$`);

export function parseNaturalTime(input: string): Date | null {
  const s = input.trim().toLowerCase();
  if (!s) return null;

  // Relative: "30m", "2h", "1d", "in 2 hours", etc.
  const relMatch = s.match(relativeRe);
  if (relMatch) {
    return offsetFromNow(parseInt(relMatch[1]), relMatch[2][0]);
  }

  // Named times
  if (s === 'tonight') {
    const d = new Date();
    d.setHours(20, 0, 0, 0);
    if (d.getTime() < Date.now()) d.setDate(d.getDate() + 1);
    return d;
  }
  if (s === 'tomorrow' || s === 'tomorrow morning' || s === 'tmr' || s === 'tmrw') {
    const d = new Date();
    d.setDate(d.getDate() + 1);
    d.setHours(9, 0, 0, 0);
    return d;
  }
  if (s === 'tomorrow afternoon' || s === 'tmr afternoon') {
    const d = new Date();
    d.setDate(d.getDate() + 1);
    d.setHours(14, 0, 0, 0);
    return d;
  }
  if (s === 'tomorrow evening' || s === 'tmr evening' || s === 'tomorrow night') {
    const d = new Date();
    d.setDate(d.getDate() + 1);
    d.setHours(20, 0, 0, 0);
    return d;
  }
  if (s === 'next week') {
    const d = new Date();
    d.setDate(d.getDate() + 7);
    d.setHours(9, 0, 0, 0);
    return d;
  }
  if (s === 'next month') {
    const d = new Date();
    d.setMonth(d.getMonth() + 1);
    d.setHours(9, 0, 0, 0);
    return d;
  }

  // Day names: "monday", "tuesday", etc. → next occurrence at 9 AM
  const days = ['sunday', 'monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday'];
  const dayIdx = days.indexOf(s);
  if (dayIdx !== -1) {
    const d = new Date();
    const today = d.getDay();
    let diff = dayIdx - today;
    if (diff <= 0) diff += 7;
    d.setDate(d.getDate() + diff);
    d.setHours(9, 0, 0, 0);
    return d;
  }

  return null;
}

export interface ReminderOption {
  label: string;
  getTime: () => string;
}

export const reminderOptions: ReminderOption[] = [
  { label: 'Tonight (8 PM)', getTime: () => reminderIsoString(parseNaturalTime('tonight')!) },
  { label: 'Tomorrow (9 AM)', getTime: () => reminderIsoString(parseNaturalTime('tomorrow')!) },
  {
    label: 'In 3 days',
    getTime: () => {
      const d = offsetFromNow(3, 'd');
      d.setHours(9, 0, 0, 0);
      return reminderIsoString(d);
    },
  },
  { label: 'Next week', getTime: () => reminderIsoString(parseNaturalTime('next week')!) },
];

export function formatReminderPreview(d: Date): string {
  const relative = timeUntil(d.toISOString());
  const timeStr = d.toLocaleString('en-US', {
    weekday: 'short', month: 'short', day: 'numeric',
    hour: 'numeric', minute: '2-digit',
  });
  return `${timeStr} (${relative})`;
}
