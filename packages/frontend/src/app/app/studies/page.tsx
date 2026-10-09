import Link from 'next/link';
import { Suspense } from 'react';

import { PageHeader } from '@/components/molecules/page-header';
import { EmptyState, ErrorState, LoadingState, NoResultsState } from '@/components/atoms/states';
import { StudyForm } from '@/components/molecules/study-form';
import { buttonClasses, Button } from '@/components/atoms/button';
import { Card } from '@/components/atoms/card';
import { DataTable } from '@/components/atoms/data-table';
import { SelectField, type SelectOption } from '@/components/atoms/select-field';
import { StatusBadge } from '@/components/atoms/status-badge';
import { TextField } from '@/components/atoms/text-field';
import type { Study, StudyStatus } from '@/lib/api';
import { formatDateTime } from '@/lib/format';
import { canWriteContent } from '@/lib/roles';
import { studyStatusLabel, studyStatusTone } from '@/lib/status';
import { describeError } from '@/trpc/errors';
import { requireSession } from '@/trpc/server';

export const metadata = { title: 'Studies' };

const STATUS_OPTIONS: SelectOption[] = [
  { value: '', label: 'All statuses' },
  { value: 'active', label: 'Active' },
  { value: 'archived', label: 'Archived' },
];

const first = (value: string | string[] | undefined): string => {
  if (Array.isArray(value)) {
    return value[0] ?? '';
  }
  return value ?? '';
};

const asStatus = (value: string): StudyStatus | undefined => {
  return value === 'active' || value === 'archived' ? value : undefined;
};

const buildHref = (params: { search: string; status?: string; cursor?: string }): string => {
  const query = new URLSearchParams();
  if (params.search) {
    query.set('search', params.search);
  }
  if (params.status) {
    query.set('status', params.status);
  }
  if (params.cursor) {
    query.set('cursor', params.cursor);
  }
  const serialised = query.toString();
  return serialised.length > 0 ? `/app/studies?${serialised}` : '/app/studies';
};

const StudiesPage = (props: PageProps<'/app/studies'>) => {
  return (
    <div className='flex flex-col gap-6'>
      <PageHeader title='Studies' description='Organise EEG recordings into research studies.' />
      <Suspense fallback={<LoadingState label='Loading studies…' />}>
        <StudiesContent searchParams={props.searchParams} />
      </Suspense>
    </div>
  );
};

const StudiesContent = async ({
  searchParams,
}: {
  searchParams: PageProps<'/app/studies'>['searchParams'];
}) => {
  const params = await searchParams;
  const search = first(params.search).trim();
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

  let studies: Study[] = [];
  let nextCursor: string | null = null;
  let errorMessage: string | null = null;
  let requestId: string | null = null;
  try {
    const page = await caller.studies.list({
      search: search || undefined,
      status,
      cursor,
    });
    studies = page.items;
    nextCursor = page.next_cursor;
  } catch (error) {
    const described = describeError(error, 'The studies could not be loaded.');
    errorMessage = described.message;
    requestId = described.requestId;
  }

  if (errorMessage !== null) {
    return <ErrorState message={errorMessage} requestId={requestId} />;
  }

  const canWrite = canWriteContent(membership.role);
  const hasFilters = search.length > 0 || status !== undefined;

  return (
    <div className='flex flex-col gap-6'>
      <form
        method='get'
        action='/app/studies'
        className='flex flex-wrap items-end gap-3'
        aria-label='Filter studies'>
        <div className='w-full sm:w-64'>
          <TextField
            name='search'
            label='Search'
            defaultValue={search}
            placeholder='Study name'
            autoComplete='off'
          />
        </div>
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

      {canWrite ? (
        <Card title='New study' description='Create a study to group related datasets.'>
          <StudyForm />
        </Card>
      ) : null}

      <Card title='Studies' description={hasFilters ? 'Filtered results' : 'All studies'}>
        {studies.length === 0 ? (
          hasFilters ? (
            <NoResultsState description='No studies match the current filters.' />
          ) : (
            <EmptyState
              title='No studies yet'
              description='Create the first study to start organising EEG recordings.'
            />
          )
        ) : (
          <>
            <DataTable columns={['Name', 'Status', 'Updated']}>
              {studies.map((study) => (
                <tr key={study.id} className='hover:bg-base-200'>
                  <td className='px-3 py-3'>
                    <Link
                      href={`/app/studies/${study.id}`}
                      className='text-base-content font-medium hover:underline'>
                      {study.name}
                    </Link>
                    {study.description ? (
                      <p className='text-base-content/60 mt-0.5 text-xs'>{study.description}</p>
                    ) : null}
                  </td>
                  <td className='px-3 py-3'>
                    <StatusBadge
                      tone={studyStatusTone(study.status)}
                      label={studyStatusLabel(study.status)}
                    />
                  </td>
                  <td className='text-base-content/60 px-3 py-3'>
                    {formatDateTime(study.updated_at)}
                  </td>
                </tr>
              ))}
            </DataTable>
            {nextCursor ? (
              <div className='mt-4 flex justify-end'>
                <Link
                  href={buildHref({ search, status, cursor: nextCursor })}
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

export default StudiesPage;
