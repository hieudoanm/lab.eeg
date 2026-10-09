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

import AnalysesPage from '@/app/app/analyses/page';
import { ApiError } from '@/lib/api/errors';
import { caller, requireSession, resetServerMock } from '@/test/server-mock';
import { renderServer } from '@/test/server-render';

const membership = {
  membership_id: 'm1',
  organisation_id: 'org-1',
  organisation_name: 'Org',
  role: 'researcher',
};

const job = {
  id: 'job-1',
  status: 'succeeded' as const,
  analysis_type: 'descriptive_statistics',
  algorithm_version: '1.0.0',
  dataset_id: '2b4c6d8e-1111-4222-8333-abcdefabcdef',
  attempt_count: 1,
  max_attempts: 3,
  requested_at: '2026-10-09T16:40:00Z',
  started_at: null,
  completed_at: null,
  error: null,
};

const render = (searchParams: Record<string, string | string[]> = {}) => {
  return renderServer(
    <AnalysesPage
      {...({ searchParams: Promise.resolve(searchParams) } as unknown as ComponentProps<
        typeof AnalysesPage
      >)}
    />,
  );
};

beforeEach(() => {
  vi.clearAllMocks();
  resetServerMock();
  requireSession.mockResolvedValue({ caller, user: { id: 'u1' }, membership });
});

describe('AnalysesPage', () => {
  it('renders jobs with a next-page link and applies filters', async () => {
    caller.analyses.listOrganisation.mockResolvedValue({
      items: [job],
      next_cursor: 'cursor-2',
      has_more: true,
    });

    await render({ status: 'succeeded' });

    expect(screen.getByRole('heading', { name: 'Analyses', level: 1 })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Succeeded' })).toHaveAttribute(
      'href',
      '/app/analyses/job-1',
    );
    expect(screen.getByRole('link', { name: 'Next page' })).toHaveAttribute(
      'href',
      '/app/analyses?status=succeeded&cursor=cursor-2',
    );
    expect(caller.analyses.listOrganisation).toHaveBeenCalledWith({
      status: 'succeeded',
      cursor: undefined,
      limit: 25,
    });
  });

  it('shows the empty state when there are no jobs and no filters', async () => {
    caller.analyses.listOrganisation.mockResolvedValue({
      items: [],
      next_cursor: null,
      has_more: false,
    });

    await render();

    expect(screen.getByText('No analyses yet')).toBeInTheDocument();
  });

  it('shows no-results when filters return nothing', async () => {
    caller.analyses.listOrganisation.mockResolvedValue({
      items: [],
      next_cursor: null,
      has_more: false,
    });

    await render({ status: 'failed' });

    expect(screen.getByText('No analyses match the current filters.')).toBeInTheDocument();
  });

  it('shows an empty state when the user has no organisation', async () => {
    requireSession.mockResolvedValue({ caller, user: { id: 'u1' }, membership: null });

    await render();

    expect(screen.getByText('No organisation')).toBeInTheDocument();
    expect(caller.analyses.listOrganisation).not.toHaveBeenCalled();
  });

  it('shows an API error with the request id', async () => {
    caller.analyses.listOrganisation.mockRejectedValue(
      new TRPCError({
        code: 'INTERNAL_SERVER_ERROR',
        message: 'Boom',
        cause: new ApiError({
          status: 500,
          code: 'server_error',
          message: 'Boom',
          requestId: 'req-3',
        }),
      }),
    );

    await render();

    expect(screen.getByRole('alert')).toHaveTextContent('Boom');
    expect(screen.getByText('Request ID: req-3')).toBeInTheDocument();
  });
});
