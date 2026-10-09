import { describe, expect, it, vi } from 'vitest';

import { ApiClient } from '@/lib/api/client';
import { registerDataset, fetchDataset, fetchDatasets } from '@/lib/api/datasets';
import {
  archiveStudy,
  createStudy,
  fetchStudies,
  fetchStudy,
  updateStudy,
} from '@/lib/api/studies';

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

describe('studies API', () => {
  it('lists studies with search and status filters', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(
        jsonResponse({ items: [], next_cursor: null, has_more: false }),
      ) as unknown as typeof fetch;

    await fetchStudies('org-1', { search: 'theta', status: 'active' }, makeClient(fetchMock));

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe(
      'http://api.test/api/v1/organisations/org-1/studies?search=theta&status=active',
    );
    expect(init.method).toBe('GET');
  });

  it('reads a single study', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ id: 's1' })) as unknown as typeof fetch;

    await fetchStudy('org-1', 's1', makeClient(fetchMock));

    const [url] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/organisations/org-1/studies/s1');
  });

  it('creates a study with a JSON body', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ id: 's1' }, 201)) as unknown as typeof fetch;

    await createStudy('org-1', { name: 'Pilot', description: null }, makeClient(fetchMock));

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/organisations/org-1/studies');
    expect(init.method).toBe('POST');
    expect(init.body).toBe(JSON.stringify({ name: 'Pilot', description: null }));
  });

  it('archives a study', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ id: 's1', status: 'archived' })) as unknown as typeof fetch;

    await archiveStudy('org-1', 's1', makeClient(fetchMock));

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/organisations/org-1/studies/s1/archive');
    expect(init.method).toBe('POST');
  });

  it('updates a study with a PATCH body', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ id: 's1', name: 'Renamed' })) as unknown as typeof fetch;

    await updateStudy('org-1', 's1', { name: 'Renamed', description: null }, makeClient(fetchMock));

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/organisations/org-1/studies/s1');
    expect(init.method).toBe('PATCH');
    expect(init.body).toBe(JSON.stringify({ name: 'Renamed', description: null }));
  });
});

describe('datasets API', () => {
  it('lists datasets within a study', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(
        jsonResponse({ items: [], next_cursor: null, has_more: false }),
      ) as unknown as typeof fetch;

    await fetchDatasets('org-1', 's1', {}, makeClient(fetchMock));

    const [url] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/organisations/org-1/studies/s1/datasets');
  });

  it('registers a dataset with a JSON body', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ id: 'd1' }, 201)) as unknown as typeof fetch;

    await registerDataset('org-1', 's1', { name: 'Session 1' }, makeClient(fetchMock));

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/organisations/org-1/studies/s1/datasets');
    expect(init.method).toBe('POST');
    expect(init.body).toBe(JSON.stringify({ name: 'Session 1' }));
  });

  it('reads a single dataset', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ id: 'd1' })) as unknown as typeof fetch;

    await fetchDataset('org-1', 'd1', makeClient(fetchMock));

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/organisations/org-1/datasets/d1');
    expect(init.method).toBe('GET');
  });
});
