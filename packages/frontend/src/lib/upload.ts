/**
 * Client-side helpers for the recording upload flow.
 *
 * The browser uploads the recording bytes to the Next.js route handler at
 * `/api/uploads/content`, which forwards them to the API with the session
 * bearer token. Server actions carry only metadata (their bodies are capped at
 * 1 MB), so the file never travels through a server action.
 */

/** Maximum recording size in bytes (mirrors the backend default). */
export const MAX_UPLOAD_BYTES = (() => {
  const configured = Number.parseInt(process.env.NEXT_PUBLIC_MAX_UPLOAD_BYTES ?? '', 10);
  return Number.isFinite(configured) && configured > 0 ? configured : 268_435_456;
})();

/** Recording formats the UI accepts (mirrors the backend contract). */
export const ACCEPTED_RECORDING_EXTENSIONS = ['.npy'] as const;

export const ACCEPTED_RECORDING_ATTRIBUTE = ACCEPTED_RECORDING_EXTENSIONS.join(',');

const extensionOf = (fileName: string): string => {
  const dot = fileName.lastIndexOf('.');
  return dot === -1 ? '' : fileName.slice(dot).toLowerCase();
};

/**
 * Validate a chosen recording before any network work happens.
 *
 * Returns a user-facing message, or `null` when the file is acceptable. The
 * server re-validates every upload regardless.
 */
export const validateRecordingFile = (file: File, maxBytes = MAX_UPLOAD_BYTES): string | null => {
  const extension = extensionOf(file.name);
  if (!ACCEPTED_RECORDING_EXTENSIONS.some((accepted) => accepted === extension)) {
    return `Only ${ACCEPTED_RECORDING_EXTENSIONS.join(', ')} recordings are supported.`;
  }
  if (file.size <= 0) {
    return 'The selected file is empty.';
  }
  if (file.size > maxBytes) {
    return 'The selected file exceeds the maximum upload size.';
  }
  return null;
};

/** Split a textarea value on newlines or commas into clean, non-empty entries. */
export const parseChannelList = (value: string): string[] => {
  return value
    .split(/[\n,]/)
    .map((entry) => entry.trim())
    .filter((entry) => entry.length > 0);
};

export interface UploadProgressOptions {
  onProgress?: (percent: number) => void;
  signal?: AbortSignal;
}

export interface UploadResult {
  ok: boolean;
  status: number;
  message: string | null;
}

const extractErrorMessage = (body: string): string | null => {
  try {
    const parsed: unknown = JSON.parse(body);
    if (typeof parsed === 'object' && parsed !== null) {
      const error = (parsed as { error?: { message?: unknown } }).error;
      if (error && typeof error.message === 'string' && error.message.length > 0) {
        return error.message;
      }
    }
  } catch {
    // Non-JSON bodies carry no safe message.
  }
  return null;
};

/**
 * Upload a recording with progress reporting via `XMLHttpRequest`.
 *
 * Resolves with the outcome; it only rejects never — network and abort failures
 * are surfaced as an unsuccessful {@link UploadResult} so callers handle one
 * shape.
 */
export const uploadRecording = (
  url: string,
  file: File,
  options: UploadProgressOptions = {},
): Promise<UploadResult> => {
  return new Promise((resolve) => {
    const xhr = new XMLHttpRequest();
    xhr.open('PUT', url);
    xhr.setRequestHeader('Content-Type', 'application/octet-stream');

    if (options.onProgress) {
      const report = options.onProgress;
      xhr.upload.onprogress = (event) => {
        if (event.lengthComputable && event.total > 0) {
          report(Math.min(100, Math.round((event.loaded / event.total) * 100)));
        }
      };
    }

    xhr.onload = () => {
      resolve({
        ok: xhr.status >= 200 && xhr.status < 300,
        status: xhr.status,
        message: extractErrorMessage(xhr.responseText),
      });
    };
    xhr.onerror = () =>
      resolve({ ok: false, status: 0, message: 'The server could not be reached.' });
    xhr.onabort = () => resolve({ ok: false, status: 0, message: 'The upload was cancelled.' });

    if (options.signal) {
      if (options.signal.aborted) {
        xhr.abort();
        return;
      }
      options.signal.addEventListener('abort', () => xhr.abort(), { once: true });
    }

    xhr.send(file);
  });
};
