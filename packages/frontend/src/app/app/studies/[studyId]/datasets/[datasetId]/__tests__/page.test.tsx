import { TRPCError } from '@trpc/server';
import { screen } from '@testing-library/react';
import type { ComponentProps } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/trpc/server', async () => {
  const { buildServerMock } = await import('@/test/server-mock');
  return buildServerMock();
});

vi.mock('@/components/molecules/dataset-upload-form', () => ({
  DatasetUploadForm: () => <div data-testid='upload-form' />,
}));

import { ApiError } from '@/lib/api/errors';
import DatasetDetailPage from '@/app/app/studies/[studyId]/datasets/[datasetId]/page';
import { caller, requireSession, resetServerMock } from '@/test/server-mock';
import { renderServer } from '@/test/server-render';

const membership = {
  membership_id: 'm1',
  organisation_id: 'org-1',
  organisation_name: 'Org',
  role: 'owner' as const,
};

const dataset = (overrides: Partial<Record<string, unknown>> = {}) => ({
  id: 'dataset-1',
  study_id: 'study-1',
  name: 'Session 1',
  description: null,
  format: 'npy',
  status: 'pending_upload',
  sampling_frequency_hz: 256,
  channel_count: 4,
  sample_count: 1024,
  duration_seconds: 4,
  channel_names: ['Fz', 'Cz'],
  channel_units: ['uV', 'uV'],
  checksum_sha256: 'a'.repeat(64),
  file_size_bytes: 4096,
  source_description: null,
  created_at: '2025-01-01T00:00:00Z',
  updated_at: '2025-01-02T00:00:00Z',
  validated_at: null,
  ...overrides,
});

const trpcError = (message: string, requestId: string | null) =>
  new TRPCError({
    code: 'NOT_FOUND',
    message,
    cause: new ApiError({ status: 404, code: 'not_found', message, requestId }),
  });

type PageProps = ComponentProps<typeof DatasetDetailPage>;

const render = () =>
  renderServer(
    <DatasetDetailPage
      {...({
        params: Promise.resolve({ studyId: 'study-1', datasetId: 'dataset-1' }),
      } as unknown as PageProps)}
    />,
  );

beforeEach(() => {
  vi.clearAllMocks();
  resetServerMock();
  requireSession.mockResolvedValue({ caller, user: { id: 'u1' }, membership });
});

describe('DatasetDetailPage', () => {
  it('renders an uploadable dataset with its metadata', async () => {
    caller.datasets.byId.mockResolvedValue(dataset({ source_description: 'Lab A' }));

    await render();

    expect(screen.getByRole('heading', { name: 'Session 1', level: 1 })).toBeInTheDocument();
    expect(screen.getByTestId('upload-form')).toBeInTheDocument();
    expect(screen.getByText('256 Hz')).toBeInTheDocument();
    expect(screen.getByText('Fz, Cz')).toBeInTheDocument();
    expect(screen.getByText('Lab A')).toBeInTheDocument();
    expect(screen.queryByText(/no accepted recording/i)).not.toBeInTheDocument();
  });

  it('renders placeholders when optional recording metadata is absent', async () => {
    caller.datasets.byId.mockResolvedValue(
      dataset({
        channel_count: null,
        sample_count: null,
        channel_names: null,
        channel_units: null,
        source_description: null,
        status: 'ready',
      }),
    );

    await render();

    expect(screen.getAllByText('—').length).toBeGreaterThan(0);
  });

  it('explains that a rejected dataset needs a corrected upload', async () => {
    caller.datasets.byId.mockResolvedValue(
      dataset({ status: 'rejected', sampling_frequency_hz: null }),
    );

    await render();

    expect(screen.getByTestId('upload-form')).toBeInTheDocument();
    expect(screen.getByText(/no accepted recording/i)).toBeInTheDocument();
  });

  it('shows the validating notice for a dataset under validation', async () => {
    caller.datasets.byId.mockResolvedValue(dataset({ status: 'validating' }));

    await render();

    expect(screen.queryByTestId('upload-form')).not.toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Validating' })).toBeInTheDocument();
    expect(screen.getByText(/Validation is running/i)).toBeInTheDocument();
  });

  it('hides upload controls for a ready dataset', async () => {
    caller.datasets.byId.mockResolvedValue(dataset({ status: 'ready' }));

    await render();

    expect(screen.queryByTestId('upload-form')).not.toBeInTheDocument();
    expect(screen.queryByText('Validating')).not.toBeInTheDocument();
  });

  it('hides write controls for viewers', async () => {
    requireSession.mockResolvedValue({
      caller,
      user: { id: 'u1' },
      membership: { ...membership, role: 'viewer' },
    });
    caller.datasets.byId.mockResolvedValue(dataset());

    await render();

    expect(screen.queryByTestId('upload-form')).not.toBeInTheDocument();
  });

  it('shows an empty state when the user has no organisation', async () => {
    requireSession.mockResolvedValue({ caller, user: { id: 'u1' }, membership: null });

    await render();

    expect(screen.getByText('No organisation')).toBeInTheDocument();
  });

  it('shows an error state when loading fails', async () => {
    caller.datasets.byId.mockRejectedValue(trpcError('Gone', 'req-7'));

    await render();

    expect(screen.getByRole('alert')).toHaveTextContent('Gone');
    expect(screen.getByText('Request ID: req-7')).toBeInTheDocument();
  });

  it('falls back to a generic error when the dataset is missing', async () => {
    caller.datasets.byId.mockResolvedValue(null);

    await render();

    expect(screen.getByRole('alert')).toHaveTextContent('The dataset could not be loaded.');
  });

  it('shows a fallback message for unrecognised failures', async () => {
    caller.datasets.byId.mockRejectedValue(new Error('boom'));

    await render();

    expect(screen.getByRole('alert')).toHaveTextContent('The dataset could not be loaded.');
  });
});
