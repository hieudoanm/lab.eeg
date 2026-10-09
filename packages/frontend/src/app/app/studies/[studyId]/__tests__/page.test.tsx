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
vi.mock('@/components/molecules/dataset-form', () => ({
  DatasetForm: () => <div data-testid='dataset-form' />,
}));
vi.mock('@/components/atoms/archive-study-button', () => ({
  ArchiveStudyButton: () => <div data-testid='archive-button' />,
}));

import { ApiError } from '@/lib/api/errors';
import StudyDetailPage from '@/app/app/studies/[studyId]/page';
import { caller, requireSession, resetServerMock } from '@/test/server-mock';
import { renderServer } from '@/test/server-render';

const membership = {
  membership_id: 'm1',
  organisation_id: 'org-1',
  organisation_name: 'Org',
  role: 'owner' as const,
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

const dataset = (overrides: Partial<Record<string, unknown>> = {}) => ({
  id: 'dataset-1',
  study_id: 'study-1',
  name: 'Session 1',
  description: null,
  format: 'npy',
  status: 'ready',
  created_at: '2025-01-01T00:00:00Z',
  ...overrides,
});

const trpcError = (message: string, requestId: string | null) =>
  new TRPCError({
    code: 'NOT_FOUND',
    message,
    cause: new ApiError({ status: 404, code: 'not_found', message, requestId }),
  });

type PageProps = ComponentProps<typeof StudyDetailPage>;

const render = () =>
  renderServer(
    <StudyDetailPage
      {...({ params: Promise.resolve({ studyId: 'study-1' }) } as unknown as PageProps)}
    />,
  );

beforeEach(() => {
  vi.clearAllMocks();
  resetServerMock();
  requireSession.mockResolvedValue({ caller, user: { id: 'u1' }, membership });
});

describe('StudyDetailPage', () => {
  it('renders an active study with write controls and its datasets', async () => {
    caller.studies.byId.mockResolvedValue(study({ description: 'A study' }));
    caller.datasets.list.mockResolvedValue({
      items: [dataset({ description: 'raw' }), dataset({ id: 'dataset-2', name: 'Session 2' })],
      next_cursor: null,
      has_more: false,
    });

    await render();

    expect(screen.getByRole('heading', { name: 'Alpha', level: 1 })).toBeInTheDocument();
    expect(screen.getByTestId('study-form')).toBeInTheDocument();
    expect(screen.getByTestId('archive-button')).toBeInTheDocument();
    expect(screen.getByTestId('dataset-form')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Session 1' })).toHaveAttribute(
      'href',
      '/app/studies/study-1/datasets/dataset-1',
    );
    expect(screen.getByText('2 datasets')).toBeInTheDocument();
  });

  it('renders a read-only archived study without write controls', async () => {
    caller.studies.byId.mockResolvedValue(study({ status: 'archived' }));
    caller.datasets.list.mockResolvedValue({ items: [], next_cursor: null, has_more: false });

    await render();

    expect(screen.getByText('Archived study')).toBeInTheDocument();
    expect(screen.queryByTestId('study-form')).not.toBeInTheDocument();
    expect(screen.queryByTestId('archive-button')).not.toBeInTheDocument();
    expect(screen.queryByTestId('dataset-form')).not.toBeInTheDocument();
    expect(screen.getByText('No datasets yet')).toBeInTheDocument();
  });

  it('hides write controls for viewers', async () => {
    requireSession.mockResolvedValue({
      caller,
      user: { id: 'u1' },
      membership: { ...membership, role: 'viewer' },
    });
    caller.studies.byId.mockResolvedValue(study());
    caller.datasets.list.mockResolvedValue({
      items: [dataset()],
      next_cursor: null,
      has_more: false,
    });

    await render();

    expect(screen.queryByTestId('study-form')).not.toBeInTheDocument();
    expect(screen.getByText('1 dataset')).toBeInTheDocument();
  });

  it('shows an empty state when the user has no organisation', async () => {
    requireSession.mockResolvedValue({ caller, user: { id: 'u1' }, membership: null });

    await render();

    expect(screen.getByText('No organisation')).toBeInTheDocument();
  });

  it('shows an error state when loading fails', async () => {
    caller.studies.byId.mockRejectedValue(trpcError('Missing', 'req-9'));
    caller.datasets.list.mockResolvedValue({ items: [], next_cursor: null, has_more: false });

    await render();

    expect(screen.getByRole('alert')).toHaveTextContent('Missing');
    expect(screen.getByText('Request ID: req-9')).toBeInTheDocument();
  });

  it('falls back to a generic error when the study is missing', async () => {
    caller.studies.byId.mockResolvedValue(null);
    caller.datasets.list.mockResolvedValue({ items: [], next_cursor: null, has_more: false });

    await render();

    expect(screen.getByRole('alert')).toHaveTextContent('The study could not be loaded.');
  });

  it('shows a fallback message for unrecognised failures', async () => {
    caller.studies.byId.mockRejectedValue(new Error('boom'));
    caller.datasets.list.mockResolvedValue({ items: [], next_cursor: null, has_more: false });

    await render();

    expect(screen.getByRole('alert')).toHaveTextContent('The study could not be loaded.');
  });
});
