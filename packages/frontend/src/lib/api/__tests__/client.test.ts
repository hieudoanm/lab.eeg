import { afterEach, describe, expect, it, vi } from 'vitest';

import { ApiClient, DEFAULT_API_BASE_URL, resolveApiBaseUrl } from '@/lib/api/client';
import { ApiError } from '@/lib/api/errors';

const originalApiInternalBaseUrl = process.env.API_INTERNAL_BASE_URL;
const originalPublicApiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL;

afterEach(() => {
  if (originalApiInternalBaseUrl === undefined) {
    delete process.env.API_INTERNAL_BASE_URL;
  } else {
    process.env.API_INTERNAL_BASE_URL = originalApiInternalBaseUrl;
  }
  if (originalPublicApiBaseUrl === undefined) {
    delete process.env.NEXT_PUBLIC_API_BASE_URL;
  } else {
    process.env.NEXT_PUBLIC_API_BASE_URL = originalPublicApiBaseUrl;
  }
  vi.unstubAllGlobals();
});

const jsonResponse = (body: unknown, status = 200): Response => {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  });
};

const makeClient = (
  fetchImpl: typeof fetch,
  getToken: () => string | null = () => null,
  baseUrl = 'http://api.test/api/v1',
) => {
  return new ApiClient({ baseUrl, fetchImpl, getToken });
};

