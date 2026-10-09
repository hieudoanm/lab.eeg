'use client';

import { useRouter } from 'next/navigation';
import { useState, type FormEvent } from 'react';

import { Button } from '@/components/atoms/button';
import { TextField } from '@/components/atoms/text-field';
import type { Study } from '@/lib/api';
import { api } from '@/trpc/react';

export interface StudyFormProps {
  /** When provided, the form updates the study instead of creating one. */
  study?: Pick<Study, 'id' | 'name' | 'description'>;
}

const readFields = (
  formData: FormData,
): { ok: true; name: string; description: string | null } | { ok: false; message: string } => {
  const rawName = formData.get('name');
  const name = typeof rawName === 'string' ? rawName.trim() : '';
  if (name.length === 0) {
    return { ok: false, message: 'Enter a study name.' };
  }
  if (name.length > 200) {
    return { ok: false, message: 'Study names are limited to 200 characters.' };
  }

  const rawDescription = formData.get('description');
  const description =
    typeof rawDescription === 'string' && rawDescription.trim().length > 0
      ? rawDescription.trim()
      : null;
  if (description !== null && description.length > 2000) {
    return { ok: false, message: 'Descriptions are limited to 2000 characters.' };
  }

  return { ok: true, name, description };
};

export const StudyForm = ({ study }: StudyFormProps) => {
  const router = useRouter();
  const editing = study !== undefined;
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const create = api.studies.create.useMutation({
    onSuccess: (created) => {
      router.push(`/app/studies/${created.id}`);
      router.refresh();
    },
    onError: (mutationError) => {
      setError(mutationError.message || 'The study could not be created.');
    },
  });
  const update = api.studies.update.useMutation({
    onSuccess: () => {
      setSuccess('Study updated.');
      router.refresh();
    },
    onError: (mutationError) => {
      setError(mutationError.message || 'The study could not be updated.');
    },
  });

  const pending = create.isPending || update.isPending;

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setError(null);
    setSuccess(null);

    const fields = readFields(new FormData(event.currentTarget));
    if (!fields.ok) {
      setError(fields.message);
      return;
    }

    if (study) {
      update.mutate({ studyId: study.id, name: fields.name, description: fields.description });
    } else {
      create.mutate({ name: fields.name, description: fields.description });
    }
  };

  return (
    <form className='flex flex-col gap-4' onSubmit={handleSubmit}>
      {error ? (
        <p role='alert' className='text-error text-sm'>
          {error}
        </p>
      ) : null}
      {success ? (
        <p role='status' className='text-success text-sm'>
          {success}
        </p>
      ) : null}

      <TextField
        name='name'
        label='Name'
        required
        maxLength={200}
        hint='Up to 200 characters.'
        autoComplete='off'
        defaultValue={study?.name}
      />
      <TextField
        name='description'
        label='Description'
        maxLength={2000}
        hint='Optional. Up to 2000 characters.'
        autoComplete='off'
        defaultValue={study?.description ?? ''}
      />

      <div>
        <Button type='submit' disabled={pending}>
          {pending
            ? editing
              ? 'Saving…'
              : 'Creating…'
            : editing
              ? 'Save changes'
              : 'Create study'}
        </Button>
      </div>
    </form>
  );
};
