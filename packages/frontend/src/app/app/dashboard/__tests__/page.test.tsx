import { TRPCError } from '@trpc/server';
import { screen } from '@testing-library/react';
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

import DashboardPage from '@/app/app/dashboard/page';
import { ApiError } from '@/lib/api/errors';
import { caller, requireSession, resetServerMock } from '@/test/server-mock';
import { renderServer } from '@/test/server-render';

const membership = {
  membership_id: 'm1',
  organisation_id: 'org-1',
  organisation_name: 'Org',
  role: 'researcher',
};

const SUMMARY = {
  study_count: 4,
  active_study_count: 3,
  dataset_count: 9,
  ready_dataset_count: 7,
  analysis_count: 21,
  analyses_by_status: {
    queued: 1,
    running: 2,
    succeeded: 15,
    failed: 2,
    cancelled: 1,
  },
};

const study = { id: 'study-1', name: 'Alpha', updated_at: '2026-10-09T16:00:00Z' };
const job = {
  id: 'job-1',
  status: 'succeeded' as const,
  dataset_id: '2b4c6d8e-1111-4222-8333-abcdefabcdef',
  requested_at: '2026-10-09T16:40:00Z',
};

const render = () => renderServer(<DashboardPage />);

beforeEach(() => {
  vi.clearAllMocks();
  resetServerMock();
  requireSession.mockResolvedValue({ caller, user: { id: 'u1' }, membership });
});

describe('DashboardPage', () => {
  it('renders organisation counts and recent activity', async () => {
    caller.summary.organisation.mockResolvedValue(SUMMARY);
    caller.studies.list.mockResolvedValue({ items: [study], next_cursor: null, has_more: false });
    caller.analyses.listOrganisation.mockResolvedValue({
      items: [job],
      next_cursor: null,
      has_more: false,
    });

    await render();

    expect(screen.getByRole('heading', { name: 'Dashboard' })).toBeInTheDocument();
    const summary = screen.getByRole('region', { name: 'Summary' });
    expect(summary).toHaveTextContent('4');
    expect(summary).toHaveTextContent('3 active');
    expect(summary).toHaveTextContent('9');
    expect(summary).toHaveTextContent('7 ready');
    expect(summary).toHaveTextContent('21');
    expect(summary).toHaveTextContent('15 succeeded · 2 failed');

    expect(screen.getByRole('link', { name: 'Alpha' })).toHaveAttribute(
      'href',
      '/app/studies/study-1',
    );
    expect(caller.studies.list).toHaveBeenCalledWith({ limit: 5 });
    expect(caller.analyses.listOrganisation).toHaveBeenCalledWith({ limit: 5 });
  });

  it('shows empty states when there is no recent activity', async () => {
    caller.summary.organisation.mockResolvedValue(SUMMARY);
    caller.studies.list.mockResolvedValue({ items: [], next_cursor: null, has_more: false });
    caller.analyses.listOrganisation.mockResolvedValue({
      items: [],
      next_cursor: null,
      has_more: false,
    });

    await render();

    expect(screen.getByText('No studies yet')).toBeInTheDocument();
    expect(screen.getByText('No analyses yet')).toBeInTheDocument();
  });

  it('shows an empty state when the user has no organisation', async () => {
    requireSession.mockResolvedValue({ caller, user: { id: 'u1' }, membership: null });

    await render();

    expect(screen.getByText('No organisation')).toBeInTheDocument();
    expect(caller.summary.organisation).not.toHaveBeenCalled();
  });

  it('shows an API error with the request id', async () => {
    caller.studies.list.mockResolvedValue({ items: [], next_cursor: null, has_more: false });
    caller.analyses.listOrganisation.mockResolvedValue({
      items: [],
      next_cursor: null,
      has_more: false,
    });
    caller.summary.organisation.mockRejectedValue(
      new TRPCError({
        code: 'INTERNAL_SERVER_ERROR',
        message: 'Boom',
        cause: new ApiError({
          status: 500,
          code: 'server_error',
          message: 'Boom',
          requestId: 'req-9',
        }),
      }),
    );

    await render();

    expect(screen.getByRole('alert')).toHaveTextContent('Boom');
    expect(screen.getByText('Request ID: req-9')).toBeInTheDocument();
  });
});
