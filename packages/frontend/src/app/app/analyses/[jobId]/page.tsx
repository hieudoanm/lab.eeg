import { Suspense } from 'react';

import { PageHeader } from '@/components/molecules/page-header';
import { EmptyState, ErrorState, LoadingState } from '@/components/atoms/states';
import { Card } from '@/components/atoms/card';
import { MetadataGrid } from '@/components/atoms/metadata-grid';
import { StatusBadge } from '@/components/atoms/status-badge';
import type { AnalysisJob, AnalysisResult, ChannelStatistics } from '@/lib/api';
import { formatDateTime, formatNumber, formatDuration, formatChecksum } from '@/lib/format';
import { datasetStatusLabel, datasetStatusTone } from '@/lib/status';
import { describeError } from '@/trpc/errors';
import { requireSession } from '@/trpc/server';
import { AnalysisStatusDetail, analysisStatusIsTerminal } from '@/components/atoms/analysis-status';
import { SignalVisualisation } from '@/components/organisms/signal-visualisation';

export const metadata = { title: 'Analysis' };

interface PageProps {
  params: Promise<{ jobId: string }>;
}

const AnalysisDetailPage = async (props: PageProps) => {
  const { jobId } = await props.params;

  return (
    <div className='flex flex-col gap-6'>
      <Suspense fallback={<LoadingState label='Loading analysis…' />}>
        <AnalysisDetailContent jobId={jobId} />
      </Suspense>
    </div>
  );
};

