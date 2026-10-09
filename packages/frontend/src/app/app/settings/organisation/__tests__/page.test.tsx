import { TRPCError } from '@trpc/server';
import { screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/trpc/server', async () => {
  const { buildServerMock } = await import('@/test/server-mock');
  return buildServerMock();
});

import { ApiError } from '@/lib/api/errors';
import OrganisationSettingsPage from '@/app/app/settings/organisation/page';
import { caller, requireSession, resetServerMock } from '@/test/server-mock';
import { renderServer } from '@/test/server-render';

const membership = {
  membership_id: 'm1',
  organisation_id: 'org-1',
  organisation_name: 'Org',
  role: 'owner' as const,
};

const organisation = {
  id: 'org-1',
  name: 'Neuro Lab',
  slug: 'neuro-lab',
  status: 'active' as const,
  role: 'owner' as const,
  created_at: '2025-01-01T00:00:00Z',
  updated_at: '2025-01-02T00:00:00Z',
};

const member = (overrides: Partial<Record<string, unknown>> = {}) => ({
  id: 'mem-1',
  user_id: 'u1',
  display_name: 'Ada',
  email: 'ada@example.org',
  role: 'owner' as const,
  created_at: '2025-01-01T00:00:00Z',
  updated_at: '2025-01-01T00:00:00Z',
  ...overrides,
});

const trpcError = (message: string, requestId: string | null) =>
  new TRPCError({
    code: 'INTERNAL_SERVER_ERROR',
    message,
    cause: new ApiError({ status: 500, code: 'unexpected_error', message, requestId }),
  });

beforeEach(() => {
  vi.clearAllMocks();
  resetServerMock();
  requireSession.mockResolvedValue({ caller, user: { id: 'u1' }, membership });
});

describe('OrganisationSettingsPage', () => {
  it('renders organisation details and its members', async () => {
    caller.identity.organisation.mockResolvedValue(organisation);
    caller.identity.members.mockResolvedValue({
      items: [member(), member({ id: 'mem-2', display_name: null, email: null, role: 'viewer' })],
      next_cursor: null,
      has_more: false,
    });

    await renderServer(<OrganisationSettingsPage />);

    expect(screen.getByRole('heading', { name: 'Organisation settings' })).toBeInTheDocument();
    expect(screen.getByText('Neuro Lab')).toBeInTheDocument();
    expect(screen.getByText('2 members')).toBeInTheDocument();
    expect(screen.getByText('Ada')).toBeInTheDocument();
    expect(screen.getByText('Unnamed member')).toBeInTheDocument();
  });

  it('uses the singular member label for a single member', async () => {
    caller.identity.organisation.mockResolvedValue(organisation);
    caller.identity.members.mockResolvedValue({
      items: [member()],
      next_cursor: null,
      has_more: false,
    });

    await renderServer(<OrganisationSettingsPage />);

    expect(screen.getByText('1 member')).toBeInTheDocument();
  });

  it('renders an empty members message', async () => {
    caller.identity.organisation.mockResolvedValue(organisation);
    caller.identity.members.mockResolvedValue({ items: [], next_cursor: null, has_more: false });

    await renderServer(<OrganisationSettingsPage />);

    expect(screen.getByText('This organisation has no members.')).toBeInTheDocument();
  });

  it('shows an empty state when the user has no organisation', async () => {
    requireSession.mockResolvedValue({ caller, user: { id: 'u1' }, membership: null });

    await renderServer(<OrganisationSettingsPage />);

    expect(screen.getByText('No organisation')).toBeInTheDocument();
  });

  it('shows an API error with the request id', async () => {
    caller.identity.organisation.mockRejectedValue(trpcError('Boom', 'req-5'));
    caller.identity.members.mockResolvedValue({ items: [], next_cursor: null, has_more: false });

    await renderServer(<OrganisationSettingsPage />);

    expect(screen.getByRole('alert')).toHaveTextContent('Boom');
    expect(screen.getByText('Request ID: req-5')).toBeInTheDocument();
  });

  it('falls back to a generic error when the organisation is missing', async () => {
    caller.identity.organisation.mockResolvedValue(null);
    caller.identity.members.mockResolvedValue({ items: [], next_cursor: null, has_more: false });

    await renderServer(<OrganisationSettingsPage />);

    expect(screen.getByRole('alert')).toHaveTextContent('The organisation could not be loaded.');
  });

  it('shows a fallback message for unrecognised failures', async () => {
    caller.identity.organisation.mockRejectedValue(new Error('boom'));
    caller.identity.members.mockResolvedValue({ items: [], next_cursor: null, has_more: false });

    await renderServer(<OrganisationSettingsPage />);

    expect(screen.getByRole('alert')).toHaveTextContent('The organisation could not be loaded.');
  });
});
