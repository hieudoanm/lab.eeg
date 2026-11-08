export function formatResultJson(raw: string): string {
  try {
    return JSON.stringify(JSON.parse(raw), null, 2).slice(0, 4000);
  } catch {
    return raw.slice(0, 4000);
  }
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
}

export function runProgressLabel(run: {
  status: string;
  progress?: number | null;
  progress_message?: string | null;
}): string {
  if (run.status !== 'running' || run.progress == null) return run.status;
  const percent = Math.round(run.progress * 100);
  return run.progress_message ? `${percent}% · ${run.progress_message}` : `${percent}%`;
}