const AnalysisDetailContent = async ({ jobId }: { jobId: string }) => {
  const { caller, membership } = await requireSession();
  if (!membership) {
    return (
      <EmptyState
        title='No organisation'
        description='You are not a member of any organisation yet.'
      />
    );
  }

  let job: AnalysisJob | null = null;
  let result: AnalysisResult | null = null;
  let dataset: Awaited<ReturnType<typeof caller.datasets.byId>> | null = null;
  let errorMessage: string | null = null;
  let requestId: string | null = null;

  try {
    job = await caller.analyses.byId({ jobId });
    if (analysisStatusIsTerminal(job.status) && job.status === 'succeeded') {
      result = await caller.analyses.result({ jobId });
    }
    if (job.dataset_id) {
      dataset = await caller.datasets.byId({ datasetId: job.dataset_id });
    }
  } catch (error) {
    const described = describeError(error, 'The analysis could not be loaded.');
    errorMessage = described.message;
    requestId = described.requestId;
  }

  if (errorMessage !== null || job === null) {
    return (
      <ErrorState
        message={errorMessage ?? 'The analysis could not be loaded.'}
        requestId={requestId}
      />
    );
  }

  const provenance = result?.provenance ?? {
    input_checksum_sha256: null,
    sampling_frequency_hz: null,
    channel_count: 0,
    sample_count: 0,
    channel_names: [] as string[],
    channel_units: [] as (string | null)[],
    parameters: {
      include_mean: true,
      include_standard_deviation: true,
      include_min_max: true,
      include_peak_to_peak: true,
    },
  };
  const samplingFrequency =
    provenance.sampling_frequency_hz ?? dataset?.sampling_frequency_hz ?? null;
  const channelNames = provenance.channel_names ?? dataset?.channel_names ?? [];
  const channelUnits = provenance.channel_units ?? dataset?.channel_units ?? [];

  return (
    <>
      <PageHeader
        title='Analysis'
        description={
          result
            ? 'Results and provenance for the completed analysis.'
            : 'Job status, parameters, and execution details.'
        }
        breadcrumbs={[{ label: 'Analyses', href: '/app/analyses' }, { label: job.id.slice(0, 8) }]}
      />

      <Card title='Job status'>
        <AnalysisStatusDetail
          status={job.status}
          attemptCount={job.attempt_count}
          maxAttempts={job.max_attempts}
          error={job.error}
        />
        <div className='mt-4 grid gap-4 md:grid-cols-2'>
          <MetadataGrid
            items={[
              { label: 'Job ID', value: job.id, mono: true },
              { label: 'Type', value: job.analysis_type },
              { label: 'Algorithm version', value: job.algorithm_version },
              { label: 'Requested', value: formatDateTime(job.requested_at) },
              { label: 'Started', value: job.started_at ? formatDateTime(job.started_at) : '—' },
              {
                label: 'Completed',
                value: job.completed_at ? formatDateTime(job.completed_at) : '—',
              },
              { label: 'Attempts', value: `${job.attempt_count} / ${job.max_attempts}` },
            ]}
          />
        </div>
      </Card>

      {dataset && (
        <Card title='Dataset'>
          <MetadataGrid
            columns={3}
            items={[
              { label: 'Name', value: dataset.name },
              {
                label: 'Status',
                value: (
                  <StatusBadge
                    tone={datasetStatusTone(dataset.status)}
                    label={datasetStatusLabel(dataset.status)}
                  />
                ),
              },
              { label: 'Format', value: dataset.format.toUpperCase() },
              {
                label: 'Sampling frequency',
                value: samplingFrequency ? `${formatNumber(samplingFrequency)} Hz` : '—',
              },
              { label: 'Channels', value: dataset.channel_count?.toString() ?? '—' },
              { label: 'Samples', value: dataset.sample_count?.toString() ?? '—' },
              {
                label: 'Duration',
                value: dataset.duration_seconds ? formatDuration(dataset.duration_seconds) : '—',
              },
              {
                label: 'Checksum (SHA-256)',
                value: formatChecksum(dataset.checksum_sha256),
                mono: true,
              },
            ]}
          />
        </Card>
      )}

      {dataset && samplingFrequency && channelNames.length > 0 && (
        <Card title='Signal visualisation'>
          <SignalVisualisation
            datasetId={dataset.id}
            samplingFrequencyHz={samplingFrequency}
            channelNames={channelNames}
            channelUnits={channelUnits}
            durationSeconds={dataset.duration_seconds}
          />
        </Card>
      )}

      {result && (
        <>
          <Card title='Provenance'>
            <MetadataGrid
              columns={3}
              items={[
                { label: 'Result schema version', value: result.result_schema_version },
                {
                  label: 'Input checksum (SHA-256)',
                  value: formatChecksum(provenance.input_checksum_sha256 as string),
                  mono: true,
                },
                {
                  label: 'Sampling frequency',
                  value: samplingFrequency ? `${formatNumber(samplingFrequency)} Hz` : '—',
                },
                { label: 'Channel count', value: (provenance.channel_count ?? '—').toString() },
                { label: 'Sample count', value: (provenance.sample_count ?? '—').toString() },
              ]}
            />
          </Card>

          <Card title='Per-channel results'>
            <div className='space-y-6'>
              {result.channels.map((channel: ChannelStatistics, index) => {
                const unit = channelUnits[index] ?? channel.unit ?? null;
                return (
                  <div key={channel.channel_index} className='rounded-lg border p-4'>
                    <h4 className='mb-3 flex items-center gap-2 font-medium'>
                      <span className='text-base-content/60 font-mono text-sm'>
                        Ch {channel.channel_index}
                      </span>
                      {channelNames[index] ?? channel.channel_name}
                      {unit && <span className='text-base-content/60 text-xs'>({unit})</span>}
                    </h4>
                    <MetadataGrid
                      columns={3}
                      items={[
                        { label: 'Mean', value: channel.mean?.toFixed(3) ?? '—' },
                        {
                          label: 'Std. deviation',
                          value: channel.standard_deviation?.toFixed(3) ?? '—',
                        },
                        { label: 'Minimum', value: channel.minimum?.toFixed(3) ?? '—' },
                        { label: 'Maximum', value: channel.maximum?.toFixed(3) ?? '—' },
                        { label: 'Peak-to-peak', value: channel.peak_to_peak?.toFixed(3) ?? '—' },
                        {
                          label: 'Valid samples',
                          value: channel.valid_sample_count?.toLocaleString() ?? '—',
                        },
                      ]}
                    />
                  </div>
                );
              })}
            </div>
          </Card>
        </>
      )}

      {dataset && !result && analysisStatusIsTerminal(job.status) && job.status !== 'succeeded' && (
        <Card title='No results available'>
          <p className='text-base-content/60 text-sm'>
            This analysis did not complete successfully. See the job status above for error details.
          </p>
        </Card>
      )}
    </>
  );
};

export default AnalysisDetailPage;
