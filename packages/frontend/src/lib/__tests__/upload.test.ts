import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import {
  ACCEPTED_RECORDING_ATTRIBUTE,
  ACCEPTED_RECORDING_EXTENSIONS,
  MAX_UPLOAD_BYTES,
  parseChannelList,
  uploadRecording,
  validateRecordingFile,
} from '@/lib/upload';

const makeFile = (name: string, size: number): File => {
  return new File([new Uint8Array(Math.max(size, 1))], name);
};

type ProgressHandler = (event: ProgressEvent) => void;

class FakeXhr {
  static instances: FakeXhr[] = [];

  status = 0;
  responseText = '';
  readonly open = vi.fn();
  readonly setRequestHeader = vi.fn();
  readonly send = vi.fn();
  readonly abort = vi.fn(() => {
    this.onabort?.();
  });
  readonly upload: { onprogress: ProgressHandler | null } = { onprogress: null };
  onload: (() => void) | null = null;
  onerror: (() => void) | null = null;
  onabort: (() => void) | null = null;

  constructor() {
    FakeXhr.instances.push(this);
  }
}

const latestXhr = (): FakeXhr => {
  const instance = FakeXhr.instances[FakeXhr.instances.length - 1];
  if (!instance) {
    throw new Error('no XMLHttpRequest was created');
  }
  return instance;
};

const progress = (loaded: number, total: number, lengthComputable = true): ProgressEvent =>
  ({ lengthComputable, loaded, total }) as ProgressEvent;

describe('validateRecordingFile', () => {
  it('accepts an .npy recording within the limit', () => {
    expect(validateRecordingFile(makeFile('recording.npy', 1024))).toBeNull();
  });

  it('accepts an uppercase .NPY extension', () => {
    expect(validateRecordingFile(makeFile('recording.NPY', 1024))).toBeNull();
  });

  it('rejects unsupported formats', () => {
    expect(validateRecordingFile(makeFile('recording.csv', 1024))).toMatch(/supported/i);
  });

  it('rejects a file name without an extension', () => {
    expect(validateRecordingFile(makeFile('recording', 1024))).toMatch(/supported/i);
  });

  it('rejects an empty file', () => {
    expect(validateRecordingFile(new File([], 'empty.npy'))).toMatch(/empty/i);
  });

  it('rejects recordings over the size limit', () => {
    expect(validateRecordingFile(makeFile('recording.npy', 2048), 1024)).toMatch(/maximum/i);
  });

  it('exposes the upload constants', () => {
    expect(MAX_UPLOAD_BYTES).toBe(268_435_456);
    expect(ACCEPTED_RECORDING_EXTENSIONS).toEqual(['.npy']);
    expect(ACCEPTED_RECORDING_ATTRIBUTE).toBe('.npy');
  });

  it('honours a configured maximum upload size', async () => {
    vi.stubEnv('NEXT_PUBLIC_MAX_UPLOAD_BYTES', '1048576');
    vi.resetModules();

    const mod = await import('@/lib/upload');
    expect(mod.MAX_UPLOAD_BYTES).toBe(1_048_576);

    vi.unstubAllEnvs();
    vi.resetModules();
  });
});

describe('parseChannelList', () => {
  it('splits on newlines and commas and trims blanks', () => {
    expect(parseChannelList('Fz\nCz, Pz\n\n  Oz  ')).toEqual(['Fz', 'Cz', 'Pz', 'Oz']);
  });

  it('returns an empty list for blank input', () => {
    expect(parseChannelList('   \n ')).toEqual([]);
  });
});

