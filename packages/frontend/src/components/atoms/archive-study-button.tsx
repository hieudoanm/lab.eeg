'use client';

import { useState } from 'react';

import { Button } from '@/components/atoms/button';
import { api } from '@/trpc/react';

export interface ArchiveStudyButtonProps {
  studyId: string;
}

export const ArchiveStudyButton = ({ studyId }: ArchiveStudyButtonProps) => {
  const [confirming, setConfirming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const archiveStudy = api.studies.archive.useMutation({
    onError: (mutationError) => {
      setError(mutationError.message || 'The study could not be archived.');
    },
  });

  const archive = () => {
    setError(null);
    archiveStudy.mutate({ studyId });
  };

  const pending = archiveStudy.isPending;

  if (!confirming) {
    return (
      <div className='flex flex-col items-start gap-2'>
        <Button variant='danger' onClick={() => setConfirming(true)}>
          Archive study
        </Button>
        <p className='text-base-content/60 text-xs'>
          Archived studies are read-only; datasets, results, and history are kept.
        </p>
      </div>
    );
  }

  return (
    <div className='flex flex-col items-start gap-3'>
      <p className='text-base-content text-sm'>Archive this study? It becomes read-only.</p>
      {error !== null ? (
        <p role='alert' className='text-error text-sm'>
          {error}
        </p>
      ) : null}
      <div className='flex gap-2'>
        <Button variant='danger' onClick={archive} disabled={pending}>
          {pending ? 'Archiving…' : 'Confirm archive'}
        </Button>
        <Button variant='secondary' onClick={() => setConfirming(false)} disabled={pending}>
          Cancel
        </Button>
      </div>
    </div>
  );
};
