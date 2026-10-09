import type { MembershipRole } from '@/lib/api/types';

/**
 * Roles allowed to create, update, and archive organisation content.
 *
 * The frontend uses this only to hide unavailable actions; the server always
 * re-checks permissions (docs/04-features.md §4, AGENTS.md §7.6).
 */
export const canWriteContent = (role: MembershipRole): boolean => {
  return role === 'owner' || role === 'researcher';
};

/** Roles allowed to manage organisation members. */
export const canManageMembers = (role: MembershipRole): boolean => {
  return role === 'owner';
};