describe('uploadRecording', () => {
  beforeEach(() => {
    FakeXhr.instances = [];
    vi.stubGlobal('XMLHttpRequest', FakeXhr);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('opens a PUT request with the octet-stream header and sends the file', async () => {
    const file = makeFile('recording.npy', 8);
    const promise = uploadRecording('http://api.test/upload', file);
    const xhr = latestXhr();

    expect(xhr.open).toHaveBeenCalledWith('PUT', 'http://api.test/upload');
    expect(xhr.setRequestHeader).toHaveBeenCalledWith('Content-Type', 'application/octet-stream');
    expect(xhr.send).toHaveBeenCalledWith(file);
    expect(xhr.upload.onprogress).toBeNull();

    xhr.status = 200;
    xhr.onload?.();

    await expect(promise).resolves.toEqual({ ok: true, status: 200, message: null });
  });

  it('treats any 2xx status as success', async () => {
    const promise = uploadRecording('http://api.test/upload', makeFile('r.npy', 4));
    const xhr = latestXhr();
    xhr.status = 201;
    xhr.onload?.();

    await expect(promise).resolves.toEqual({ ok: true, status: 201, message: null });
  });

  it('surfaces the JSON error message on a failed upload', async () => {
    const promise = uploadRecording('http://api.test/upload', makeFile('r.npy', 4));
    const xhr = latestXhr();
    xhr.status = 422;
    xhr.responseText = JSON.stringify({ error: { message: 'Rejected by the server.' } });
    xhr.onload?.();

    await expect(promise).resolves.toEqual({
      ok: false,
      status: 422,
      message: 'Rejected by the server.',
    });
  });

  it('returns no message for a non-JSON error body', async () => {
    const promise = uploadRecording('http://api.test/upload', makeFile('r.npy', 4));
    const xhr = latestXhr();
    xhr.status = 500;
    xhr.responseText = '<html>boom</html>';
    xhr.onload?.();

    await expect(promise).resolves.toEqual({ ok: false, status: 500, message: null });
  });

  it('ignores JSON bodies without a usable error message', async () => {
    const bodies = [
      '',
      '42',
      JSON.stringify({ error: {} }),
      JSON.stringify({ error: { message: 5 } }),
    ];
    for (const body of bodies) {
      FakeXhr.instances = [];
      const promise = uploadRecording('http://api.test/upload', makeFile('r.npy', 4));
      const xhr = latestXhr();
      xhr.status = 400;
      xhr.responseText = body;
      xhr.onload?.();
      await expect(promise).resolves.toEqual({ ok: false, status: 400, message: null });
    }
  });

  it('resolves an unreachable-server result on error', async () => {
    const promise = uploadRecording('http://api.test/upload', makeFile('r.npy', 4));
    const xhr = latestXhr();
    xhr.onerror?.();

    await expect(promise).resolves.toEqual({
      ok: false,
      status: 0,
      message: 'The server could not be reached.',
    });
  });

  it('resolves a cancellation result on abort', async () => {
    const promise = uploadRecording('http://api.test/upload', makeFile('r.npy', 4));
    const xhr = latestXhr();
    xhr.onabort?.();

    await expect(promise).resolves.toEqual({
      ok: false,
      status: 0,
      message: 'The upload was cancelled.',
    });
  });

  it('reports progress as an integer percentage', async () => {
    const onProgress = vi.fn();
    const promise = uploadRecording('http://api.test/upload', makeFile('r.npy', 4), { onProgress });
    const xhr = latestXhr();

    xhr.upload.onprogress?.(progress(50, 100));
    xhr.upload.onprogress?.(progress(1, 3));
    xhr.upload.onprogress?.(progress(200, 100));

    expect(onProgress).toHaveBeenNthCalledWith(1, 50);
    expect(onProgress).toHaveBeenNthCalledWith(2, 33);
    expect(onProgress).toHaveBeenNthCalledWith(3, 100);

    xhr.status = 204;
    xhr.onload?.();
    await promise;
  });

  it('ignores progress events without a computable total', async () => {
    const onProgress = vi.fn();
    const promise = uploadRecording('http://api.test/upload', makeFile('r.npy', 4), { onProgress });
    const xhr = latestXhr();

    xhr.upload.onprogress?.(progress(50, 0, false));
    xhr.upload.onprogress?.(progress(50, 0, true));

    expect(onProgress).not.toHaveBeenCalled();

    xhr.status = 204;
    xhr.onload?.();
    await promise;
  });

  it('aborts immediately when the signal is already aborted', async () => {
    const controller = new AbortController();
    controller.abort();
    const promise = uploadRecording('http://api.test/upload', makeFile('r.npy', 4), {
      signal: controller.signal,
    });
    const xhr = latestXhr();

    expect(xhr.abort).toHaveBeenCalledTimes(1);
    expect(xhr.send).not.toHaveBeenCalled();

    await expect(promise).resolves.toEqual({
      ok: false,
      status: 0,
      message: 'The upload was cancelled.',
    });
  });

  it('aborts mid-flight when the signal fires', async () => {
    const controller = new AbortController();
    const promise = uploadRecording('http://api.test/upload', makeFile('r.npy', 4), {
      signal: controller.signal,
    });
    const xhr = latestXhr();
    expect(xhr.send).toHaveBeenCalledTimes(1);

    controller.abort();

    expect(xhr.abort).toHaveBeenCalledTimes(1);
    await expect(promise).resolves.toEqual({
      ok: false,
      status: 0,
      message: 'The upload was cancelled.',
    });
  });
});
