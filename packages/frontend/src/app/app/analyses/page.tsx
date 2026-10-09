import Link from 'next/link';
import { Suspense } from 'react';

import { PageHeader } from '@/components/molecules/page-header';
import { EmptyState, ErrorState, LoadingState, NoResultsState } from '@/components/atoms/states';
import { buttonClasses, Button } from '@/components/atoms/button';
import { Card } from '@/components/atoms/card';
import { DataTable } from '@/components/atoms/data-table';
import { SelectField, type SelectOption } from '@/components/atoms/select-field';
import { AnalysisStatusBadge } from '@/components/atoms/analysis-status';
import type { AnalysisJob, AnalysisJobStatus } from '@/lib/api';
import { formatDateTime } from '@/lib/format';
import { describeError } from '@/trpc/errors';
import { requireSession } from '@/trpc/server';

export const metadata = { title: 'Analyses' };

const STATUS_OPTIONS: SelectOption[] = [
  { value: '', label: 'All statuses' },
  { value: 'queued', label: 'Queued' },
  { value: 'running', label: 'Running' },
  { value: 'succeeded', label: 'Succeeded' },
  { value: 'failed', label: 'Failed' },
  { value: 'cancelled', label: 'Cancelled' },
];

const STATUSES: AnalysisJobStatus[] = ['queued', 'running', 'succeeded', 'failed', 'cancelled'];

const first = (value: string | string[] | undefined): string => {
  if (Array.isArray(value)) {
    return value[0] ?? '';
  }
  return value ?? '';
};

const asStatus = (value: string): AnalysisJobStatus | undefined => {
  return STATUSES.includes(value as AnalysisJobStatus) ? (value as AnalysisJobStatus) : undefined;
};

const buildHref = (params: { status?: string; cursor?: string }): string => {
  const query = new URLSearchParams();
  if (params.status) {
    query.set('status', params.status);
  }
  if (params.cursor) {
    query.set('cursor', params.cursor);
  }
  const serialised = query.toString();
  return serialised.length > 0 ? `/app/analyses?${serialised}` : '/app/analyses';
};

const AnalysesPage = (props: PageProps<'/app/analyses'>) => {
  return (
    <div className='flex flex-col gap-6'>
      <PageHeader
        title='Analyses'
        description='Descriptive analysis jobs submitted across this organisation.'
      />
      <Suspense fallback={<LoadingState label='Loading analyses…' />}>
        <AnalysesContent searchParams={props.searchParams} />
      </Suspense>
    </div>
  );
};

const AnalysesContent = async ({
  searchParams,
}: {
  searchParams: PageProps<'/app/analyses'>['searchParams'];
}) => {
  const params = await searchParams;
  const status = asStatus(first(params.status));
  const cursor = first(params.cursor) || undefined;

  const { caller, membership } = await requireSession();
  if (!membership) {
    return (
      <EmptyState
        title='No organisation'
        description='You are not a member of any organisation yet.'
      />
    );
  }

  let jobs: AnalysisJob[] = [];
  let nextCursor: string | null = null;
  let errorMessage: string | null = null;
  let requestId: string | null = null;
  try {
    const page = await caller.analyses.listOrganisation({ status, cursor, limit: 25 });
    jobs = page.items;
    nextCursor = page.next_cursor;
  } catch (error) {
    const described = describeError(error, 'The analyses could not be loaded.');
    errorMessage = described.message;
    requestId = described.requestId;
  }

  if (errorMessage !== null) {
    return <ErrorState message={errorMessage} requestId={requestId} />;
  }

  const hasFilters = status !== undefined;

  return (
    <div className='flex flex-col gap-6'>
      <form
        method='get'
        action='/app/analyses'
        className='flex flex-wrap items-end gap-3'
        aria-label='Filter analyses'>
        <div className='w-full sm:w-48'>
          <SelectField
            name='status'
            label='Status'
            options={STATUS_OPTIONS}
            defaultValue={status ?? ''}
          />
        </div>
        <Button type='submit' variant='secondary'>
          Apply
        </Button>
      </form>

      <Card title='Analyses' description={hasFilters ? 'Filtered results' : 'All analyses'}>
        {jobs.length === 0 ? (
          hasFilters ? (
            <NoResultsState description='No analyses match the current filters.' />
          ) : (
            <EmptyState
              title='No analyses yet'
              description='Submit an analysis from a dataset page to see it here.'
            />
          )
        ) : (
          <>
            <DataTable columns={['Status', 'Type', 'Dataset', 'Requested']}>
              {jobs.map((job) => (
                <tr key={job.id} className='hover:bg-base-200'>
                  <td className='px-3 py-3'>
                    <Link href={`/app/analyses/${job.id}`} className='hover:underline'>
                      <AnalysisStatusBadge status={job.status} />
                    </Link>
                  </td>
                  <td className='text-base-content/80 px-3 py-3'>{job.analysis_type}</td>
                  <td className='text-base-content/60 px-3 py-3 font-mono text-sm'>
                    {job.dataset_id.slice(0, 8)}
                  </td>
                  <td className='text-base-content/60 px-3 py-3'>
                    {formatDateTime(job.requested_at)}
                  </td>
                </tr>
              ))}
            </DataTable>
            {nextCursor ? (
              <div className='mt-4 flex justify-end'>
                <Link
                  href={buildHref({ status, cursor: nextCursor })}
                  className={buttonClasses('secondary', 'sm')}>
                  Next page
                </Link>
              </div>
            ) : null}
          </>
        )}
      </Card>
    </div>
  );
};

export default AnalysesPage;
