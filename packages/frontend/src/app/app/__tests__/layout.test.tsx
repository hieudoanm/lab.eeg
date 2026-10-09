import { screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/trpc/server', async () => {
  const { buildServerMock } = await import('@/test/server-mock');
  return buildServerMock();
});

vi.mock('@/components/app-shell', () => ({
  AppShell: ({ children, topBar }: { children: React.ReactNode; topBar: React.ReactNode }) => (
    <div>
      {topBar}
      <main>{children}</main>
    </div>
  ),
}));

vi.mock('@/components/molecules/top-bar', () => ({
  TopBar: ({
    userName,
    activeOrganisationId,
  }: {
    userName: string;
    activeOrganisationId: string | null;
  }) => (
    <header data-testid='top-bar'>
      <span data-testid='user-name'>{userName}</span>
      <span data-testid='active-org'>{activeOrganisationId ?? 'none'}</span>
    </header>
  ),
}));

import AppLayout from '@/app/app/layout';
import { requireSession, resetServerMock } from '@/test/server-mock';
import { renderServer } from '@/test/server-render';

const render = () =>
  renderServer(
    <AppLayout params={Promise.resolve({})}>
      <p>page body</p>
    </AppLayout>,
  );

beforeEach(() => {
  vi.clearAllMocks();
  resetServerMock();
});

describe('AppLayout', () => {
  it('renders the shell with the session top bar', async () => {
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
      membership: {
        membership_id: 'm1',
        organisation_id: 'org-1',
        organisation_name: 'Org',
        role: 'owner',
      },
    });

    await render();

    expect(screen.getByText('page body')).toBeInTheDocument();
    expect(screen.getByTestId('user-name')).toHaveTextContent('Ada');
    expect(screen.getByTestId('active-org')).toHaveTextContent('org-1');
  });

  it('falls back to the email then a generic account name and no active organisation', async () => {
    requireSession.mockResolvedValue({
      user: {
        id: 'u1',
        display_name: null,
        email: 'fallback@example.org',
        memberships: [],
      },
      membership: null,
    });

    await render();

    expect(screen.getByTestId('user-name')).toHaveTextContent('fallback@example.org');
    expect(screen.getByTestId('active-org')).toHaveTextContent('none');
  });

  it('renders a generic account name when neither name nor email is present', async () => {
    requireSession.mockResolvedValue({
      user: { id: 'u1', display_name: null, email: null, memberships: [] },
      membership: null,
    });

    await render();

    expect(screen.getByTestId('user-name')).toHaveTextContent('Account');
  });
});
