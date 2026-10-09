import { describe, expect, it } from 'vitest';

import { canManageMembers, canWriteContent } from '../roles';

describe('roles', () => {
  it('checks write permissions', () => {
    expect(canWriteContent('owner')).toBe(true);
    expect(canWriteContent('researcher')).toBe(true);
    expect(canWriteContent('viewer')).toBe(false);
  });

  it('checks member management permissions', () => {
    expect(canManageMembers('owner')).toBe(true);
    expect(canManageMembers('researcher')).toBe(false);
    expect(canManageMembers('viewer')).toBe(false);
  });
});
