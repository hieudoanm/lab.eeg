'use client';

import { useRouter } from 'next/navigation';
import { useState, type FormEvent } from 'react';

import { Button } from '@/components/atoms/button';
import { TextField } from '@/components/atoms/text-field';
import { TextareaField } from '@/components/atoms/textarea-field';
import { api } from '@/trpc/react';

export interface DatasetFormProps {
  studyId: string;
}

const readFields = (
  formData: FormData,
):
  | { ok: true; name: string; description: string | null; sourceDescription: string | null }
  | { ok: false; message: string } => {
  const rawName = formData.get('name');
  const name = typeof rawName === 'string' ? rawName.trim() : '';
  if (name.length === 0) {
    return { ok: false, message: 'Enter a dataset name.' };
  }
  if (name.length > 200) {
    return { ok: false, message: 'Dataset names are limited to 200 characters.' };
  }

  const rawDescription = formData.get('description');
  const description =
    typeof rawDescription === 'string' && rawDescription.trim().length > 0
      ? rawDescription.trim()
      : null;
  if (description !== null && description.length > 2000) {
    return { ok: false, message: 'Descriptions are limited to 2000 characters.' };
  }

  const rawSource = formData.get('source_description');
  const sourceDescription =
    typeof rawSource === 'string' && rawSource.trim().length > 0 ? rawSource.trim() : null;
  if (sourceDescription !== null && sourceDescription.length > 2000) {
    return { ok: false, message: 'Source notes are limited to 2000 characters.' };
  }

  return { ok: true, name, description, sourceDescription };
};

export const DatasetForm = ({ studyId }: DatasetFormProps) => {
  const router = useRouter();
  const [error, setError] = useState<string | null>(null);
  const register = api.datasets.register.useMutation({
    onSuccess: (dataset) => {
      router.push(`/app/studies/${studyId}/datasets/${dataset.id}`);
      router.refresh();
    },
    onError: (mutationError) => {
      setError(mutationError.message || 'The dataset could not be registered.');
    },
  });

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setError(null);

    const fields = readFields(new FormData(event.currentTarget));
    if (!fields.ok) {
      setError(fields.message);
      return;
    }

    register.mutate({
      studyId,
      name: fields.name,
      description: fields.description,
      sourceDescription: fields.sourceDescription,
    });
  };

  return (
    <form className='flex flex-col gap-4' onSubmit={handleSubmit}>
      {error ? (
        <p role='alert' className='text-error text-sm'>
          {error}
        </p>
      ) : null}

      <TextField
        name='name'
        label='Name'
        required
        maxLength={200}
        hint='A short identifier for the recording, such as the session or subject.'
        autoComplete='off'
      />
      <TextareaField
        name='description'
        label='Description'
        maxLength={2000}
        hint='Optional. Up to 2000 characters.'
      />
      <TextareaField
        name='source_description'
        label='Source'
        maxLength={2000}
        hint='Optional. Capture device, montage, or acquisition notes.'
      />
      <p className='text-base-content/60 text-xs'>
        Format: NPY (.npy). The recording is uploaded after the dataset is registered.
      </p>

      <div>
        <Button type='submit' disabled={register.isPending}>
          {register.isPending ? 'Registering…' : 'Register dataset'}
        </Button>
      </div>
    </form>
  );
};
