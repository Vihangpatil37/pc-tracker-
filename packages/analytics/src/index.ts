// Formatting helpers only — no computation

/** Format seconds into a human-readable duration string (e.g. "2h 15m", "45m 30s") */
export function formatDuration(seconds: number): string {
  if (seconds < 0) return "0s";
  if (seconds < 60) return `${seconds}s`;
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  if (m < 60) return s > 0 ? `${m}m ${s}s` : `${m}m`;
  const h = Math.floor(m / 60);
  const rem = m % 60;
  return rem > 0 ? `${h}h ${rem}m` : `${h}h`;
}

/** Format a Unix timestamp (seconds) to a localized time string */
export function formatTime(ts: number): string {
  return new Date(ts * 1000).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** Format a Unix timestamp (seconds) to a localized date string */
export function formatDate(ts: number): string {
  return new Date(ts * 1000).toLocaleDateString([], {
    month: "short",
    day: "numeric",
  });
}

/** Format percentage from a fraction */
export function formatPercentage(part: number, total: number): string {
  if (total === 0) return "0%";
  return `${Math.round((part / total) * 100)}%`;
}

/** Clean an executable name (e.g. "chrome.exe" → "chrome") */
export function cleanExeName(exe: string): string {
  return exe.replace(/\.exe$/i, "");
}
