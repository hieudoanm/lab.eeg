import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { MembershipSummary } from '@/lib/api/types';

vi.mock('@/trpc/react', async () => {
  const { buildTrpcMock } = await import('@/test/trpc-mock');
  return buildTrpcMock();
});

const { refresh } = vi.hoisted(() => ({ refresh: vi.fn() }));

vi.mock('next/navigation', () => ({ useRouter: () => ({ refresh, push: vi.fn() }) }));

import { OrganisationSwitcher } from '@/components/molecules/organisation-switcher';
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
  refresh.mockReset();
});

describe('OrganisationSwitcher', () => {
  it('renders a labelled option per membership', () => {
    render(<OrganisationSwitcher memberships={memberships} activeOrganisationId='o1' />);

    const select = screen.getByRole('combobox', { name: 'Active organisation' });
    expect(select).toHaveValue('o1');
    expect(screen.getAllByRole('option')).toHaveLength(2);
  });

  it('renders an empty control when no organisation is active or available', () => {
    render(<OrganisationSwitcher memberships={[]} activeOrganisationId={null} />);

    expect(screen.getByRole('combobox')).toBeInTheDocument();
    expect(screen.queryAllByRole('option')).toHaveLength(0);
  });

  it('switches organisation and refreshes the route', async () => {
    mutationController('identity.selectOrganisation').resolveWith(undefined);
    const user = userEvent.setup();
    render(<OrganisationSwitcher memberships={memberships} activeOrganisationId='o1' />);

    await user.selectOptions(screen.getByRole('combobox'), 'o2');

    await waitFor(() =>
      expect(mutationController('identity.selectOrganisation').calls[0]).toEqual({
        organisationId: 'o2',
      }),
    );
    await waitFor(() => expect(refresh).toHaveBeenCalled());
  });

  it('disables the control while switching', async () => {
    let resolveSwitch: (value: undefined) => void = () => undefined;
    mutationController('identity.selectOrganisation').resolveWith(
      new Promise<undefined>((resolve) => {
        resolveSwitch = resolve;
      }),
    );
    const user = userEvent.setup();
    render(<OrganisationSwitcher memberships={memberships} activeOrganisationId='o1' />);

    await user.selectOptions(screen.getByRole('combobox'), 'o2');

    await waitFor(() => expect(screen.getByRole('combobox')).toBeDisabled());

    resolveSwitch(undefined);
    await waitFor(() => expect(screen.getByRole('combobox')).not.toBeDisabled());
  });

  it('surfaces a failure and re-enables the control', async () => {
    mutationController('identity.selectOrganisation').rejectWith(new Error('boom'));
    const user = userEvent.setup();
    render(<OrganisationSwitcher memberships={memberships} activeOrganisationId='o1' />);

    await user.selectOptions(screen.getByRole('combobox'), 'o2');

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Could not switch organisation. Please try again.',
    );
    expect(refresh).not.toHaveBeenCalled();
    expect(screen.getByRole('combobox')).not.toBeDisabled();
  });
});
