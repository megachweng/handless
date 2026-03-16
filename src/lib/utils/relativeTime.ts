const rtfCache = new Map<string, Intl.RelativeTimeFormat>();
const dtfCache = new Map<string, Intl.DateTimeFormat>();

function getRelativeFormatter(locale: string): Intl.RelativeTimeFormat {
  let fmt = rtfCache.get(locale);
  if (!fmt) {
    fmt = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
    rtfCache.set(locale, fmt);
  }
  return fmt;
}

function getDateFormatter(
  locale: string,
  withYear: boolean,
): Intl.DateTimeFormat {
  const key = `${locale}:${withYear ? "y" : "n"}`;
  let fmt = dtfCache.get(key);
  if (!fmt) {
    fmt = new Intl.DateTimeFormat(locale, {
      month: "short",
      day: "numeric",
      ...(withYear ? { year: "numeric" } : {}),
    });
    dtfCache.set(key, fmt);
  }
  return fmt;
}

/**
 * Format a unix timestamp (seconds) as a human-readable relative time string.
 * Uses Intl.RelativeTimeFormat for locale-aware output.
 * Falls back to a short absolute date for anything older than a week.
 */
export function formatRelativeTime(
  timestampSeconds: number,
  locale: string,
): string {
  const now = Date.now();
  const then = timestampSeconds * 1000;
  const diffSeconds = Math.floor((now - then) / 1000);

  const rtf = getRelativeFormatter(locale);

  if (diffSeconds < 60) {
    // "now" in the user's locale
    return rtf.format(0, "second");
  }
  if (diffSeconds < 3600) {
    return rtf.format(-Math.floor(diffSeconds / 60), "minute");
  }
  if (diffSeconds < 86400) {
    return rtf.format(-Math.floor(diffSeconds / 3600), "hour");
  }
  if (diffSeconds < 604800) {
    return rtf.format(-Math.floor(diffSeconds / 86400), "day");
  }

  // Older than a week: show short date
  const date = new Date(then);
  const sameYear = date.getFullYear() === new Date().getFullYear();

  return getDateFormatter(locale, !sameYear).format(date);
}

/**
 * Format a unix timestamp (seconds) as an absolute datetime string.
 * Used for tooltip display alongside relative timestamps.
 */
export function formatAbsoluteTime(timestampSeconds: number): string {
  const date = new Date(timestampSeconds * 1000);
  const y = date.getFullYear();
  const mo = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  const h = String(date.getHours()).padStart(2, "0");
  const mi = String(date.getMinutes()).padStart(2, "0");
  return `${y}-${mo}-${d} ${h}:${mi}`;
}
