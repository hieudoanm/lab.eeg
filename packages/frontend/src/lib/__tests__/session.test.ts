import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('server-only', () => ({}));

const { cookiesGet } = vi.hoisted(() => ({ cookiesGet: vi.fn() }));

vi.mock('next/headers', () => ({
  cookies: async () => ({ get: cookiesGet }),
}));

import {
  ORGANISATION_COOKIE_NAME,
  ORGANISATION_COOKIE_OPTIONS,
  SESSION_COOKIE_NAME,
  SESSION_COOKIE_OPTIONS,
  getSessionToken,
  selectActiveMembership,
} from '@/lib/session';
import type { CurrentUser, MembershipSummary } from '@/lib/api/types';

const memberships: MembershipSummary[] = [
  { membership_id: 'm1', organisation_id: 'org-1', organisation_name: 'One', role: 'owner' },
  { membership_id: 'm2', organisation_id: 'org-2', organisation_name: 'Two', role: 'viewer' },
];

const user = (overrides: Partial<CurrentUser> = {}): CurrentUser => ({
  id: 'u1',
  display_name: 'Ada',
  email: 'ada@example.org',
  memberships,
  ...overrides,
});

beforeEach(() => {
  vi.clearAllMocks();
});

describe('session constants', () => {
  it('exposes stable cookie names', () => {
    expect(SESSION_COOKIE_NAME).toBe('eeg-lab_session');
    expect(ORGANISATION_COOKIE_NAME).toBe('eeg-lab_organisation');
  });

  it('protects both cookies with HttpOnly, Lax and the root path', () => {
    for (const options of [SESSION_COOKIE_OPTIONS, ORGANISATION_COOKIE_OPTIONS]) {
      expect(options).toMatchObject({ httpOnly: true, sameSite: 'lax', path: '/' });
    }
  });
});

describe('getSessionToken', () => {
  it('returns the session cookie value', async () => {
    cookiesGet.mockReturnValue({ value: 'token-123' });

    await expect(getSessionToken()).resolves.toBe('token-123');
    expect(cookiesGet).toHaveBeenCalledWith(SESSION_COOKIE_NAME);
  });

  it('returns null when the cookie is absent', async () => {
    cookiesGet.mockReturnValue(undefined);

    await expect(getSessionToken()).resolves.toBeNull();
  });
});

describe('selectActiveMembership', () => {
  it('returns the only membership when one exists', () => {
    expect(selectActiveMembership(user({ memberships: [memberships[0]] }), null)).toBe(
      memberships[0],
    );
  });

  it('returns null when the user has no memberships', () => {
    expect(selectActiveMembership(user({ memberships: [] }), null)).toBeNull();
  });

  it('returns the preferred membership when it belongs to the user', () => {
    expect(selectActiveMembership(user(), 'org-2')).toBe(memberships[1]);
  });

  it('falls back to the first membership when the preference is unknown', () => {
    expect(selectActiveMembership(user(), 'org-unknown')).toBe(memberships[0]);
  });

  it('falls back to the first membership when no preference is set', () => {
    expect(selectActiveMembership(user(), undefined)).toBe(memberships[0]);
    expect(selectActiveMembership(user(), null)).toBe(memberships[0]);
  });
});
