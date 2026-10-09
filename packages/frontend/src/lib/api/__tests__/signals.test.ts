import { describe, expect, it, vi } from 'vitest';

import { ApiClient } from '@/lib/api/client';
import { fetchSignalWindow } from '@/lib/api/signals';

const jsonResponse = (body: unknown, status = 200): Response => {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  });
};

const makeClient = (fetchImpl: typeof fetch) => {
  return new ApiClient({ baseUrl: 'http://api.test/api/v1', fetchImpl, getToken: () => null });
};

const lastCall = (fetchMock: typeof fetch) => {
  const calls = (fetchMock as unknown as ReturnType<typeof vi.fn>).mock.calls;
  return calls[calls.length - 1] as [string, RequestInit];
};

describe('signals API', () => {
  it('fetches a signal window with channels joined by commas', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ channels: [], samples: [] })) as unknown as typeof fetch;

    await fetchSignalWindow(
      'org-1',
      'd1',
      { startSeconds: 0, endSeconds: 5, channels: [0, 2] },
      makeClient(fetchMock),
    );

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe(
      'http://api.test/api/v1/organisations/org-1/datasets/d1/signals?start_seconds=0&end_seconds=5&channels=0%2C2',
    );
    expect(init.method).toBe('GET');
  });

  it('omits the channels query when no channels are selected', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ channels: [], samples: [] })) as unknown as typeof fetch;

    await fetchSignalWindow(
      'org-1',
      'd1',
      { startSeconds: 1.5, endSeconds: 3 },
      makeClient(fetchMock),
    );

    const [url] = lastCall(fetchMock);
    expect(url).toBe(
      'http://api.test/api/v1/organisations/org-1/datasets/d1/signals?start_seconds=1.5&end_seconds=3',
    );
  });
});
