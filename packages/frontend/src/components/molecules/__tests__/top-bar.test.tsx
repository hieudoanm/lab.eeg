import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/trpc/react', async () => {
  const { buildTrpcMock } = await import('@/test/trpc-mock');
  return buildTrpcMock();
});

const { push, refresh } = vi.hoisted(() => ({ push: vi.fn(), refresh: vi.fn() }));

vi.mock('next/navigation', () => ({ useRouter: () => ({ push, refresh }) }));

import type { MembershipSummary } from '@/lib/api/types';
import { TopBar } from '@/components/molecules/top-bar';
import { mutationController, resetTrpcMocks } from '@/test/trpc-mock';

const memberships: MembershipSummary[] = [
  { membership_id: 'm1', organisation_id: 'o1', organisation_name: 'Alpha Lab', role: 'owner' },
  {
    membership_id: 'm2',
    organisation_id: 'o2',
    organisation_name: 'Beta Lab',
    role: 'researcher',
  },
];

beforeEach(() => {
  resetTrpcMocks();
  push.mockReset();
  refresh.mockReset();
});

describe('TopBar', () => {
  it('shows the active organisation name when there is a single membership', () => {
    render(
      <TopBar memberships={[memberships[0]]} activeOrganisationId='o1' userName='Ada Lovelace' />,
    );

    expect(screen.getByText('Alpha Lab')).toBeInTheDocument();
    expect(screen.queryByRole('combobox')).not.toBeInTheDocument();
  });

  it('falls back to a placeholder when no organisation is active', () => {
    render(<TopBar memberships={[]} activeOrganisationId={null} userName='Ada Lovelace' />);

    expect(screen.getByText('No organisation')).toBeInTheDocument();
  });

  it('renders the user initial and name', () => {
    render(
      <TopBar memberships={[memberships[0]]} activeOrganisationId='o1' userName='ada lovelace' />,
    );

    expect(screen.getByText('ada lovelace')).toBeInTheDocument();
    expect(screen.getByText('A')).toBeInTheDocument();
  });

  it('reveals the profile link and sign out control from the account menu', async () => {
    const user = userEvent.setup();
    render(
      <TopBar memberships={[memberships[0]]} activeOrganisationId='o1' userName='ada lovelace' />,
    );

    await user.click(screen.getByText('ada lovelace'));

    expect(screen.getByRole('link', { name: 'Profile' })).toHaveAttribute(
      'href',
      '/app/settings/profile',
    );
    expect(screen.getByRole('button', { name: 'Sign out' })).toBeInTheDocument();
  });

  it('renders an organisation switcher and switches organisation', async () => {
    mutationController('identity.selectOrganisation').resolveWith(undefined);
    const user = userEvent.setup();
    render(<TopBar memberships={memberships} activeOrganisationId='o1' userName='Ada' />);

    const select = screen.getByRole('combobox', { name: 'Active organisation' });
    expect(select).toHaveValue('o1');
    expect(screen.getByRole('option', { name: 'Alpha Lab' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Beta Lab' })).toBeInTheDocument();

    await user.selectOptions(select, 'o2');

    await waitFor(() =>
      expect(mutationController('identity.selectOrganisation').calls[0]).toEqual({
        organisationId: 'o2',
      }),
    );
    await waitFor(() => expect(refresh).toHaveBeenCalled());
  });

  it('signs out and navigates to the login page', async () => {
    mutationController('identity.signOut').resolveWith(undefined);
    const user = userEvent.setup();
    render(<TopBar memberships={[memberships[0]]} activeOrganisationId='o1' userName='Ada' />);

    await user.click(screen.getByText('Ada'));
    await user.click(screen.getByRole('button', { name: 'Sign out' }));

    await waitFor(() => expect(mutationController('identity.signOut').calls).toHaveLength(1));
    expect(push).toHaveBeenCalledWith('/login');
    expect(refresh).toHaveBeenCalled();
  });
});
