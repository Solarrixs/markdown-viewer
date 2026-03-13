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
  return `${days}d${suffix}`;
}
