import { describe, expect, it, vi } from 'vitest';

import {
  cancelAnalysis,
  fetchAnalyses,
  fetchOrganisationAnalyses,
  fetchAnalysisJob,
  fetchAnalysisResult,
  submitAnalysis,
  type AnalysisSubmitBody,
} from '@/lib/api/analyses';
import { ApiClient } from '@/lib/api/client';

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

const SUBMIT_BODY: AnalysisSubmitBody = {
  analysis_type: 'descriptive_statistics',
  algorithm_version: '1.0.0',
  parameters: { include_mean: true, include_standard_deviation: true },
};

describe('analyses API', () => {
  it('lists analyses with cursor and limit', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(
        jsonResponse({ items: [], next_cursor: null, has_more: false }),
      ) as unknown as typeof fetch;

    await fetchAnalyses('org-1', 'd1', { cursor: 'c1', limit: 25 }, makeClient(fetchMock));

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe(
      'http://api.test/api/v1/organisations/org-1/datasets/d1/analyses?cursor=c1&limit=25',
    );
    expect(init.method).toBe('GET');
  });

  it('lists analyses without query when no options are given', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ items: [] })) as unknown as typeof fetch;

    await fetchAnalyses('org-1', 'd1', {}, makeClient(fetchMock));

    const [url] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/organisations/org-1/datasets/d1/analyses');
  });

  it('lists organisation analyses with all filters', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(
        jsonResponse({ items: [], next_cursor: null, has_more: false }),
      ) as unknown as typeof fetch;

    await fetchOrganisationAnalyses(
      'org-1',
      { cursor: 'c1', limit: 25, datasetId: 'd1', studyId: 's1', status: 'failed' },
      makeClient(fetchMock),
    );

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe(
      'http://api.test/api/v1/organisations/org-1/analyses?cursor=c1&limit=25&dataset_id=d1&study_id=s1&status=failed',
    );
    expect(init.method).toBe('GET');
  });

  it('lists organisation analyses without query when no options are given', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ items: [] })) as unknown as typeof fetch;

    await fetchOrganisationAnalyses('org-1', {}, makeClient(fetchMock));

    const [url] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/organisations/org-1/analyses');
  });

  it('reads a single analysis job', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ id: 'j1' })) as unknown as typeof fetch;

    await fetchAnalysisJob('org-1', 'j1', makeClient(fetchMock));

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/organisations/org-1/analyses/j1');
    expect(init.method).toBe('GET');
  });

  it('reads an analysis result', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ analysis_id: 'j1' })) as unknown as typeof fetch;

    await fetchAnalysisResult('org-1', 'j1', makeClient(fetchMock));

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/organisations/org-1/analyses/j1/results');
    expect(init.method).toBe('GET');
  });

  it('submits an analysis with an idempotency key and JSON body', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ id: 'j1' }, 202)) as unknown as typeof fetch;

    await submitAnalysis('org-1', 'd1', SUBMIT_BODY, 'key-1', makeClient(fetchMock));

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/organisations/org-1/datasets/d1/analyses');
    expect(init.method).toBe('POST');
    expect(init.body).toBe(JSON.stringify(SUBMIT_BODY));
    expect(init.headers).toMatchObject({
      'Content-Type': 'application/json',
      'Idempotency-Key': 'key-1',
    });
  });

  it('cancels an analysis job', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(
        jsonResponse({ id: 'j1', status: 'cancelled' }),
      ) as unknown as typeof fetch;

    await cancelAnalysis('org-1', 'j1', makeClient(fetchMock));

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/organisations/org-1/analyses/j1/cancel');
    expect(init.method).toBe('POST');
    expect(init.body).toBe('{}');
  });
});
