'use client';

import { useState, useCallback } from 'react';
import { useRouter } from 'next/navigation';
import { Button } from '@/components/atoms/button';
import { Card } from '@/components/atoms/card';
import { LoadingState, ErrorState } from '@/components/atoms/states';
import { api } from '@/trpc/react';
import type { AnalysisJob } from '@/lib/api';
import { formatDateTime } from '@/lib/format';
import {
  AnalysisStatusBadge,
  analysisStatusIsTerminal,
  analysisStatusCanCancel,
} from '@/components/atoms/analysis-status';

export interface AnalysisActionsProps {
  datasetId: string;
  datasetName: string;
  samplingFrequencyHz: number | null;
  channelCount: number | null;
}

const AnalysisActions = ({
  datasetId,
  datasetName,
  samplingFrequencyHz,
  channelCount,
}: AnalysisActionsProps) => {
  const router = useRouter();
  const [idempotencyKey, setIdempotencyKey] = useState(() => `analysis-${Date.now()}`);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

  const {
    data: analyses,
    isLoading,
    error,
    refetch,
  } = api.analyses.list.useQuery({ datasetId, cursor: undefined, limit: 50 }, { enabled: true });

  const submitMutation = api.analyses.submit.useMutation({
    onSuccess: () => {
      setIdempotencyKey(() => `analysis-${Date.now()}`);
      refetch();
      setSubmitError(null);
    },
    onError: (err) => {
      setSubmitError(err.message);
    },
  });

  const cancelMutation = api.analyses.cancel.useMutation({
    onSuccess: () => {
      refetch();
    },
  });

  const handleSubmit = useCallback(() => {
    if (!samplingFrequencyHz || !channelCount) {
      setSubmitError('Dataset must have sampling frequency and channel count');
      return;
    }
    setIsSubmitting(true);
    setSubmitError(null);
    submitMutation.mutate({
      datasetId,
      algorithmVersion: '1.0.0',
      parameters: {
        include_mean: true,
        include_standard_deviation: true,
        include_min_max: true,
        include_peak_to_peak: true,
      },
      idempotencyKey,
    });
    setIsSubmitting(false);
  }, [datasetId, samplingFrequencyHz, channelCount, idempotencyKey, submitMutation]);

  const handleCancel = useCallback(
    (jobId: string) => {
      cancelMutation.mutate({ jobId });
    },
    [cancelMutation],
  );

  const handleViewResults = useCallback(
    (jobId: string) => {
      router.push(`/app/analyses/${jobId}`);
    },
    [router],
  );

  if (isLoading) {
    return <LoadingState label='Loading analyses…' />;
  }

  if (error) {
    return <ErrorState message={error.message} />;
  }

  return (
    <Card title='Analyses' description={`Submitted analyses for ${datasetName}.`}>
      <div className='flex flex-col gap-4'>
        <div className='flex items-center gap-4'>
          <input
            type='text'
            value={idempotencyKey}
            onChange={(e) => setIdempotencyKey(e.target.value)}
            placeholder='Idempotency key (auto-generated)'
            className='input input-sm max-w-xs flex-1'
            readOnly
          />
          <Button
            onClick={handleSubmit}
            disabled={
              isSubmitting || submitMutation.isPending || !samplingFrequencyHz || !channelCount
            }
            className='w-auto'>
            {isSubmitting || submitMutation.isPending ? 'Submitting…' : 'Submit analysis'}
          </Button>
        </div>

        {submitError && (
          <div className='text-error bg-error/10 rounded-md p-3 text-sm' role='alert'>
            {submitError}
          </div>
        )}

        {analyses && analyses.items.length > 0 ? (
          <div className='space-y-3'>
            {analyses.items.map((job: AnalysisJob) => (
              <div
                key={job.id}
                className='border-base-300 flex flex-col gap-2 rounded-lg border p-4 md:flex-row md:items-center md:justify-between'>
                <div className='flex flex-col gap-1 gap-4 md:flex-row md:items-center'>
                  <div className='flex items-center gap-2'>
                    <AnalysisStatusBadge status={job.status} />
                    <span className='font-mono text-sm'>{job.id.slice(0, 8)}</span>
                  </div>
                  <div className='text-base-content/60 text-sm'>
                    {formatDateTime(job.requested_at)} · {job.analysis_type} v
                    {job.algorithm_version} · Attempt {job.attempt_count}
                  </div>
                  {job.error && <div className='text-error text-sm'>{job.error.message}</div>}
                </div>
                <div className='flex items-center gap-2'>
                  {analysisStatusCanCancel(job.status) && (
                    <Button
                      variant='ghost'
                      size='sm'
                      onClick={() => handleCancel(job.id)}
                      disabled={cancelMutation.isPending}>
                      Cancel
                    </Button>
                  )}
                  {analysisStatusIsTerminal(job.status) && job.status === 'succeeded' && (
                    <Button variant='ghost' size='sm' onClick={() => handleViewResults(job.id)}>
                      View results
                    </Button>
                  )}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <p className='text-base-content/60 text-sm'>No analyses submitted yet.</p>
        )}
      </div>
    </Card>
  );
};

export { AnalysisActions };
