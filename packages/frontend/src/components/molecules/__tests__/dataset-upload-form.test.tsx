import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { UploadResult } from '@/lib/upload';

vi.mock('@/trpc/react', async () => {
  const { buildTrpcMock } = await import('@/test/trpc-mock');
  return buildTrpcMock();
});

const { refresh, uploadRecording } = vi.hoisted(() => ({
  refresh: vi.fn(),
  uploadRecording: vi.fn(),
}));

vi.mock('next/navigation', () => ({
  useRouter: () => ({ refresh, push: vi.fn() }),
}));

vi.mock('@/lib/upload', async () => {
  const actual = await vi.importActual<typeof import('@/lib/upload')>('@/lib/upload');
  return { ...actual, uploadRecording };
});

import { DatasetUploadForm } from '@/components/molecules/dataset-upload-form';
import { ToastProvider } from '@/components/atoms/toast';
import { mutationController, resetTrpcMocks } from '@/test/trpc-mock';

const createSession = () => mutationController('uploads.createSession');
const finalise = () => mutationController('uploads.finalise');

const renderForm = () =>
  render(
    <ToastProvider>
      <DatasetUploadForm organisationId='org-1' studyId='s1' datasetId='d1' />
    </ToastProvider>,
  );

const npyFile = (): File => new File([new Uint8Array([1, 2, 3, 4])], 'recording.npy');

const fileInput = (): HTMLInputElement => screen.getByLabelText(/Recording file/);

const chooseFile = (file: File) => fireEvent.change(fileInput(), { target: { files: [file] } });

const setFrequency = (value: string) =>
  fireEvent.change(screen.getByLabelText(/Sampling frequency/), { target: { value } });

const setChannels = (value: string) =>
  fireEvent.change(screen.getByLabelText(/Channel names/), { target: { value } });

const setUnits = (value: string) =>
  fireEvent.change(screen.getByLabelText(/Channel units/), { target: { value } });

const submit = () => {
  const form = fileInput().closest('form');
  if (!form) {
    throw new Error('upload form not found');
  }
  fireEvent.submit(form);
};

const fillValid = () => {
  chooseFile(npyFile());
  setFrequency('250');
  setChannels('Cz\nPz');
};

const alertText = () =>
  screen
    .queryAllByRole('alert')
    .map((element) => element.textContent)
    .join('\n');

beforeEach(() => {
  resetTrpcMocks();
  refresh.mockReset();
  uploadRecording.mockReset();
});

