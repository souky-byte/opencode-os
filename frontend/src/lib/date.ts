const MINUTE = 60 * 1000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;
const WEEK = 7 * DAY;
const MONTH = 30 * DAY;
const YEAR = 365 * DAY;

export function formatDistanceToNow(date: Date): string {
  const now = new Date();
  const diff = now.getTime() - date.getTime();

  if (diff < MINUTE) {
    return "just now";
  }
  if (diff < HOUR) {
    const minutes = Math.floor(diff / MINUTE);
    return `${minutes}m ago`;
  }
  if (diff < DAY) {
    const hours = Math.floor(diff / HOUR);
    return `${hours}h ago`;
  }
  if (diff < WEEK) {
    const days = Math.floor(diff / DAY);
    return `${days}d ago`;
  }
  if (diff < MONTH) {
    const weeks = Math.floor(diff / WEEK);
    return `${weeks}w ago`;
  }
  if (diff < YEAR) {
    const months = Math.floor(diff / MONTH);
    return `${months}mo ago`;
  }

  const years = Math.floor(diff / YEAR);
  return `${years}y ago`;
}

export function formatDate(date: Date): string {
  return date.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

export function formatDateTime(date: Date): string {
  return date.toLocaleString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
}
