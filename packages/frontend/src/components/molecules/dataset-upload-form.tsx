'use client';

import { useRouter } from 'next/navigation';
import { useState } from 'react';

import { Button } from '@/components/atoms/button';
import { TextField } from '@/components/atoms/text-field';
import { TextareaField } from '@/components/atoms/textarea-field';
import { useToast } from '@/components/atoms/toast';
import { cn } from '@/lib/cn';
import { formatBytes } from '@/lib/format';
import {
  ACCEPTED_RECORDING_ATTRIBUTE,
  MAX_UPLOAD_BYTES,
  parseChannelList,
  uploadRecording,
  validateRecordingFile,
} from '@/lib/upload';
import { api } from '@/trpc/react';

type Stage = 'idle' | 'creating' | 'uploading' | 'validating' | 'ready' | 'error';

interface CreateUploadSessionInput {
  datasetId: string;
  fileName: string;
  fileSizeBytes: number;
  samplingFrequencyHz: number;
  channelNames: string[];
  channelUnits: string[] | null;
}

const errorMessage = (error: unknown, fallback: string): string =>
  error instanceof Error && error.message.length > 0 ? error.message : fallback;

const STEPS = ['Upload', 'Validation', 'Ready'] as const;

export interface DatasetUploadFormProps {
  organisationId: string;
  studyId: string;
  datasetId: string;
}

const stepIndex = (stage: Stage): number => {
  if (stage === 'ready') {
    return 2;
  }
  if (stage === 'validating') {
    return 1;
  }
  return 0;
};

