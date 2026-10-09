import 'server-only';

import { cookies } from 'next/headers';

import type { CurrentUser, MembershipSummary } from '@/lib/api/types';

/**
 * Name of the HttpOnly session cookie held by the Next.js BFF.
 *
 * The tRPC layer reads this cookie to authenticate upstream calls to the REST
 * API. It is written by the `identity.signIn` mutation and cleared on sign-out.
 */
export const SESSION_COOKIE_NAME = 'eeg-lab_session';

/** Cookie that remembers which organisation the workspace is scoped to. */
export const ORGANISATION_COOKIE_NAME = 'eeg-lab_organisation';

export const SESSION_COOKIE_OPTIONS = {
  httpOnly: true,
  sameSite: 'lax',
  path: '/',
  secure: process.env.NODE_ENV === 'production',
} as const;

export const ORGANISATION_COOKIE_OPTIONS = {
  httpOnly: true,
  sameSite: 'lax',
  path: '/',
  secure: process.env.NODE_ENV === 'production',
} as const;

/**
 * Read the raw session token, or `null` when the caller is unauthenticated.
 *
 * Intended for server-side proxies (such as the upload route handler) that must
 * forward the bearer token to the API without triggering a redirect.
 */
export const getSessionToken = async (): Promise<string | null> => {
  const store = await cookies();
  return store.get(SESSION_COOKIE_NAME)?.value ?? null;
};

/**
 * Choose the active organisation for a session.
 *
 * The selector cookie is only a preference: if it names an organisation the
 * user is not a member of, the first available membership is used instead, so a
 * tampered value cannot grant access.
 */
export const selectActiveMembership = (
  user: Pick<CurrentUser, 'memberships'>,
  preference: string | null | undefined,
): MembershipSummary | null => {
  if (user.memberships.length <= 1) {
    return user.memberships[0] ?? null;
  }
  if (preference) {
    const match = user.memberships.find((membership) => membership.organisation_id === preference);
    if (match) {
      return match;
    }
  }
  return user.memberships[0] ?? null;
};
