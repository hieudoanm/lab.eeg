import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/trpc/react', async () => {
  const { buildTrpcMock } = await import('@/test/trpc-mock');
  return buildTrpcMock();
});

const { push, refresh } = vi.hoisted(() => ({ push: vi.fn(), refresh: vi.fn() }));

vi.mock('next/navigation', () => ({ useRouter: () => ({ push, refresh }) }));

import { LoginForm } from '@/components/molecules/login-form';
import { mutationController, resetTrpcMocks } from '@/test/trpc-mock';

const withStatus = (status: number, message: string): Error =>
  Object.assign(new Error(message), { data: { httpStatus: status } });

const signIn = () => mutationController('identity.signIn');

beforeEach(() => {
  resetTrpcMocks();
  push.mockReset();
  refresh.mockReset();
});

describe('LoginForm', () => {
  it('signs in and navigates to the workspace', async () => {
    signIn().resolveWith({ expiresAt: '2026-01-01T00:00:00Z' });
    const user = userEvent.setup();
    render(<LoginForm />);

    await user.type(screen.getByLabelText(/Email address/), 'ada@example.org');
    await user.click(screen.getByRole('button', { name: 'Sign in' }));

    await screen.findByRole('button', { name: 'Sign in' });
    expect(signIn().calls[0]).toEqual({ subject: 'ada@example.org', email: 'ada@example.org' });
    expect(push).toHaveBeenCalledWith('/app/dashboard');
    expect(refresh).toHaveBeenCalled();
  });

  it('requires an email address before submitting', async () => {
    const user = userEvent.setup();
    render(<LoginForm />);

    await user.click(screen.getByRole('button', { name: 'Sign in' }));

    expect(await screen.findByRole('alert')).toHaveTextContent('Enter an email address');
    expect(signIn().calls).toHaveLength(0);
    expect(push).not.toHaveBeenCalled();
  });

  it('explains that sign-in is unavailable in this environment', async () => {
    signIn().rejectWith(withStatus(404, 'The requested resource does not exist.'));
    const user = userEvent.setup();
    render(<LoginForm />);

    await user.type(screen.getByLabelText(/Email address/), 'ada@example.org');
    await user.click(screen.getByRole('button', { name: 'Sign in' }));

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Sign-in is not available in this environment.',
    );
    expect(push).not.toHaveBeenCalled();
  });

  it('shows the server message for other HTTP failures', async () => {
    signIn().rejectWith(withStatus(500, 'Something went wrong on our side.'));
    const user = userEvent.setup();
    render(<LoginForm />);

    await user.type(screen.getByLabelText(/Email address/), 'ada@example.org');
    await user.click(screen.getByRole('button', { name: 'Sign in' }));

    expect(await screen.findByRole('alert')).toHaveTextContent('Something went wrong on our side.');
    expect(push).not.toHaveBeenCalled();
  });

  it('shows a connection message for network failures', async () => {
    signIn().rejectWith(new Error('Failed to fetch'));
    const user = userEvent.setup();
    render(<LoginForm />);

    await user.type(screen.getByLabelText(/Email address/), 'ada@example.org');
    await user.click(screen.getByRole('button', { name: 'Sign in' }));

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'We could not sign you in. Check your connection and try again.',
    );
    expect(push).not.toHaveBeenCalled();
  });
});