describe('ApiClient', () => {
  it('uses the internal API URL for server-side requests when configured', () => {
    vi.stubGlobal('window', undefined);
    process.env.API_INTERNAL_BASE_URL = 'http://backend:8000/api/v1';

    expect(resolveApiBaseUrl()).toBe('http://backend:8000/api/v1');
  });

  it('ignores the internal URL when running on the server without one', () => {
    vi.stubGlobal('window', undefined);
    delete process.env.API_INTERNAL_BASE_URL;
    process.env.NEXT_PUBLIC_API_BASE_URL = 'http://public.test/api/v1';

    expect(resolveApiBaseUrl()).toBe('http://public.test/api/v1');
  });

  it('falls back to the public API URL in the browser', () => {
    process.env.NEXT_PUBLIC_API_BASE_URL = 'http://public.test/api/v1';

    expect(resolveApiBaseUrl()).toBe('http://public.test/api/v1');
  });

  it('trims the configured public API URL', () => {
    process.env.NEXT_PUBLIC_API_BASE_URL = '  http://public.test/api/v1  ';

    expect(resolveApiBaseUrl()).toBe('http://public.test/api/v1');
  });

  it('uses the default API URL when nothing is configured', () => {
    delete process.env.NEXT_PUBLIC_API_BASE_URL;
    delete process.env.API_INTERNAL_BASE_URL;

    expect(resolveApiBaseUrl()).toBe(DEFAULT_API_BASE_URL);
  });

  it('trims trailing slashes from a custom base URL', async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse({})) as unknown as typeof fetch;
    const client = makeClient(fetchMock, () => null, 'http://api.test/api/v1///');

    await client.get('/me');

    const url = (fetchMock as unknown as ReturnType<typeof vi.fn>).mock.calls[0][0] as string;
    expect(url).toBe('http://api.test/api/v1/me');
  });

  it('returns parsed JSON for a successful request', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ id: 'abc' })) as unknown as typeof fetch;
    const client = makeClient(fetchMock);

    await expect(client.get('/me')).resolves.toEqual({ id: 'abc' });

    const [url, init] = (fetchMock as unknown as ReturnType<typeof vi.fn>).mock.calls[0];
    expect(url).toBe('http://api.test/api/v1/me');
    expect((init as RequestInit).method).toBe('GET');
    expect((init as RequestInit).headers).toMatchObject({ Accept: 'application/json' });
  });

  it('attaches the bearer token when one is available', async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse({})) as unknown as typeof fetch;
    const client = makeClient(fetchMock, () => 'token-123');

    await client.get('/me');

    const init = (fetchMock as unknown as ReturnType<typeof vi.fn>).mock.calls[0][1] as RequestInit;
    expect(init.headers).toMatchObject({ Authorization: 'Bearer token-123' });
  });

  it('includes credentials by default so the session cookie is sent', async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse({})) as unknown as typeof fetch;
    const client = makeClient(fetchMock);

    await client.get('/me');

    const init = (fetchMock as unknown as ReturnType<typeof vi.fn>).mock.calls[0][1] as RequestInit;
    expect(init.credentials).toBe('include');
  });

  it('allows a request to override credentials', async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse({})) as unknown as typeof fetch;
    const client = makeClient(fetchMock);

    await client.get('/me', { credentials: 'omit' });

    const init = (fetchMock as unknown as ReturnType<typeof vi.fn>).mock.calls[0][1] as RequestInit;
    expect(init.credentials).toBe('omit');
  });

  it('serialises JSON bodies, content type, and idempotency keys', async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse({}, 202)) as unknown as typeof fetch;
    const client = makeClient(fetchMock);

    await client.post('/organisations/1/datasets/2/analyses', {
      body: { analysis_type: 'descriptive_statistics' },
      idempotencyKey: 'key-1',
    });

    const init = (fetchMock as unknown as ReturnType<typeof vi.fn>).mock.calls[0][1] as RequestInit;
    expect(init.method).toBe('POST');
    expect(init.body).toBe(JSON.stringify({ analysis_type: 'descriptive_statistics' }));
    expect(init.headers).toMatchObject({
      'Content-Type': 'application/json',
      'Idempotency-Key': 'key-1',
    });
  });

  it('omits empty query parameters and encodes the rest', async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse({})) as unknown as typeof fetch;
    const client = makeClient(fetchMock);

    await client.get('/studies', { query: { limit: 20, cursor: null, search: '' } });

    const url = (fetchMock as unknown as ReturnType<typeof vi.fn>).mock.calls[0][0] as string;
    expect(url).toBe('http://api.test/api/v1/studies?limit=20');
  });

  it('sends a PATCH request with a JSON body', async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse({})) as unknown as typeof fetch;
    const client = makeClient(fetchMock);

    await client.patch('/studies/1', { body: { name: 'Renamed' } });

    const [url, init] = (fetchMock as unknown as ReturnType<typeof vi.fn>).mock.calls[0] as [
      string,
      RequestInit,
    ];
    expect(url).toBe('http://api.test/api/v1/studies/1');
    expect(init.method).toBe('PATCH');
    expect(init.body).toBe(JSON.stringify({ name: 'Renamed' }));
  });

  it('sends a DELETE request', async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse({})) as unknown as typeof fetch;
    const client = makeClient(fetchMock);

    await client.delete('/studies/1');

    const init = (fetchMock as unknown as ReturnType<typeof vi.fn>).mock.calls[0][1] as RequestInit;
    expect(init.method).toBe('DELETE');
  });

  it('normalises a path without a leading slash', async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse({})) as unknown as typeof fetch;
    const client = makeClient(fetchMock);

    await client.get('me');

    const url = (fetchMock as unknown as ReturnType<typeof vi.fn>).mock.calls[0][0] as string;
    expect(url).toBe('http://api.test/api/v1/me');
  });

  it('returns null for a successful response that is not JSON', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(new Response('plain text', { status: 200 })) as unknown as typeof fetch;
    const client = makeClient(fetchMock);

    await expect(client.get('/me')).resolves.toBeNull();
  });

  it('returns null for a successful response without a content type', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(new Response(null, { status: 200 })) as unknown as typeof fetch;
    const client = makeClient(fetchMock);

    await expect(client.get('/me')).resolves.toBeNull();
  });

  it('returns null when a JSON response body is malformed', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response('not json', {
        status: 200,
        headers: { 'content-type': 'application/json' },
      }),
    ) as unknown as typeof fetch;
    const client = makeClient(fetchMock);

    await expect(client.get('/me')).resolves.toBeNull();
  });

  it('maps the documented error envelope onto ApiError', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      jsonResponse(
        {
          error: {
            code: 'validation_error',
            message: 'The request contains invalid fields.',
            details: [{ field: 'name', code: 'blank', message: 'Name must not be blank.' }],
            request_id: 'req_123',
          },
        },
        422,
      ),
    ) as unknown as typeof fetch;
    const client = makeClient(fetchMock);

    const error = await client.post('/studies', { body: {} }).catch((err: unknown) => err);
    expect(error).toBeInstanceOf(ApiError);
    const apiError = error as ApiError;
    expect(apiError.status).toBe(422);
    expect(apiError.code).toBe('validation_error');
    expect(apiError.requestId).toBe('req_123');
    expect(apiError.details).toHaveLength(1);
  });

  it('ignores malformed error details', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      jsonResponse(
        {
          error: {
            code: 'validation_error',
            message: 'The request contains invalid fields.',
            details: [null, 'nope', 42],
          },
        },
        422,
      ),
    ) as unknown as typeof fetch;
    const client = makeClient(fetchMock);

    const error = (await client.get('/studies').catch((err: unknown) => err)) as ApiError;
    expect(error.details).toEqual([]);
  });

  it('falls back to a safe error when the envelope is missing', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(
        new Response('<html>boom</html>', { status: 500 }),
      ) as unknown as typeof fetch;
    const client = makeClient(fetchMock);

    const error = (await client.get('/studies').catch((err: unknown) => err)) as ApiError;
    expect(error.code).toBe('unexpected_error');
    expect(error.status).toBe(500);
  });

  it('returns undefined for 204 responses', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(new Response(null, { status: 204 })) as unknown as typeof fetch;
    const client = makeClient(fetchMock);

    await expect(client.delete('/studies/1')).resolves.toBeUndefined();
  });

  it('normalises network failures', async () => {
    const fetchMock = vi
      .fn()
      .mockRejectedValue(new TypeError('failed to fetch')) as unknown as typeof fetch;
    const client = makeClient(fetchMock);

    const error = (await client.get('/me').catch((err: unknown) => err)) as ApiError;
    expect(error.code).toBe('network_error');
  });

  it('normalises aborted requests', async () => {
    const fetchMock = vi
      .fn()
      .mockRejectedValue(new DOMException('aborted', 'AbortError')) as unknown as typeof fetch;
    const client = makeClient(fetchMock);

    const error = (await client.get('/me').catch((err: unknown) => err)) as ApiError;
    expect(error.code).toBe('request_cancelled');
  });
});
