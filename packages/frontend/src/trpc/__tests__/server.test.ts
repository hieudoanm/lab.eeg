import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('server-only', () => ({}));

const { createTRPCContext, fetchCurrentUser, redirect } = vi.hoisted(() => ({
  createTRPCContext: vi.fn(),
  fetchCurrentUser: vi.fn(),
  redirect: vi.fn(),
}));

vi.mock('@/server/api/trpc', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/server/api/trpc')>();
  return { ...actual, createTRPCContext };
});
vi.mock('@/lib/api/identity', () => ({ fetchCurrentUser }));
vi.mock('next/navigation', () => ({ redirect }));

import { ApiError } from '@/lib/api/errors';
import type { CurrentUser } from '@/lib/api/types';
import { getCaller, requireSession } from '@/trpc/server';

const membership = {
  membership_id: 'm1',
  organisation_id: 'org-1',
  organisation_name: 'Org',
  role: 'owner' as const,
};

const user: CurrentUser = {
  id: 'u1',
  display_name: 'Ada',
  email: 'ada@example.org',
  memberships: [membership],
};

const context = (overrides: Partial<{ token: string | null; preference: string | null }> = {}) => ({
  token: overrides.token === undefined ? 'token-123' : overrides.token,
  client: {},
  organisationPreference: overrides.preference ?? null,
  setSessionToken: vi.fn(),
  clearSessionToken: vi.fn(),
  setOrganisationPreference: vi.fn(),
});

beforeEach(() => {
  vi.clearAllMocks();
});

describe('getCaller', () => {
  it('builds a caller from the request context', async () => {
    createTRPCContext.mockResolvedValue(context());

    const caller = await getCaller();

    expect(caller).toBeDefined();
    expect(createTRPCContext).toHaveBeenCalledTimes(1);
  });
});

describe('requireSession', () => {
  it('returns the caller, user and active membership', async () => {
    createTRPCContext.mockResolvedValue(context());
    fetchCurrentUser.mockResolvedValue(user);

    const session = await requireSession();

    expect(session.caller).toBeDefined();
    expect(session.user).toEqual(user);
    expect(session.membership).toEqual(membership);
  });

  it('honours the organisation preference cookie', async () => {
    const second = { ...membership, membership_id: 'm2', organisation_id: 'org-2' };
    createTRPCContext.mockResolvedValue(context({ preference: 'org-2' }));
    fetchCurrentUser.mockResolvedValue({ ...user, memberships: [membership, second] });

    const session = await requireSession();

    expect(session.membership?.organisation_id).toBe('org-2');
  });

  it('redirects to login when there is no session', async () => {
    createTRPCContext.mockResolvedValue(context({ token: null }));
    redirect.mockImplementation(() => {
      throw new Error('NEXT_REDIRECT:/login');
    });

    await expect(requireSession()).rejects.toThrow('NEXT_REDIRECT:/login');

    expect(redirect).toHaveBeenCalledWith('/login');
    expect(fetchCurrentUser).not.toHaveBeenCalled();
  });

  it('redirects to login when the session is rejected upstream', async () => {
    createTRPCContext.mockResolvedValue(context());
    fetchCurrentUser.mockRejectedValue(
      new ApiError({ status: 401, code: 'unauthorized', message: 'nope' }),
    );
    redirect.mockImplementation(() => {
      throw new Error('NEXT_REDIRECT:/login');
    });

    await expect(requireSession()).rejects.toThrow('NEXT_REDIRECT:/login');
  });

  it('propagates unexpected failures', async () => {
    createTRPCContext.mockResolvedValue(context());
    fetchCurrentUser.mockRejectedValue(
      new ApiError({ status: 500, code: 'server_error', message: 'boom' }),
    );

    await expect(requireSession()).rejects.toMatchObject({ code: 'INTERNAL_SERVER_ERROR' });
    expect(redirect).not.toHaveBeenCalled();
  });
});
