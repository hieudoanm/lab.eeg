import { describe, expect, it, vi } from 'vitest';

import { ApiClient } from '@/lib/api/client';
import { fetchOrganisationSummary } from '@/lib/api/summary';

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

describe('summary API', () => {
  it('fetches the organisation dashboard summary', async () => {
    const summary = {
      study_count: 4,
      active_study_count: 3,
      dataset_count: 9,
      ready_dataset_count: 7,
      analysis_count: 21,
      analyses_by_status: {
        queued: 1,
        running: 2,
        succeeded: 15,
        failed: 2,
        cancelled: 1,
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse(summary)) as unknown as typeof fetch;

    const result = await fetchOrganisationSummary('org-1', makeClient(fetchMock));

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/organisations/org-1/summary');
    expect(init.method).toBe('GET');
    expect(result).toEqual(summary);
  });
});
