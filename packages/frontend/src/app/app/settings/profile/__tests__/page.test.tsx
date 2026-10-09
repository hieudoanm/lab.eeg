import { screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/trpc/server', async () => {
  const { buildServerMock } = await import('@/test/server-mock');
  return buildServerMock();
});

import ProfileSettingsPage from '@/app/app/settings/profile/page';
import { requireSession, resetServerMock } from '@/test/server-mock';
import { renderServer } from '@/test/server-render';

beforeEach(() => {
  vi.clearAllMocks();
  resetServerMock();
});

describe('ProfileSettingsPage', () => {
  it('renders the profile and memberships', async () => {
    requireSession.mockResolvedValue({
      user: {
        id: 'u1',
        display_name: 'Ada',
        email: 'ada@example.org',
        memberships: [
          {
            membership_id: 'm1',
            organisation_id: 'org-1',
            organisation_name: 'Org',
            role: 'owner',
          },
        ],
      },
      membership: null,
    });

    await renderServer(<ProfileSettingsPage />);

    expect(screen.getByRole('heading', { name: 'Profile' })).toBeInTheDocument();
    expect(screen.getByText('Ada')).toBeInTheDocument();
    expect(screen.getByText('ada@example.org')).toBeInTheDocument();
    expect(screen.getByText('Org')).toBeInTheDocument();
  });

  it('falls back for missing details and no memberships', async () => {
    requireSession.mockResolvedValue({
      user: { id: 'u1', display_name: null, email: null, memberships: [] },
      membership: null,
    });

    await renderServer(<ProfileSettingsPage />);

    expect(screen.getAllByText('Not set')).toHaveLength(2);
    expect(screen.getByText(/not a member of any organisation/i)).toBeInTheDocument();
  });
});
