import { ApiError } from './errors';
import { getAccessToken } from '@/lib/auth/token';

export const DEFAULT_API_BASE_URL = 'http://localhost:8000/api/v1';

export const resolveApiBaseUrl = (): string => {
  if (typeof window === 'undefined') {
    const internal = process.env.API_INTERNAL_BASE_URL?.trim();
    if (internal) {
      return internal;
    }
  }
  const configured = process.env.NEXT_PUBLIC_API_BASE_URL?.trim();
  return configured && configured.length > 0 ? configured : DEFAULT_API_BASE_URL;
};

export type QueryValue = string | number | boolean | null | undefined;

export interface RequestOptions {
  method?: 'GET' | 'POST' | 'PATCH' | 'DELETE';
  body?: unknown;
  query?: Record<string, QueryValue>;
  signal?: AbortSignal;
  idempotencyKey?: string;
  headers?: Record<string, string>;
  credentials?: RequestCredentials;
}

export interface ApiClientOptions {
  baseUrl?: string;
  getToken?: () => string | null;
  fetchImpl?: typeof fetch;
  credentials?: RequestCredentials;
}

const buildQuery = (query: Record<string, QueryValue> | undefined): string => {
  if (!query) {
    return '';
  }
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(query)) {
    if (value === null || value === undefined || value === '') {
      continue;
    }
    params.set(key, String(value));
  }
  const serialised = params.toString();
  return serialised.length > 0 ? `?${serialised}` : '';
};

/**
 * Typed HTTP client for the EEG Lab API.
 *
 * It normalises errors into {@link ApiError}, attaches bearer tokens when
 * available, and never parses a response body that is not JSON.
 */
export class ApiClient {
  private readonly baseUrl: string;
  private readonly getToken: () => string | null;
  private readonly fetchImpl: typeof fetch;
  private readonly credentials: RequestCredentials;

  constructor(options: ApiClientOptions = {}) {
    this.baseUrl = (options.baseUrl ?? resolveApiBaseUrl()).replace(/\/+$/, '');
    this.getToken = options.getToken ?? getAccessToken;
    this.fetchImpl = options.fetchImpl ?? globalThis.fetch.bind(globalThis);
    this.credentials = options.credentials ?? 'include';
  }

  async request<T>(path: string, options: RequestOptions = {}): Promise<T> {
    const { method = 'GET', body, query, signal, idempotencyKey, headers = {} } = options;
    const url = `${this.baseUrl}${path.startsWith('/') ? path : `/${path}`}${buildQuery(query)}`;

    const requestHeaders: Record<string, string> = { Accept: 'application/json', ...headers };
    const token = this.getToken();
    if (token) {
      requestHeaders.Authorization = `Bearer ${token}`;
    }
    if (body !== undefined) {
      requestHeaders['Content-Type'] = 'application/json';
    }
    if (idempotencyKey) {
      requestHeaders['Idempotency-Key'] = idempotencyKey;
    }

    let response: Response;
    try {
      response = await this.fetchImpl(url, {
        method,
        headers: requestHeaders,
        body: body === undefined ? undefined : JSON.stringify(body),
        signal,
        credentials: options.credentials ?? this.credentials,
      });
    } catch (error) {
      if (error instanceof DOMException && error.name === 'AbortError') {
        throw ApiError.cancelled();
      }
      throw ApiError.network();
    }

    if (response.status === 204) {
      return undefined as T;
    }

    const contentType = response.headers.get('content-type') ?? '';
    const isJson = contentType.includes('application/json');
    const payload: unknown = isJson ? await response.json().catch(() => null) : null;

    if (!response.ok) {
      throw ApiError.fromResponse(response.status, payload);
    }

    return payload as T;
  }

  get<T>(path: string, options: Omit<RequestOptions, 'method' | 'body'> = {}): Promise<T> {
    return this.request<T>(path, { ...options, method: 'GET' });
  }

  post<T>(path: string, options: Omit<RequestOptions, 'method'> = {}): Promise<T> {
    return this.request<T>(path, { ...options, method: 'POST' });
  }

  patch<T>(path: string, options: Omit<RequestOptions, 'method'> = {}): Promise<T> {
    return this.request<T>(path, { ...options, method: 'PATCH' });
  }

  delete<T>(path: string, options: Omit<RequestOptions, 'method' | 'body'> = {}): Promise<T> {
    return this.request<T>(path, { ...options, method: 'DELETE' });
  }
}

export const apiClient = new ApiClient();