describe('DatasetUploadForm', () => {
  it('renders the recording metadata fields', () => {
    renderForm();

    expect(screen.getByLabelText(/Recording file/)).toBeInTheDocument();
    expect(screen.getByLabelText(/Sampling frequency/)).toBeInTheDocument();
    expect(screen.getByLabelText(/Channel names/)).toBeInTheDocument();
    expect(screen.getByLabelText(/Channel units/)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Upload recording' })).toBeInTheDocument();
    expect(screen.getByText('Upload')).toBeInTheDocument();
    expect(screen.getByText('Validation')).toBeInTheDocument();
    expect(screen.getByText('Ready')).toBeInTheDocument();
  });

  it('rejects an unsupported file format', () => {
    renderForm();

    chooseFile(new File(['data'], 'recording.csv'));

    expect(screen.getByRole('alert')).toHaveTextContent(/supported/i);
  });

  it('rejects an empty recording', () => {
    renderForm();

    chooseFile(new File([], 'empty.npy'));

    expect(screen.getByRole('alert')).toHaveTextContent('The selected file is empty.');
  });

  it('rejects a recording that is too large', () => {
    renderForm();

    const oversized = new File([new Uint8Array([1])], 'huge.npy');
    Object.defineProperty(oversized, 'size', { value: 300_000_000 });
    chooseFile(oversized);

    expect(screen.getByRole('alert')).toHaveTextContent('exceeds the maximum upload size');
  });

  it('shows the selected recording with a remove action', () => {
    renderForm();

    chooseFile(npyFile());

    expect(screen.getByText('recording.npy')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Remove' })).toBeInTheDocument();
  });

  it('removes a selected recording', () => {
    renderForm();

    chooseFile(npyFile());
    fireEvent.click(screen.getByRole('button', { name: 'Remove' }));

    expect(screen.queryByText('recording.npy')).not.toBeInTheDocument();
  });

  it('clears the selected recording when the input is emptied', () => {
    renderForm();

    chooseFile(npyFile());
    fireEvent.change(fileInput(), { target: { files: [] } });

    expect(screen.queryByText('recording.npy')).not.toBeInTheDocument();
  });

  it('reports all missing metadata when submitted empty', () => {
    renderForm();

    submit();

    const text = alertText();
    expect(text).toContain('Choose a recording to upload.');
    expect(text).toContain('Enter a sampling frequency greater than zero.');
    expect(text).toContain('List at least one channel name, one per line.');
    expect(createSession().calls).toHaveLength(0);
  });

  it('rejects a channel unit count that does not match the channel names', () => {
    renderForm();

    chooseFile(npyFile());
    setFrequency('250');
    setChannels('Cz\nPz');
    setUnits('µV');
    submit();

    expect(screen.getByRole('alert')).toHaveTextContent(
      'Provide a unit for every channel, or leave units blank.',
    );
    expect(createSession().calls).toHaveLength(0);
  });

  it('rejects recordings with more than 256 channels', () => {
    renderForm();

    chooseFile(npyFile());
    setFrequency('250');
    setChannels(Array.from({ length: 257 }, (_value, index) => `ch${index}`).join('\n'));
    submit();

    expect(screen.getByRole('alert')).toHaveTextContent('Recordings are limited to 256 channels.');
  });

  it('parses channel names separated by commas and newlines', async () => {
    createSession().rejectWith(new Error('stop'));
    renderForm();

    chooseFile(npyFile());
    setFrequency('250');
    setChannels('Cz, Pz\nFz ');
    submit();

    await waitFor(() =>
      expect(createSession().calls[0]).toEqual({
        datasetId: 'd1',
        fileName: 'recording.npy',
        fileSizeBytes: 4,
        samplingFrequencyHz: 250,
        channelNames: ['Cz', 'Pz', 'Fz'],
        channelUnits: null,
      }),
    );
  });

  it('includes channel units when every channel has one', async () => {
    createSession().rejectWith(new Error('stop'));
    renderForm();

    chooseFile(npyFile());
    setFrequency('250');
    setChannels('Cz\nPz');
    setUnits('µV\nmV');
    submit();

    await waitFor(() =>
      expect(createSession().calls[0]).toEqual(
        expect.objectContaining({ channelUnits: ['µV', 'mV'] }),
      ),
    );
  });

  it('advances through creating, uploading, validating and ready', async () => {
    createSession().resolveWith({ upload_session_id: 'sess-1' });
    finalise().resolveWith(undefined);

    let resolveUpload: (result: UploadResult) => void = () => undefined;
    uploadRecording.mockImplementation(
      (_url: string, _file: File, options?: { onProgress?: (percent: number) => void }) => {
        options?.onProgress?.(37);
        return new Promise<UploadResult>((resolve) => {
          resolveUpload = resolve;
        });
      },
    );

    renderForm();
    fillValid();
    submit();

    await waitFor(() => expect(uploadRecording).toHaveBeenCalledTimes(1));
    expect(screen.getByRole('progressbar', { name: 'Upload progress' })).toBeInTheDocument();
    expect(screen.getByText('37%')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Uploading…' })).toBeDisabled();

    const [target] = uploadRecording.mock.calls[0] as [string, File, unknown];
    expect(target).toContain('/api/uploads/content?');
    expect(target).toContain('organisationId=org-1');
    expect(target).toContain('datasetId=d1');
    expect(target).toContain('sessionId=sess-1');

    act(() => {
      resolveUpload({ ok: true, status: 200, message: null });
    });

    await waitFor(() =>
      expect(finalise().calls[0]).toEqual({ studyId: 's1', datasetId: 'd1', sessionId: 'sess-1' }),
    );
    expect(await screen.findByText('Recording validated and ready.')).toBeInTheDocument();
    expect(screen.getByText('Recording ready')).toBeInTheDocument();
    expect(screen.getByText('recording.npy validated successfully.')).toBeInTheDocument();
    expect(refresh).toHaveBeenCalled();
  });

  it('shows the creating state while the session is being prepared', async () => {
    let rejectSession: (error: Error) => void = () => undefined;
    createSession().resolveWith(
      new Promise((_resolve, reject) => {
        rejectSession = reject;
      }),
    );
    renderForm();
    fillValid();
    submit();

    const button = await screen.findByRole('button', { name: 'Preparing…' });
    expect(button).toBeDisabled();
    expect(fileInput()).toBeDisabled();
    expect(
      screen.getByText('Do not close this page until the upload completes.'),
    ).toBeInTheDocument();

    act(() => {
      rejectSession(new Error('stop'));
    });

    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'Try again' })).toBeInTheDocument(),
    );
  });

  it('shows the validation step while finalising', async () => {
    createSession().resolveWith({ upload_session_id: 'sess-1' });
    uploadRecording.mockResolvedValue({ ok: true, status: 200, message: null });
    let resolveFinalise: (value: undefined) => void = () => undefined;
    finalise().resolveWith(
      new Promise<undefined>((resolve) => {
        resolveFinalise = resolve;
      }),
    );
    renderForm();
    fillValid();
    submit();

    const button = await screen.findByRole('button', { name: 'Validating…' });
    expect(button).toBeDisabled();
    expect(screen.getByText('Validation')).toHaveAttribute('aria-current', 'step');

    act(() => {
      resolveFinalise(undefined);
    });

    await waitFor(() =>
      expect(screen.getByText('Recording validated and ready.')).toBeInTheDocument(),
    );
  });

  it('surfaces a rejected upload session with the server message', async () => {
    createSession().rejectWith(new Error('You are not a member of an organisation yet.'));
    renderForm();
    fillValid();
    submit();

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'You are not a member of an organisation yet.',
    );
    expect(uploadRecording).not.toHaveBeenCalled();
    expect(screen.getByRole('button', { name: 'Try again' })).toBeInTheDocument();
  });

  it('falls back to a default message when the session cannot start', async () => {
    createSession().rejectWith(new Error(''));
    renderForm();
    fillValid();
    submit();

    expect(await screen.findByRole('alert')).toHaveTextContent('The upload could not be started.');
  });

  it('surfaces an upload failure with the server message', async () => {
    createSession().resolveWith({ upload_session_id: 'sess-1' });
    uploadRecording.mockResolvedValue({
      ok: false,
      status: 500,
      message: 'The server could not store the recording.',
    });
    renderForm();
    fillValid();
    submit();

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'The server could not store the recording.',
    );
    expect(finalise().calls).toHaveLength(0);
  });

  it('falls back to a default message when the upload fails silently', async () => {
    createSession().resolveWith({ upload_session_id: 'sess-1' });
    uploadRecording.mockResolvedValue({ ok: false, status: 0, message: null });
    renderForm();
    fillValid();
    submit();

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'The recording could not be uploaded.',
    );
  });

  it('surfaces a validation failure with the server message', async () => {
    createSession().resolveWith({ upload_session_id: 'sess-1' });
    uploadRecording.mockResolvedValue({ ok: true, status: 200, message: null });
    finalise().rejectWith(new Error('The recording could not be validated.'));
    renderForm();
    fillValid();
    submit();

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'The recording could not be validated.',
    );
    expect(refresh).toHaveBeenCalled();
  });

  it('falls back to a default message when validation fails silently', async () => {
    createSession().resolveWith({ upload_session_id: 'sess-1' });
    uploadRecording.mockResolvedValue({ ok: true, status: 200, message: null });
    finalise().rejectWith(new Error(''));
    renderForm();
    fillValid();
    submit();

    expect(await screen.findByRole('alert')).toHaveTextContent('The recording failed validation.');
  });

  it('reports a recording that becomes invalid before submission', () => {
    renderForm();

    const file = npyFile();
    chooseFile(file);
    setFrequency('250');
    setChannels('Cz');
    Object.defineProperty(file, 'size', { value: 0 });
    submit();

    expect(screen.getByRole('alert')).toHaveTextContent('The selected file is empty.');
    expect(createSession().calls).toHaveLength(0);
  });

  it('reports a missing recording after the file is removed', () => {
    renderForm();

    chooseFile(npyFile());
    setFrequency('250');
    setChannels('Cz');
    fireEvent.click(screen.getByRole('button', { name: 'Remove' }));
    submit();

    expect(screen.getByRole('alert')).toHaveTextContent('Choose a recording to upload.');
    expect(createSession().calls).toHaveLength(0);
  });
});
