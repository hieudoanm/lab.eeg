import { TRPCError } from '@trpc/server';
import { screen } from '@testing-library/react';
import type { ComponentProps } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/trpc/server', async () => {
  const { buildServerMock } = await import('@/test/server-mock');
  return buildServerMock();
});

vi.mock('next/link', () => ({
  default: ({ href, children, ...rest }: { href: string; children: React.ReactNode }) => (
    <a href={href} {...rest}>
      {children}
    </a>
  ),
}));

vi.mock('@/components/molecules/study-form', () => ({
  StudyForm: () => <div data-testid='study-form' />,
}));

import { ApiError } from '@/lib/api/errors';
import StudiesPage from '@/app/app/studies/page';
import { caller, requireSession, resetServerMock } from '@/test/server-mock';
import { renderServer } from '@/test/server-render';

const membership = {
  membership_id: 'm1',
  organisation_id: 'org-1',
  organisation_name: 'Org',
  role: 'researcher' as const,
};

const study = (overrides: Partial<Record<string, unknown>> = {}) => ({
  id: 'study-1',
  name: 'Alpha',
  description: null,
  status: 'active',
  created_at: '2025-01-01T00:00:00Z',
  updated_at: '2025-01-02T00:00:00Z',
  archived_at: null,
  ...overrides,
});

const trpcError = (message: string, requestId: string | null) =>
  new TRPCError({
    code: 'INTERNAL_SERVER_ERROR',
    message,
    cause: new ApiError({ status: 500, code: 'unexpected_error', message, requestId }),
  });

type PageProps = ComponentProps<typeof StudiesPage>;

const render = (searchParams: Record<string, string | string[]> = {}) =>
  renderServer(
    <StudiesPage {...({ searchParams: Promise.resolve(searchParams) } as unknown as PageProps)} />,
  );

beforeEach(() => {
  vi.clearAllMocks();
  resetServerMock();
  requireSession.mockResolvedValue({ caller, user: { id: 'u1' }, membership });
});

describe('StudiesPage', () => {
  it('renders studies with a next-page link and the create form for writers', async () => {
    caller.studies.list.mockResolvedValue({
      items: [
        study({ description: 'A study' }),
        study({ id: 'study-2', name: 'Beta', status: 'archived' }),
      ],
      next_cursor: 'cursor-2',
      has_more: true,
    });

    await render({ status: 'active' });

    expect(screen.getByRole('heading', { name: 'Studies', level: 1 })).toBeInTheDocument();
    expect(screen.getByTestId('study-form')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Alpha' })).toHaveAttribute(
      'href',
      '/app/studies/study-1',
    );
    expect(screen.getByRole('link', { name: 'Next page' })).toHaveAttribute(
      'href',
      '/app/studies?status=active&cursor=cursor-2',
    );
    expect(caller.studies.list).toHaveBeenCalledWith({
      search: undefined,
      status: 'active',
      cursor: undefined,
    });
  });

  it('carries the active search and status into the next-page link', async () => {
    caller.studies.list.mockResolvedValue({
      items: [study()],
      next_cursor: 'cursor-2',
      has_more: true,
    });

    await render({ search: 'alpha', status: 'active' });

    expect(screen.getByRole('link', { name: 'Next page' })).toHaveAttribute(
      'href',
      '/app/studies?search=alpha&status=active&cursor=cursor-2',
    );
  });

  it('hides write controls and shows no-results when filters return nothing', async () => {
    requireSession.mockResolvedValue({
      caller,
      user: { id: 'u1' },
      membership: { ...membership, role: 'viewer' },
    });
    caller.studies.list.mockResolvedValue({ items: [], next_cursor: null, has_more: false });

    await render({ search: 'zzz' });

    expect(screen.queryByTestId('study-form')).not.toBeInTheDocument();
    expect(screen.getByText('No studies match the current filters.')).toBeInTheDocument();
  });

  it('shows the empty state when there are no studies and no filters', async () => {
    caller.studies.list.mockResolvedValue({ items: [], next_cursor: null, has_more: false });

    await render();

    expect(screen.getByText('No studies yet')).toBeInTheDocument();
  });

  it('reads the first value when a search param is repeated', async () => {
    caller.studies.list.mockResolvedValue({ items: [], next_cursor: null, has_more: false });

    await render({ search: ['first', 'second'] });

    expect(caller.studies.list).toHaveBeenCalledWith({
      search: 'first',
      status: undefined,
      cursor: undefined,
    });
  });

  it('shows an empty state when the user has no organisation', async () => {
    requireSession.mockResolvedValue({ caller, user: { id: 'u1' }, membership: null });

    await render();

    expect(screen.getByText('No organisation')).toBeInTheDocument();
    expect(caller.studies.list).not.toHaveBeenCalled();
  });

  it('shows an API error with the request id', async () => {
    caller.studies.list.mockRejectedValue(trpcError('Boom', 'req-3'));

    await render();

    expect(screen.getByRole('alert')).toHaveTextContent('Boom');
    expect(screen.getByText('Request ID: req-3')).toBeInTheDocument();
  });

  it('shows a fallback message for unrecognised failures', async () => {
    caller.studies.list.mockRejectedValue(new Error('nope'));

    await render();

    expect(screen.getByRole('alert')).toHaveTextContent('The studies could not be loaded.');
  });
});
