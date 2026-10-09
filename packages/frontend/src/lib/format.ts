const DATE_TIME = new Intl.DateTimeFormat('en-GB', {
  dateStyle: 'medium',
  timeStyle: 'short',
  timeZone: 'UTC',
});

const NUMBER = new Intl.NumberFormat('en-GB', { maximumFractionDigits: 4 });

/** Format an ISO 8601 timestamp for display, always in UTC. */
export const formatDateTime = (value: string | null | undefined): string => {
  if (!value) {
    return '—';
  }
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return '—';
  }
  return `${DATE_TIME.format(parsed)} UTC`;
};

/** Format a duration in seconds using a compact, human-readable form. */
export const formatDuration = (seconds: number | null | undefined): string => {
  if (seconds === null || seconds === undefined || !Number.isFinite(seconds)) {
    return '—';
  }
  if (seconds < 60) {
    return `${NUMBER.format(seconds)} s`;
  }
  const minutes = Math.floor(seconds / 60);
  const remainder = Math.round(seconds % 60);
  return `${minutes} min ${remainder} s`;
};

/** Format a finite number without discarding meaningful precision. */
export const formatNumber = (value: number | null | undefined): string => {
  if (value === null || value === undefined || !Number.isFinite(value)) {
    return '—';
  }
  return NUMBER.format(value);
};

/** Format a byte count using binary units. */
export const formatBytes = (bytes: number | null | undefined): string => {
  if (bytes === null || bytes === undefined || !Number.isFinite(bytes)) {
    return '—';
  }
  const units = ['B', 'KiB', 'MiB', 'GiB'];
  let value = bytes;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  return `${NUMBER.format(value)} ${units[unitIndex]}`;
};

/** Format a 64-character SHA-256 digest for compact display. */
export const formatChecksum = (checksum: string | null | undefined): string => {
  if (!checksum) {
    return '—';
  }
  return checksum.length <= 16 ? checksum : `${checksum.slice(0, 12)}…`;
};
