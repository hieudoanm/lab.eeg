import Link from 'next/link';
import { Suspense } from 'react';

import { PageHeader } from '@/components/molecules/page-header';
import { EmptyState, ErrorState, LoadingState } from '@/components/atoms/states';
import { Card } from '@/components/atoms/card';
import { AnalysisStatusBadge } from '@/components/atoms/analysis-status';
import type { AnalysisJob, OrganisationDashboardSummary, Study } from '@/lib/api';
import { formatDateTime } from '@/lib/format';
import { describeError } from '@/trpc/errors';
import { requireSession } from '@/trpc/server';

export const metadata = { title: 'Dashboard' };

const RECENT_LIMIT = 5;

const Stat = ({ label, value, hint }: { label: string; value: string; hint: string }) => {
  return (
    <div className='border-base-300 bg-base-100 rounded-[var(--radius-card)] border p-5'>
      <p className='text-base-content/60 text-xs font-medium tracking-wide uppercase'>{label}</p>
      <p className='text-base-content mt-2 text-2xl font-semibold'>{value}</p>
      <p className='text-base-content/60 mt-1 text-xs'>{hint}</p>
    </div>
  );
};

const DashboardPage = () => {
  return (
    <div className='flex flex-col gap-8'>
      <PageHeader
        title='Dashboard'
        description='An overview of research activity in the active organisation.'
      />
      <Suspense fallback={<LoadingState label='Loading dashboard…' />}>
        <DashboardContent />
      </Suspense>
    </div>
  );
};

const DashboardContent = async () => {
  const { caller, membership } = await requireSession();
  if (!membership) {
    return (
      <EmptyState
        title='No organisation'
        description='You are not a member of any organisation yet.'
      />
    );
  }

  let summary: OrganisationDashboardSummary | null = null;
  let studies: Study[] = [];
  let analyses: AnalysisJob[] = [];
  let errorMessage: string | null = null;
  let requestId: string | null = null;
  try {
    [summary, studies, analyses] = await Promise.all([
      caller.summary.organisation(),
      caller.studies.list({ limit: RECENT_LIMIT }).then((page) => page.items),
      caller.analyses.listOrganisation({ limit: RECENT_LIMIT }).then((page) => page.items),
    ]);
  } catch (error) {
    const described = describeError(error, 'The dashboard could not be loaded.');
    errorMessage = described.message;
    requestId = described.requestId;
  }

  if (errorMessage !== null || summary === null) {
    return (
      <ErrorState
        message={errorMessage ?? 'The dashboard could not be loaded.'}
        requestId={requestId}
      />
    );
  }

  return (
    <>
      <section aria-label='Summary' className='grid gap-4 sm:grid-cols-3'>
        <Stat
          label='Studies'
          value={summary.study_count.toLocaleString()}
          hint={`${summary.active_study_count.toLocaleString()} active`}
        />
        <Stat
          label='Datasets'
          value={summary.dataset_count.toLocaleString()}
          hint={`${summary.ready_dataset_count.toLocaleString()} ready`}
        />
        <Stat
          label='Analyses'
          value={summary.analysis_count.toLocaleString()}
          hint={`${summary.analyses_by_status.succeeded.toLocaleString()} succeeded · ${summary.analyses_by_status.failed.toLocaleString()} failed`}
        />
      </section>

      <Card title='Recent studies' description='The most recently updated studies.'>
        {studies.length === 0 ? (
          <EmptyState
            title='No studies yet'
            description='Create a study to start organising EEG recordings.'
          />
        ) : (
          <ul className='divide-base-300 divide-y'>
            {studies.map((study) => (
              <li key={study.id} className='flex items-center justify-between gap-4 py-3'>
                <Link
                  href={`/app/studies/${study.id}`}
                  className='text-base-content font-medium hover:underline'>
                  {study.name}
                </Link>
                <span className='text-base-content/60 text-sm'>
                  {formatDateTime(study.updated_at)}
                </span>
              </li>
            ))}
          </ul>
        )}
      </Card>

      <Card title='Recent analyses' description='The most recent analysis jobs.'>
        {analyses.length === 0 ? (
          <EmptyState
            title='No analyses yet'
            description='Submit an analysis from a dataset page to see it here.'
          />
        ) : (
          <ul className='divide-base-300 divide-y'>
            {analyses.map((job) => (
              <li key={job.id} className='flex items-center justify-between gap-4 py-3'>
                <Link href={`/app/analyses/${job.id}`} className='hover:underline'>
                  <AnalysisStatusBadge status={job.status} />
                </Link>
                <span className='text-base-content/60 font-mono text-sm'>
                  {job.dataset_id.slice(0, 8)} · {formatDateTime(job.requested_at)}
                </span>
              </li>
            ))}
          </ul>
        )}
      </Card>
    </>
  );
};

export default DashboardPage;
