import { describe, expect, it, vi } from 'vitest';

import { ApiClient } from '@/lib/api/client';
import { createUploadSession, finaliseUpload } from '@/lib/api/uploads';

const jsonResponse = (body: unknown, status = 200): Response => {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  });
};

const makeClient = (fetchImpl: typeof fetch) => {
  return new ApiClient({ baseUrl: 'http://api.test/api/v1', fetchImpl, getToken: () => null });
};

const lastCall = (fetchMock: typeof fetch) => {
  const calls = (fetchMock as unknown as ReturnType<typeof vi.fn>).mock.calls;
  return calls[calls.length - 1] as [string, RequestInit];
};

const MANIFEST = {
  sampling_frequency_hz: 256,
  channel_names: ['Fz', 'Cz'],
  channel_units: ['uV', 'uV'],
};

describe('uploads API', () => {
  it('creates an upload session with the recording manifest', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ upload_session_id: 'u1' }, 201)) as unknown as typeof fetch;

    await createUploadSession(
      'org-1',
      'd1',
      { file_name: 'recording.npy', file_size_bytes: 2048, manifest: MANIFEST },
      makeClient(fetchMock),
    );

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe('http://api.test/api/v1/organisations/org-1/datasets/d1/upload-sessions');
    expect(init.method).toBe('POST');
    expect(init.body).toBe(
      JSON.stringify({ file_name: 'recording.npy', file_size_bytes: 2048, manifest: MANIFEST }),
    );
  });

  it('finalises an upload session', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(
        jsonResponse({ dataset: { id: 'd1', status: 'ready' } }),
      ) as unknown as typeof fetch;

    await finaliseUpload('org-1', 'd1', 'u1', makeClient(fetchMock));

    const [url, init] = lastCall(fetchMock);
    expect(url).toBe(
      'http://api.test/api/v1/organisations/org-1/datasets/d1/upload-sessions/u1/finalise',
    );
    expect(init.method).toBe('POST');
    expect(init.body).toBeUndefined();
  });
});
