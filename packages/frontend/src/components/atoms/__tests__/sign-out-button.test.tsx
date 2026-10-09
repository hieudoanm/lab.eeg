import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/trpc/react', async () => {
  const { buildTrpcMock } = await import('@/test/trpc-mock');
  return buildTrpcMock();
});

const { push, refresh } = vi.hoisted(() => ({ push: vi.fn(), refresh: vi.fn() }));

vi.mock('next/navigation', () => ({ useRouter: () => ({ push, refresh }) }));

import { SignOutButton } from '@/components/atoms/sign-out-button';
import { mutationController, resetTrpcMocks } from '@/test/trpc-mock';

beforeEach(() => {
  resetTrpcMocks();
  push.mockReset();
  refresh.mockReset();
});

describe('SignOutButton', () => {
  it('ends the session and navigates to the login page', async () => {
    mutationController('identity.signOut').resolveWith(undefined);
    const user = userEvent.setup();
    render(<SignOutButton />);

    await user.click(screen.getByRole('button', { name: 'Sign out' }));

    await waitFor(() => expect(mutationController('identity.signOut').calls).toHaveLength(1));
    expect(push).toHaveBeenCalledWith('/login');
    expect(refresh).toHaveBeenCalledTimes(1);
  });

  it('shows a pending label while signing out', async () => {
    let resolveSignOut: (value: undefined) => void = () => undefined;
    mutationController('identity.signOut').resolveWith(
      new Promise<undefined>((resolve) => {
        resolveSignOut = resolve;
      }),
    );
    const user = userEvent.setup();
    render(<SignOutButton />);

    await user.click(screen.getByRole('button', { name: 'Sign out' }));

    const pending = await screen.findByRole('button', { name: 'Signing out…' });
    expect(pending).toBeDisabled();

    resolveSignOut(undefined);
    await waitFor(() => expect(push).toHaveBeenCalledWith('/login'));
  });

  it('surfaces a failure without navigating', async () => {
    mutationController('identity.signOut').rejectWith(new Error('network'));
    const user = userEvent.setup();
    render(<SignOutButton />);

    await user.click(screen.getByRole('button', { name: 'Sign out' }));

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Could not sign out. Please try again.',
    );
    expect(push).not.toHaveBeenCalled();
    expect(refresh).not.toHaveBeenCalled();
    expect(screen.getByRole('button', { name: 'Sign out' })).not.toBeDisabled();
  });
});
