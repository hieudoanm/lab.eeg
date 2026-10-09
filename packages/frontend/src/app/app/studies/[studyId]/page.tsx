import Link from 'next/link';
import { Suspense } from 'react';

import { ArchiveStudyButton } from '@/components/atoms/archive-study-button';
import { DatasetForm } from '@/components/molecules/dataset-form';
import { PageHeader } from '@/components/molecules/page-header';
import { EmptyState, ErrorState, LoadingState } from '@/components/atoms/states';
import { Card } from '@/components/atoms/card';
import { DataTable } from '@/components/atoms/data-table';
import { MetadataGrid } from '@/components/atoms/metadata-grid';
import { StatusBadge } from '@/components/atoms/status-badge';
import { StudyForm } from '@/components/molecules/study-form';
import type { Dataset, Study } from '@/lib/api';
import { formatDateTime } from '@/lib/format';
import { canWriteContent } from '@/lib/roles';
import {
  datasetStatusLabel,
  datasetStatusTone,
  studyStatusLabel,
  studyStatusTone,
} from '@/lib/status';
import { describeError } from '@/trpc/errors';
import { requireSession } from '@/trpc/server';

export const metadata = { title: 'Study' };

const StudyDetailPage = (props: PageProps<'/app/studies/[studyId]'>) => {
  return (
    <div className='flex flex-col gap-6'>
      <Suspense fallback={<LoadingState label='Loading study…' />}>
        <StudyDetailContent params={props.params} />
      </Suspense>
    </div>
  );
};

const StudyDetailContent = async ({
  params,
}: {
  params: PageProps<'/app/studies/[studyId]'>['params'];
}) => {
  const { studyId } = await params;

  const { caller, membership } = await requireSession();
  if (!membership) {
    return (
      <EmptyState
        title='No organisation'
        description='You are not a member of any organisation yet.'
      />
    );
  }

  let study: Study | null = null;
  let datasets: Dataset[] = [];
  let errorMessage: string | null = null;
  let requestId: string | null = null;
  try {
    const [studyResult, datasetResult] = await Promise.all([
      caller.studies.byId({ studyId }),
      caller.datasets.list({ studyId }),
    ]);
    study = studyResult;
    datasets = datasetResult.items;
  } catch (error) {
    const described = describeError(error, 'The study could not be loaded.');
    errorMessage = described.message;
    requestId = described.requestId;
  }

  if (errorMessage !== null || study === null) {
    return (
      <ErrorState
        message={errorMessage ?? 'The study could not be loaded.'}
        requestId={requestId}
      />
    );
  }

  const canWrite = canWriteContent(membership.role);
  const isArchived = study.status === 'archived';

  return (
    <>
      <PageHeader
        title={study.name}
        description={study.description ?? 'Study workspace.'}
        breadcrumbs={[{ label: 'Studies', href: '/app/studies' }, { label: study.name }]}
      />

      <MetadataGrid
        columns={3}
        items={[
          {
            label: 'Status',
            value: (
              <StatusBadge
                tone={studyStatusTone(study.status)}
                label={studyStatusLabel(study.status)}
              />
            ),
          },
          { label: 'Created', value: formatDateTime(study.created_at) },
          { label: 'Updated', value: formatDateTime(study.updated_at) },
          { label: 'Study ID', value: study.id, mono: true },
        ]}
      />

      {isArchived ? (
        <Card title='Archived study'>
          <p className='text-base-content/60 text-sm'>
            This study is archived and read-only. Its datasets, results, and history are preserved.
          </p>
        </Card>
      ) : null}

      {canWrite && !isArchived ? (
        <Card title='Edit study' description='Rename the study or update its description.'>
          <StudyForm study={study} />
        </Card>
      ) : null}

      {canWrite && !isArchived ? (
        <Card
          title='Archive'
          description='Archiving keeps all datasets and history but makes the study read-only.'>
          <ArchiveStudyButton studyId={study.id} />
        </Card>
      ) : null}

      {canWrite && !isArchived ? (
        <Card
          title='Register dataset'
          description='Register a dataset, then upload its EEG recording.'>
          <DatasetForm studyId={study.id} />
        </Card>
      ) : null}

      <Card
        title='Datasets'
        description={`${datasets.length} dataset${datasets.length === 1 ? '' : 's'}`}>
        {datasets.length === 0 ? (
          <EmptyState
            title='No datasets yet'
            description='Register a dataset to associate an EEG recording with this study.'
          />
        ) : (
          <DataTable columns={['Name', 'Format', 'Status', 'Created']}>
            {datasets.map((dataset) => (
              <tr key={dataset.id} className='hover:bg-base-200'>
                <td className='px-3 py-3'>
                  <Link
                    href={`/app/studies/${studyId}/datasets/${dataset.id}`}
                    className='text-base-content font-medium hover:underline'>
                    {dataset.name}
                  </Link>
                  {dataset.description ? (
                    <p className='text-base-content/60 mt-0.5 text-xs'>{dataset.description}</p>
                  ) : null}
                </td>
                <td className='text-base-content/60 px-3 py-3 uppercase'>{dataset.format}</td>
                <td className='px-3 py-3'>
                  <StatusBadge
                    tone={datasetStatusTone(dataset.status)}
                    label={datasetStatusLabel(dataset.status)}
                  />
                </td>
                <td className='text-base-content/60 px-3 py-3'>
                  {formatDateTime(dataset.created_at)}
                </td>
              </tr>
            ))}
          </DataTable>
        )}
      </Card>
    </>
  );
};

export default StudyDetailPage;
