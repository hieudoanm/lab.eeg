import { Suspense } from 'react';

import { DatasetUploadForm } from '@/components/molecules/dataset-upload-form';
import { PageHeader } from '@/components/molecules/page-header';
import { EmptyState, ErrorState, LoadingState } from '@/components/atoms/states';
import { Card } from '@/components/atoms/card';
import { MetadataGrid } from '@/components/atoms/metadata-grid';
import { StatusBadge } from '@/components/atoms/status-badge';
import type { Dataset, DatasetStatus } from '@/lib/api';
import {
  formatBytes,
  formatChecksum,
  formatDateTime,
  formatDuration,
  formatNumber,
} from '@/lib/format';
import { canWriteContent } from '@/lib/roles';
import { datasetStatusLabel, datasetStatusTone } from '@/lib/status';
import { describeError } from '@/trpc/errors';
import { requireSession } from '@/trpc/server';
import { SignalVisualisation } from '@/components/organisms/signal-visualisation';
import { AnalysisActions } from '@/components/molecules/analysis-actions';

const UPLOADABLE_STATUSES: DatasetStatus[] = ['pending_upload', 'rejected', 'failed'];

export const metadata = { title: 'Dataset' };
const DatasetDetailPage = (props: PageProps<'/app/studies/[studyId]/datasets/[datasetId]'>) => {
  return (
    <div className='flex flex-col gap-6'>
      <Suspense fallback={<LoadingState label='Loading dataset…' />}>
        <DatasetDetailContent params={props.params} />
      </Suspense>
    </div>
  );
};

const DatasetDetailContent = async ({
  params,
}: {
  params: PageProps<'/app/studies/[studyId]/datasets/[datasetId]'>['params'];
}) => {
  const { studyId, datasetId } = await params;

  const { caller, membership } = await requireSession();
  if (!membership) {
    return (
      <EmptyState
        title='No organisation'
        description='You are not a member of any organisation yet.'
      />
    );
  }

  let dataset: Dataset | null = null;
  let errorMessage: string | null = null;
  let requestId: string | null = null;
  try {
    dataset = await caller.datasets.byId({ datasetId });
  } catch (error) {
    const described = describeError(error, 'The dataset could not be loaded.');
    errorMessage = described.message;
    requestId = described.requestId;
  }

  if (errorMessage !== null || dataset === null) {
    return (
      <ErrorState
        message={errorMessage ?? 'The dataset could not be loaded.'}
        requestId={requestId}
      />
    );
  }

  const samplingFrequency =
    dataset.sampling_frequency_hz === null
      ? '—'
      : `${formatNumber(dataset.sampling_frequency_hz)} Hz`;

  const canWrite = canWriteContent(membership.role);
  const canUpload = canWrite && UPLOADABLE_STATUSES.includes(dataset.status);

  return (
    <>
      <PageHeader
        title={dataset.name}
        description={dataset.description ?? 'Recording summary and lifecycle state.'}
        breadcrumbs={[
          { label: 'Studies', href: '/app/studies' },
          { label: 'Study', href: `/app/studies/${studyId}` },
          { label: dataset.name },
        ]}
      />

      <MetadataGrid
        columns={3}
        items={[
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
          { label: 'Sampling frequency', value: samplingFrequency },
          { label: 'Channels', value: dataset.channel_count ?? '—' },
          { label: 'Samples', value: dataset.sample_count ?? '—' },
          { label: 'Duration', value: formatDuration(dataset.duration_seconds) },
          { label: 'File size', value: formatBytes(dataset.file_size_bytes) },
          {
            label: 'Checksum (SHA-256)',
            value: formatChecksum(dataset.checksum_sha256),
            mono: true,
          },
          { label: 'Source', value: dataset.source_description ?? '—' },
          { label: 'Channel names', value: dataset.channel_names?.join(', ') ?? '—' },
          { label: 'Channel units', value: dataset.channel_units?.join(', ') ?? '—' },
          { label: 'Created', value: formatDateTime(dataset.created_at) },
          { label: 'Updated', value: formatDateTime(dataset.updated_at) },
          { label: 'Validated', value: formatDateTime(dataset.validated_at) },
          { label: 'Dataset ID', value: dataset.id, mono: true },
        ]}
      />

      {canUpload ? (
        <Card
          title='Upload recording'
          description='Upload the NPY recording and validate it against the recording metadata.'>
          {dataset.status === 'rejected' || dataset.status === 'failed' ? (
            <p role='status' className='text-base-content/60 mb-4 text-sm'>
              This dataset has no accepted recording. Upload a corrected recording to validate it.
            </p>
          ) : null}
          <DatasetUploadForm
            organisationId={membership.organisation_id}
            studyId={studyId}
            datasetId={dataset.id}
          />
        </Card>
      ) : null}

      {canWrite && !canUpload && dataset.status === 'validating' ? (
        <Card title='Validating'>
          <p className='text-base-content/60 text-sm'>
            Validation is running. Actions that need a ready recording will be available shortly.
          </p>
        </Card>
      ) : null}

      {dataset.status === 'ready' && dataset.sampling_frequency_hz && dataset.channel_count && (
        <>
          <Card title='Signal visualisation'>
            <SignalVisualisation
              datasetId={dataset.id}
              samplingFrequencyHz={dataset.sampling_frequency_hz}
              channelNames={dataset.channel_names ?? []}
              channelUnits={dataset.channel_units ?? []}
              durationSeconds={dataset.duration_seconds}
            />
          </Card>

          {canWrite && (
            <AnalysisActions
              datasetId={dataset.id}
              datasetName={dataset.name}
              samplingFrequencyHz={dataset.sampling_frequency_hz}
              channelCount={dataset.channel_count}
            />
          )}
        </>
      )}
    </>
  );
};

export default DatasetDetailPage;
