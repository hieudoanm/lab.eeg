import { describe, expect, it, vi } from 'vitest';

import { ApiClient } from '@/lib/api/client';
import {
  endSession,
  fetchCurrentUser,
  fetchOrganisation,
  fetchOrganisationMembers,
  fetchOrganisations,
  issueDevelopmentToken,
} from '@/lib/api/identity';

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

describe('identity API', () => {
  it('reads the current user from /me', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ id: 'u1' })) as unknown as typeof fetch;

    await fetchCurrentUser(makeClient(fetchMock));

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/me');
    expect(init.method).toBe('GET');
  });

  it('lists the organisations the user belongs to', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ items: [] })) as unknown as typeof fetch;

    await fetchOrganisations(makeClient(fetchMock));

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/organisations');
    expect(init.method).toBe('GET');
  });

  it('reads a single organisation', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ id: 'org-1' })) as unknown as typeof fetch;

    await fetchOrganisation('org-1', makeClient(fetchMock));

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/organisations/org-1');
    expect(init.method).toBe('GET');
  });

  it('requests the members of an organisation', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ items: [] })) as unknown as typeof fetch;

    await fetchOrganisationMembers('org-1', makeClient(fetchMock));

    const [url] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/organisations/org-1/memberships');
  });

  it('mints a development token with the supplied identity', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ access_token: 't' })) as unknown as typeof fetch;

    await issueDevelopmentToken(
      { subject: 'ada@example.org', email: 'ada@example.org' },
      makeClient(fetchMock),
    );

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/auth/token');
    expect(init.method).toBe('POST');
    expect(init.body).toBe(
      JSON.stringify({ subject: 'ada@example.org', email: 'ada@example.org' }),
    );
    expect(init.credentials).toBe('include');
  });

  it('mints a development token with an empty payload by default', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ access_token: 't' })) as unknown as typeof fetch;

    await issueDevelopmentToken(undefined, makeClient(fetchMock));

    const [, init] = lastCall(fetchMock);
    expect(init.body).toBe('{}');
  });

  it('clears the session through /auth/logout', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(new Response(null, { status: 204 })) as unknown as typeof fetch;

    await expect(endSession(makeClient(fetchMock))).resolves.toBeUndefined();

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/auth/logout');
    expect(init.method).toBe('POST');
  });
});
