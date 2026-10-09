import type { ApiErrorDetail } from './types';

interface ErrorEnvelope {
  error?: {
    code?: unknown;
    message?: unknown;
    details?: unknown;
    request_id?: unknown;
  };
}

const isErrorDetail = (value: unknown): value is ApiErrorDetail => {
  if (typeof value !== 'object' || value === null) {
    return false;
  }
  const detail = value as Record<string, unknown>;
  return (
    typeof detail.field === 'string' &&
    typeof detail.code === 'string' &&
    typeof detail.message === 'string'
  );
};

/**
 * A normalised API failure. Error codes come from the documented envelope
 * (docs/07-api-spec.md §4); unexpected shapes fall back to safe defaults.
 */
export class ApiError extends Error {
  readonly status: number;
  readonly code: string;
  readonly details: ApiErrorDetail[];
  readonly requestId: string | null;

  constructor(options: {
    status: number;
    code: string;
    message: string;
    details?: ApiErrorDetail[];
    requestId?: string | null;
  }) {
    super(options.message);
    this.name = 'ApiError';
    this.status = options.status;
    this.code = options.code;
    this.details = options.details ?? [];
    this.requestId = options.requestId ?? null;
  }

  static fromResponse(status: number, body: unknown): ApiError {
    const envelope = (typeof body === 'object' && body !== null ? body : {}) as ErrorEnvelope;
    const error = envelope.error ?? {};
    const rawDetails = Array.isArray(error.details) ? error.details : [];
    return new ApiError({
      status,
      code: typeof error.code === 'string' ? error.code : 'unexpected_error',
      message:
        typeof error.message === 'string' && error.message.length > 0
          ? error.message
          : 'The request could not be completed.',
      details: rawDetails.filter(isErrorDetail),
      requestId: typeof error.request_id === 'string' ? error.request_id : null,
    });
  }

  static network(message = 'The server could not be reached.'): ApiError {
    return new ApiError({ status: 0, code: 'network_error', message });
  }

  static cancelled(): ApiError {
    return new ApiError({
      status: 0,
      code: 'request_cancelled',
      message: 'The request was cancelled.',
    });
  }
}
