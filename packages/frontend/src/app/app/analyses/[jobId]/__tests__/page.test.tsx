import { TRPCError } from '@trpc/server';
import { screen } from '@testing-library/react';
import type { ComponentProps } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/trpc/server', async () => {
  const { buildServerMock } = await import('@/test/server-mock');
  return buildServerMock();
});

vi.mock('@/trpc/react', async () => {
  const { buildTrpcMock } = await import('@/test/trpc-mock');
  return buildTrpcMock();
});

import { ApiError } from '@/lib/api/errors';
import AnalysisDetailPage from '@/app/app/analyses/[jobId]/page';
import { caller, requireSession, resetServerMock } from '@/test/server-mock';
import { renderServer } from '@/test/server-render';

const membership = {
  membership_id: 'm1',
  organisation_id: 'org-1',
  organisation_name: 'Org',
  role: 'owner' as const,
};

const job = (overrides: Partial<Record<string, unknown>> = {}) => ({
  id: 'job-1234567890',
  organisation_id: 'org-1',
  dataset_id: 'dataset-1',
  analysis_type: 'descriptive_statistics',
  algorithm_version: '1.0.0',
  status: 'succeeded',
  attempt_count: 1,
  requested_at: '2025-01-01T00:00:00Z',
  started_at: '2025-01-01T00:00:10Z',
  completed_at: '2025-01-01T00:00:20Z',
  error: null,
  ...overrides,
});

const dataset = (overrides: Partial<Record<string, unknown>> = {}) => ({
  id: 'dataset-1',
  study_id: 'study-1',
  name: 'Session 1',
  description: null,
  format: 'npy',
  status: 'ready',
  sampling_frequency_hz: 256,
  channel_count: 2,
  sample_count: 1024,
  duration_seconds: 4,
  channel_names: ['Fz', 'Cz'],
  channel_units: ['uV', 'uV'],
  checksum_sha256: 'b'.repeat(64),
  file_size_bytes: 4096,
  source_description: null,
  created_at: '2025-01-01T00:00:00Z',
  updated_at: '2025-01-02T00:00:00Z',
  validated_at: '2025-01-01T00:00:00Z',
  ...overrides,
});

const result = (overrides: Partial<Record<string, unknown>> = {}) => ({
  analysis_id: 'job-1234567890',
  dataset_id: 'dataset-1',
  algorithm_version: '1.0.0',
  result_schema_version: '1.0.0',
  created_at: '2025-01-01T00:00:20Z',
  channels: [
    {
      channel_index: 0,
      channel_name: 'Fz',
      unit: null,
      mean: 0.1234,
      standard_deviation: 1.234,
      minimum: -2,
      maximum: 2,
      peak_to_peak: 4,
      valid_sample_count: 1024,
    },
  ],
  provenance: {
    input_checksum_sha256: 'c'.repeat(64),
    sampling_frequency_hz: 256,
    channel_count: 1,
    sample_count: 1024,
    channel_names: ['Fz'],
    channel_units: ['uV'],
    parameters: {
      include_mean: true,
      include_standard_deviation: true,
      include_min_max: true,
      include_peak_to_peak: true,
    },
  },
  ...overrides,
});

const trpcError = (message: string, requestId: string | null) =>
  new TRPCError({
    code: 'INTERNAL_SERVER_ERROR',
    message,
    cause: new ApiError({ status: 500, code: 'unexpected_error', message, requestId }),
  });

type PageProps = ComponentProps<typeof AnalysisDetailPage>;

const render = () =>
  renderServer(
    <AnalysisDetailPage
      {...({ params: Promise.resolve({ jobId: 'job-1234567890' }) } as unknown as PageProps)}
    />,
  );

beforeEach(() => {
  vi.clearAllMocks();
  resetServerMock();
  requireSession.mockResolvedValue({ caller, user: { id: 'u1' }, membership });
});

describe('AnalysisDetailPage', () => {
  it('renders a completed analysis with provenance and per-channel results', async () => {
    caller.analyses.byId.mockResolvedValue(job());
    caller.analyses.result.mockResolvedValue(result());
    caller.datasets.byId.mockResolvedValue(dataset());

    await render();

    expect(screen.getByRole('heading', { name: 'Analysis', level: 1 })).toBeInTheDocument();
    expect(screen.getByText('Provenance')).toBeInTheDocument();
    expect(screen.getByText('Per-channel results')).toBeInTheDocument();
    expect(screen.getByText('Ch 0')).toBeInTheDocument();
    expect(caller.analyses.result).toHaveBeenCalledWith({ jobId: 'job-1234567890' });
  });

  it('does not fetch a result while the job is still running', async () => {
    caller.analyses.byId.mockResolvedValue(job({ status: 'running', completed_at: null }));
    caller.datasets.byId.mockResolvedValue(dataset());

    await render();

    expect(screen.getByText('Running')).toBeInTheDocument();
    expect(caller.analyses.result).not.toHaveBeenCalled();
  });

  it('renders fallback provenance when the result omits it', async () => {
    const withoutProvenance = result();
    delete (withoutProvenance as Record<string, unknown>).provenance;
    caller.analyses.byId.mockResolvedValue(job());
    caller.analyses.result.mockResolvedValue(withoutProvenance);
    caller.datasets.byId.mockResolvedValue(dataset({ sampling_frequency_hz: null }));

    await render();

    expect(screen.getByText('Provenance')).toBeInTheDocument();
    expect(screen.getByText('Ch 0')).toBeInTheDocument();
  });

  it('shows an empty state when the user has no organisation', async () => {
    requireSession.mockResolvedValue({ caller, user: { id: 'u1' }, membership: null });

    await render();

    expect(screen.getByText('No organisation')).toBeInTheDocument();
    expect(caller.analyses.byId).not.toHaveBeenCalled();
  });

  it('shows an API error with the request id', async () => {
    caller.analyses.byId.mockRejectedValue(trpcError('Boom', 'req-3'));

    await render();

    expect(screen.getByRole('alert')).toHaveTextContent('Boom');
    expect(screen.getByText('Request ID: req-3')).toBeInTheDocument();
  });

  it('falls back to a generic error when the job is missing', async () => {
    caller.analyses.byId.mockResolvedValue(null);

    await render();

    expect(screen.getByRole('alert')).toHaveTextContent('The analysis could not be loaded.');
  });

  it('shows a fallback message for unrecognised failures', async () => {
    caller.analyses.byId.mockRejectedValue(new Error('boom'));

    await render();

    expect(screen.getByRole('alert')).toHaveTextContent('The analysis could not be loaded.');
  });
});