export const DatasetUploadForm = ({
  organisationId,
  studyId,
  datasetId,
}: DatasetUploadFormProps) => {
  const router = useRouter();
  const { showToast } = useToast();

  const [file, setFile] = useState<File | null>(null);
  const [samplingFrequency, setSamplingFrequency] = useState('');
  const [channelNames, setChannelNames] = useState('');
  const [channelUnits, setChannelUnits] = useState('');
  const [stage, setStage] = useState<Stage>('idle');
  const [progress, setProgress] = useState(0);
  const [formError, setFormError] = useState<string | null>(null);
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});

  const busy = stage === 'creating' || stage === 'uploading' || stage === 'validating';

  const createSession = api.uploads.createSession.useMutation();
  const finalise = api.uploads.finalise.useMutation();

  const selectFile = (selected: File | null) => {
    setFormError(null);
    setFieldErrors((current) => ({ ...current, file: '' }));
    if (!selected) {
      setFile(null);
      return;
    }
    const problem = validateRecordingFile(selected);
    if (problem) {
      setFile(null);
      setFieldErrors((current) => ({ ...current, file: problem }));
      return;
    }
    setFile(selected);
  };

  const validate = (): { ok: true; input: CreateUploadSessionInput } | { ok: false } => {
    const errors: Record<string, string> = {};
    if (!file) {
      errors.file = 'Choose a recording to upload.';
    } else {
      const problem = validateRecordingFile(file);
      if (problem) {
        errors.file = problem;
      }
    }

    const frequency = Number.parseFloat(samplingFrequency);
    if (!samplingFrequency.trim() || !Number.isFinite(frequency) || frequency <= 0) {
      errors.samplingFrequency = 'Enter a sampling frequency greater than zero.';
    }

    const names = parseChannelList(channelNames);
    if (names.length === 0) {
      errors.channelNames = 'List at least one channel name, one per line.';
    } else if (names.length > 256) {
      errors.channelNames = 'Recordings are limited to 256 channels.';
    } else if (names.some((name) => name.length > 100)) {
      errors.channelNames = 'Channel names are limited to 100 characters.';
    }

    const units = parseChannelList(channelUnits);
    if (units.length > 0 && units.length !== names.length) {
      errors.channelUnits = 'Provide a unit for every channel, or leave units blank.';
    }

    setFieldErrors(errors);
    if (Object.keys(errors).length > 0 || !file) {
      return { ok: false };
    }

    return {
      ok: true,
      input: {
        datasetId,
        fileName: file.name,
        fileSizeBytes: file.size,
        samplingFrequencyHz: frequency,
        channelNames: names,
        channelUnits: units.length > 0 ? units : null,
      },
    };
  };

  const submit = async () => {
    setFormError(null);
    const validation = validate();
    if (!validation.ok || !file) {
      setStage('idle');
      return;
    }

    setStage('creating');
    let uploadSessionId: string;
    try {
      const session = await createSession.mutateAsync(validation.input);
      uploadSessionId = session.upload_session_id;
    } catch (error) {
      setStage('error');
      setFormError(errorMessage(error, 'The upload could not be started.'));
      return;
    }

    setStage('uploading');
    setProgress(0);
    const target = `/api/uploads/content?${new URLSearchParams({
      organisationId,
      datasetId,
      sessionId: uploadSessionId,
    }).toString()}`;
    const upload = await uploadRecording(target, file, { onProgress: setProgress });
    if (!upload.ok) {
      setStage('error');
      setFormError(upload.message ?? 'The recording could not be uploaded.');
      return;
    }

    setStage('validating');
    try {
      await finalise.mutateAsync({ studyId, datasetId, sessionId: uploadSessionId });
      setStage('ready');
      showToast({
        title: 'Recording ready',
        description: `${file.name} validated successfully.`,
        tone: 'success',
      });
    } catch (error) {
      setStage('error');
      setFormError(errorMessage(error, 'The recording failed validation.'));
    }
    router.refresh();
  };

  const activeStep = stepIndex(stage);

  return (
    <form
      className='flex flex-col gap-4'
      onSubmit={(event) => {
        event.preventDefault();
        void submit();
      }}>
      <ul className='steps text-xs'>
        {STEPS.map((step, index) => (
          <li
            key={step}
            aria-current={stage !== 'error' && index === activeStep ? 'step' : undefined}
            className={cn('step', stage !== 'error' && index <= activeStep && 'step-primary')}>
            {step}
          </li>
        ))}
      </ul>

      {formError ? (
        <p role='alert' className='text-error text-sm'>
          {formError}
        </p>
      ) : null}
      {stage === 'ready' ? (
        <p role='status' className='text-success text-sm'>
          Recording validated and ready.
        </p>
      ) : null}

      <div className='flex flex-col gap-1.5'>
        <label htmlFor='recording' className='text-base-content text-sm font-medium'>
          Recording file
          <span aria-hidden='true' className='text-error ml-1'>
            *
          </span>
        </label>
        <p className='text-base-content/60 text-xs'>
          Supported format: NPY (.npy). Maximum size: {formatBytes(MAX_UPLOAD_BYTES)}.
        </p>
        <input
          id='recording'
          type='file'
          accept={ACCEPTED_RECORDING_ATTRIBUTE}
          disabled={busy}
          onChange={(event) => selectFile(event.target.files?.[0] ?? null)}
          aria-invalid={fieldErrors.file ? true : undefined}
          className={cn(
            'file-input w-full disabled:cursor-not-allowed disabled:opacity-60',
            fieldErrors.file ? 'file-input-error' : '',
          )}
        />
        {fieldErrors.file ? (
          <p role='alert' className='text-error text-xs'>
            {fieldErrors.file}
          </p>
        ) : null}
        {file ? (
          <div className='text-base-content/60 flex flex-wrap items-center gap-2 text-xs'>
            <span className='text-base-content font-medium'>{file.name}</span>
            <span>{formatBytes(file.size)}</span>
            <span className='uppercase'>.npy</span>
            <Button
              type='button'
              variant='ghost'
              size='sm'
              disabled={busy}
              onClick={() => selectFile(null)}>
              Remove
            </Button>
          </div>
        ) : null}
      </div>

      <TextField
        name='sampling_frequency_hz'
        label='Sampling frequency (Hz)'
        required
        inputMode='decimal'
        value={samplingFrequency}
        onChange={(event) => setSamplingFrequency(event.target.value)}
        disabled={busy}
        error={fieldErrors.samplingFrequency}
        hint='Must match the recording; the .npy array stores no sampling rate.'
      />

      <TextareaField
        name='channel_names'
        label='Channel names'
        required
        value={channelNames}
        onChange={(event) => setChannelNames(event.target.value)}
        disabled={busy}
        error={fieldErrors.channelNames}
        hint='One channel per line, in array order. Must match the first array dimension.'
      />

      <TextareaField
        name='channel_units'
        label='Channel units'
        value={channelUnits}
        onChange={(event) => setChannelUnits(event.target.value)}
        disabled={busy}
        error={fieldErrors.channelUnits}
        hint='Optional. One unit per channel (µV, mV, …); leave blank if unknown.'
      />

      {stage === 'uploading' ? (
        <div className='flex items-center gap-3'>
          <progress
            value={progress}
            max={100}
            aria-label='Upload progress'
            className='progress progress-primary w-full'
          />
          <span className='text-base-content/60 text-xs'>{progress}%</span>
        </div>
      ) : null}

      <div className='flex flex-wrap items-center gap-3'>
        <Button type='submit' disabled={busy}>
          {stage === 'creating'
            ? 'Preparing…'
            : stage === 'uploading'
              ? 'Uploading…'
              : stage === 'validating'
                ? 'Validating…'
                : stage === 'error'
                  ? 'Try again'
                  : 'Upload recording'}
        </Button>
        {busy ? (
          <span role='status' className='text-base-content/60 text-xs'>
            Do not close this page until the upload completes.
          </span>
        ) : null}
      </div>
    </form>
  );
};
