import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const { getSessionToken, resolveApiBaseUrl } = vi.hoisted(() => ({
  getSessionToken: vi.fn(),
  resolveApiBaseUrl: vi.fn(),
}));

vi.mock('@/lib/session', () => ({ getSessionToken }));
vi.mock('@/lib/api/client', () => ({ resolveApiBaseUrl }));

import { PUT } from '@/app/api/uploads/content/route';

const ORG = '11111111-1111-4111-8111-111111111111';
const DATASET = '22222222-2222-4222-8222-222222222222';
const SESSION = '33333333-3333-4333-8333-333333333333';

const makeRequest = (params: Record<string, string>, init?: RequestInit): Request => {
  const query = new URLSearchParams(params).toString();
  return new Request(`http://web.test/api/uploads/content?${query}`, init);
};

beforeEach(() => {
  vi.clearAllMocks();
  resolveApiBaseUrl.mockReturnValue('http://api.test/api/v1/');
  getSessionToken.mockResolvedValue('token-123');
  vi.stubGlobal('fetch', vi.fn());
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('PUT /api/uploads/content', () => {
  it('rejects invalid identifiers', async () => {
    const response = await PUT(
      makeRequest({ organisationId: 'x', datasetId: DATASET, sessionId: SESSION }),
    );

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toMatchObject({
      error: { code: 'invalid_request' },
    });
    expect(globalThis.fetch).not.toHaveBeenCalled();
  });

  it('rejects requests that omit the upload target', async () => {
    const response = await PUT(makeRequest({}));

    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toMatchObject({
      error: { code: 'invalid_request' },
    });
    expect(globalThis.fetch).not.toHaveBeenCalled();
  });

  it('requires an authenticated session', async () => {
    getSessionToken.mockResolvedValue(null);

    const response = await PUT(
      makeRequest({ organisationId: ORG, datasetId: DATASET, sessionId: SESSION }),
    );

    expect(response.status).toBe(401);
    expect(globalThis.fetch).not.toHaveBeenCalled();
  });

  it('streams the body to the API and forwards a 204', async () => {
    (globalThis.fetch as ReturnType<typeof vi.fn>).mockResolvedValue(
      new Response(null, { status: 204 }),
    );

    const response = await PUT(
      makeRequest(
        { organisationId: ORG, datasetId: DATASET, sessionId: SESSION },
        { method: 'PUT', body: 'bytes', headers: { 'content-type': 'application/octet-stream' } },
      ),
    );

    expect(response.status).toBe(204);
    const [url, init] = (globalThis.fetch as ReturnType<typeof vi.fn>).mock.calls[0];
    expect(url).toBe(
      `http://api.test/api/v1/organisations/${ORG}/datasets/${DATASET}/upload-sessions/${SESSION}/content`,
    );
    expect((init as RequestInit).headers).toMatchObject({
      Authorization: 'Bearer token-123',
      'Content-Type': 'application/octet-stream',
    });
  });

  it('forwards non-204 responses with their content type', async () => {
    (globalThis.fetch as ReturnType<typeof vi.fn>).mockResolvedValue(
      new Response(JSON.stringify({ error: { code: 'invalid_recording' } }), {
        status: 422,
        headers: { 'content-type': 'application/json' },
      }),
    );

    const response = await PUT(
      makeRequest({ organisationId: ORG, datasetId: DATASET, sessionId: SESSION }),
    );

    expect(response.status).toBe(422);
    expect(response.headers.get('content-type')).toContain('application/json');
    await expect(response.json()).resolves.toMatchObject({
      error: { code: 'invalid_recording' },
    });
  });

  it('uses a default content type when the request omits one', async () => {
    (globalThis.fetch as ReturnType<typeof vi.fn>).mockResolvedValue(
      new Response(null, { status: 204 }),
    );

    await PUT(makeRequest({ organisationId: ORG, datasetId: DATASET, sessionId: SESSION }));

    const [, init] = (globalThis.fetch as ReturnType<typeof vi.fn>).mock.calls[0];
    expect((init as RequestInit).headers).toMatchObject({
      'Content-Type': 'application/octet-stream',
    });
  });

  it('defaults the response content type when the API omits one', async () => {
    (globalThis.fetch as ReturnType<typeof vi.fn>).mockResolvedValue(
      new Response(null, { status: 500 }),
    );

    const response = await PUT(
      makeRequest({ organisationId: ORG, datasetId: DATASET, sessionId: SESSION }),
    );

    expect(response.status).toBe(500);
    expect(response.headers.get('content-type')).toContain('application/json');
  });

  it('returns 502 when the upstream request fails', async () => {
    (globalThis.fetch as ReturnType<typeof vi.fn>).mockRejectedValue(new Error('network'));

    const response = await PUT(
      makeRequest({ organisationId: ORG, datasetId: DATASET, sessionId: SESSION }),
    );

    expect(response.status).toBe(502);
    await expect(response.json()).resolves.toMatchObject({
      error: { code: 'upload_unavailable' },
    });
  });
});
