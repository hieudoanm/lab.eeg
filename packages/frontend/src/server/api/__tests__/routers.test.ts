import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('server-only', () => ({}));

const { cookiesGet, cookiesSet, cookiesDelete, revalidatePath } = vi.hoisted(() => ({
  cookiesGet: vi.fn(),
  cookiesSet: vi.fn(),
  cookiesDelete: vi.fn(),
  revalidatePath: vi.fn(),
}));

vi.mock('next/headers', () => ({
  cookies: async () => ({ get: cookiesGet, set: cookiesSet, delete: cookiesDelete }),
}));

vi.mock('next/cache', () => ({ revalidatePath }));

const api = vi.hoisted(() => ({
  fetchCurrentUser: vi.fn(),
  issueDevelopmentToken: vi.fn(),
  endSession: vi.fn(),
  fetchOrganisations: vi.fn(),
  fetchOrganisation: vi.fn(),
  fetchOrganisationMembers: vi.fn(),
  fetchStudies: vi.fn(),
  fetchStudy: vi.fn(),
  createStudy: vi.fn(),
  updateStudy: vi.fn(),
  archiveStudy: vi.fn(),
  fetchDatasets: vi.fn(),
  fetchDataset: vi.fn(),
  registerDataset: vi.fn(),
  createUploadSession: vi.fn(),
  finaliseUpload: vi.fn(),
  fetchAnalyses: vi.fn(),
  fetchOrganisationAnalyses: vi.fn(),
  fetchAnalysisJob: vi.fn(),
  fetchAnalysisResult: vi.fn(),
  submitAnalysis: vi.fn(),
  cancelAnalysis: vi.fn(),
  fetchSignalWindow: vi.fn(),
  fetchOrganisationSummary: vi.fn(),
}));

vi.mock('@/lib/api', () => api);
vi.mock('@/lib/api/identity', () => ({
  fetchCurrentUser: api.fetchCurrentUser,
  issueDevelopmentToken: api.issueDevelopmentToken,
  endSession: api.endSession,
}));

import type { ApiClient } from '@/lib/api/client';
import { ApiError } from '@/lib/api/errors';
import type { CurrentUser } from '@/lib/api/types';
import { appRouter } from '@/server/api/root';
import { createTRPCContext, guard, toTRPCError, type TRPCContext } from '@/server/api/trpc';

const orgId = 'org-1';
const membership = {
  membership_id: 'm1',
  organisation_id: orgId,
  organisation_name: 'Org',
  role: 'owner' as const,
};
const user: CurrentUser = {
  id: 'u1',
  display_name: 'Ada',
  email: 'ada@example.org',
  memberships: [membership],
};

const buildContext = (overrides: Partial<TRPCContext> = {}): TRPCContext => ({
  token: 'token-123',
  client: {} as ApiClient,
  organisationPreference: null,
  setSessionToken: vi.fn(async () => undefined),
  clearSessionToken: vi.fn(async () => undefined),
  setOrganisationPreference: vi.fn(async () => undefined),
  ...overrides,
});

const caller = (overrides: Partial<TRPCContext> = {}) =>
  appRouter.createCaller(buildContext(overrides));

beforeEach(() => {
  vi.clearAllMocks();
  api.fetchCurrentUser.mockResolvedValue(user);
});

describe('createTRPCContext', () => {
  it('reads the session cookie and writes cookies through the store', async () => {
    cookiesGet.mockImplementation((name: string) =>
      name === 'eeg-lab_session' ? { value: 'token-abc' } : { value: 'org-2' },
    );

    const context = await createTRPCContext();

    expect(context.token).toBe('token-abc');
    expect(context.organisationPreference).toBe('org-2');

    await context.setSessionToken('new-token', 3600);
    expect(cookiesSet).toHaveBeenCalledWith(
      'eeg-lab_session',
      'new-token',
      expect.objectContaining({ maxAge: 3600, httpOnly: true }),
    );

    await context.clearSessionToken();
    expect(cookiesDelete).toHaveBeenCalledWith('eeg-lab_session');

    await context.setOrganisationPreference('org-9');
    expect(cookiesSet).toHaveBeenCalledWith(
      'eeg-lab_organisation',
      'org-9',
      expect.objectContaining({ httpOnly: true }),
    );
  });
});

describe('toTRPCError and guard', () => {
  it.each([
    [400, 'BAD_REQUEST'],
    [401, 'UNAUTHORIZED'],
    [403, 'FORBIDDEN'],
    [404, 'NOT_FOUND'],
    [409, 'CONFLICT'],
    [413, 'PAYLOAD_TOO_LARGE'],
    [422, 'UNPROCESSABLE_CONTENT'],
    [500, 'INTERNAL_SERVER_ERROR'],
  ])('maps upstream status %i to %s', (status, code) => {
    const error = new ApiError({ status, code: 'x', message: 'boom' });

    expect(toTRPCError(error).code).toBe(code);
  });

  it('returns the operation result on success', async () => {
    await expect(guard(async () => 42)).resolves.toBe(42);
  });

  it('converts ApiError and rethrows other failures', async () => {
    await expect(
      guard(() => Promise.reject(new ApiError({ status: 404, code: 'x', message: 'gone' }))),
    ).rejects.toMatchObject({ code: 'NOT_FOUND' });

    const boom = new Error('boom');
    await expect(guard(async () => Promise.reject(boom))).rejects.toBe(boom);
  });
});

describe('authorisation', () => {
  it('rejects an anonymous caller', async () => {
    await expect(caller({ token: null }).identity.current()).rejects.toMatchObject({
      code: 'UNAUTHORIZED',
    });
    expect(api.fetchCurrentUser).not.toHaveBeenCalled();
  });

  it('treats a rejected session as unauthenticated', async () => {
    api.fetchCurrentUser.mockRejectedValue(
      new ApiError({ status: 401, code: 'unauthorized', message: 'nope' }),
    );

    await expect(caller().identity.current()).rejects.toMatchObject({ code: 'UNAUTHORIZED' });
  });

  it('propagates unexpected failures from user resolution', async () => {
    api.fetchCurrentUser.mockRejectedValue(
      new ApiError({ status: 500, code: 'server_error', message: 'boom' }),
    );

    await expect(caller().identity.current()).rejects.toMatchObject({
      code: 'INTERNAL_SERVER_ERROR',
    });
  });

  it('rejects organisation procedures when the user has no membership', async () => {
    api.fetchCurrentUser.mockResolvedValue({ ...user, memberships: [] });

    await expect(caller().studies.list({})).rejects.toMatchObject({ code: 'FORBIDDEN' });
  });
});

describe('identity router', () => {
  it('returns the session user and active membership', async () => {
    const result = await caller({ organisationPreference: orgId }).identity.session();

    expect(result.user).toEqual(user);
    expect(result.membership).toEqual(membership);
  });

  it('resolves the preference cookie for the active membership', async () => {
    const second = { ...membership, membership_id: 'm2', organisation_id: 'org-2' };
    api.fetchCurrentUser.mockResolvedValue({ ...user, memberships: [membership, second] });

    const result = await caller({ organisationPreference: 'org-2' }).identity.session();

    expect(result.membership?.organisation_id).toBe('org-2');
  });

  it('fetches organisations and organisation details', async () => {
    api.fetchOrganisations.mockResolvedValue({ items: [], next_cursor: null, has_more: false });
    api.fetchOrganisation.mockResolvedValue({ id: orgId, name: 'Org' });

    await caller().identity.organisations();
    expect(api.fetchOrganisations).toHaveBeenCalled();

    await caller().identity.organisation();
    expect(api.fetchOrganisation).toHaveBeenCalledWith(orgId, expect.anything());
  });

  it('fetches members', async () => {
    api.fetchOrganisationMembers.mockResolvedValue({
      items: [],
      next_cursor: null,
      has_more: false,
    });

    await caller().identity.members();

    expect(api.fetchOrganisationMembers).toHaveBeenCalledWith(orgId, expect.anything());
  });

  it('signs in by storing the session cookie', async () => {
    api.issueDevelopmentToken.mockResolvedValue({
      access_token: 'token-new',
      token_type: 'Bearer',
      expires_at: new Date(Date.now() + 3_600_000).toISOString(),
    });
    const setSessionToken = vi.fn(async () => undefined);

    const result = await caller({ setSessionToken }).identity.signIn({
      subject: 'ada@example.org',
      email: 'ada@example.org',
    });

    expect(api.issueDevelopmentToken).toHaveBeenCalledWith(
      { subject: 'ada@example.org', email: 'ada@example.org', display_name: undefined },
      expect.anything(),
    );
    expect(setSessionToken).toHaveBeenCalledWith('token-new', expect.any(Number));
    expect(revalidatePath).toHaveBeenCalledWith('/app', 'layout');
    expect(result.expiresAt).toBeTypeOf('string');
  });

  it('falls back to an hour when the token expiry is unparseable', async () => {
    api.issueDevelopmentToken.mockResolvedValue({
      access_token: 'token-new',
      token_type: 'Bearer',
      expires_at: 'not-a-date',
    });
    const setSessionToken = vi.fn(async () => undefined);

    await caller({ setSessionToken }).identity.signIn({ subject: 'ada@example.org' });

    expect(setSessionToken).toHaveBeenCalledWith('token-new', 3600);
  });

  it('signs out even when the upstream logout fails', async () => {
    api.endSession.mockRejectedValue(new ApiError({ status: 500, code: 'x', message: 'boom' }));
    const clearSessionToken = vi.fn(async () => undefined);

    await caller({ clearSessionToken }).identity.signOut();

    expect(clearSessionToken).toHaveBeenCalled();
    expect(revalidatePath).toHaveBeenCalledWith('/app', 'layout');
  });

  it('stores the organisation preference', async () => {
    const setOrganisationPreference = vi.fn(async () => undefined);

    await caller({ setOrganisationPreference }).identity.selectOrganisation({
      organisationId: 'org-2',
    });

    expect(setOrganisationPreference).toHaveBeenCalledWith('org-2');
  });
});

describe('studies router', () => {
  it('lists and reads studies scoped to the active organisation', async () => {
    api.fetchStudies.mockResolvedValue({ items: [], next_cursor: null, has_more: false });
    api.fetchStudy.mockResolvedValue({ id: 'study-1' });

    await caller().studies.list({ status: 'active' });
    expect(api.fetchStudies).toHaveBeenCalledWith(
      orgId,
      { search: undefined, status: 'active', cursor: undefined },
      expect.anything(),
    );

    await caller().studies.byId({ studyId: 'study-1' });
    expect(api.fetchStudy).toHaveBeenCalledWith(orgId, 'study-1', expect.anything());
  });

  it('creates, updates and archives studies and revalidates', async () => {
    api.createStudy.mockResolvedValue({ id: 'study-1' });
    api.updateStudy.mockResolvedValue({ id: 'study-1' });
    api.archiveStudy.mockResolvedValue({ id: 'study-1' });

    await caller().studies.create({ name: 'Alpha', description: null });
    expect(api.createStudy).toHaveBeenCalledWith(
      orgId,
      { name: 'Alpha', description: null },
      expect.anything(),
    );

    await caller().studies.update({ studyId: 'study-1', name: 'Beta', description: 'd' });
    expect(api.updateStudy).toHaveBeenCalledWith(
      orgId,
      'study-1',
      { name: 'Beta', description: 'd' },
      expect.anything(),
    );

    await caller().studies.archive({ studyId: 'study-1' });
    expect(api.archiveStudy).toHaveBeenCalledWith(orgId, 'study-1', expect.anything());

    expect(revalidatePath).toHaveBeenCalledWith('/app/studies');
  });
});

describe('datasets router', () => {
  it('lists, reads and registers datasets', async () => {
    api.fetchDatasets.mockResolvedValue({ items: [], next_cursor: null, has_more: false });
    api.fetchDataset.mockResolvedValue({ id: 'dataset-1' });
    api.registerDataset.mockResolvedValue({ id: 'dataset-1' });

    await caller().datasets.list({ studyId: 'study-1' });
    expect(api.fetchDatasets).toHaveBeenCalledWith(
      orgId,
      'study-1',
      { cursor: undefined, limit: undefined },
      expect.anything(),
    );

    await caller().datasets.byId({ datasetId: 'dataset-1' });
    expect(api.fetchDataset).toHaveBeenCalledWith(orgId, 'dataset-1', expect.anything());

    await caller().datasets.register({ studyId: 'study-1', name: 'Session' });
    expect(api.registerDataset).toHaveBeenCalledWith(
      orgId,
      'study-1',
      { name: 'Session', description: null, source_description: null },
      expect.anything(),
    );
    expect(revalidatePath).toHaveBeenCalledWith('/app/studies/study-1');
  });
});

describe('uploads router', () => {
  it('creates an upload session', async () => {
    api.createUploadSession.mockResolvedValue({ upload_session_id: 'sess-1' });

    await caller().uploads.createSession({
      datasetId: 'dataset-1',
      fileName: 'recording.npy',
      fileSizeBytes: 4,
      samplingFrequencyHz: 250,
      channelNames: ['Cz', 'Pz'],
    });

    expect(api.createUploadSession).toHaveBeenCalledWith(
      orgId,
      'dataset-1',
      {
        file_name: 'recording.npy',
        file_size_bytes: 4,
        manifest: {
          sampling_frequency_hz: 250,
          channel_names: ['Cz', 'Pz'],
          channel_units: null,
        },
      },
      expect.anything(),
    );
  });

  it('finalises an upload and revalidates the dataset', async () => {
    api.finaliseUpload.mockResolvedValue({ status: 'ready' });

    await caller().uploads.finalise({
      studyId: 'study-1',
      datasetId: 'dataset-1',
      sessionId: 'sess-1',
    });

    expect(api.finaliseUpload).toHaveBeenCalledWith(
      orgId,
      'dataset-1',
      'sess-1',
      expect.anything(),
    );
    expect(revalidatePath).toHaveBeenCalledWith('/app/studies/study-1/datasets/dataset-1');
  });

  it('revalidates even when finalisation fails', async () => {
    api.finaliseUpload.mockRejectedValue(new ApiError({ status: 422, code: 'x', message: 'bad' }));

    await expect(
      caller().uploads.finalise({
        studyId: 'study-1',
        datasetId: 'dataset-1',
        sessionId: 'sess-1',
      }),
    ).rejects.toMatchObject({ code: 'UNPROCESSABLE_CONTENT' });
    expect(revalidatePath).toHaveBeenCalledWith('/app/studies/study-1/datasets/dataset-1');
  });
});

describe('analyses router', () => {
  it('lists, reads jobs and results', async () => {
    api.fetchAnalyses.mockResolvedValue({ items: [], next_cursor: null, has_more: false });
    api.fetchAnalysisJob.mockResolvedValue({ id: 'job-1' });
    api.fetchAnalysisResult.mockResolvedValue({ analysis_id: 'job-1' });

    await caller().analyses.list({ datasetId: 'dataset-1' });
    expect(api.fetchAnalyses).toHaveBeenCalledWith(
      orgId,
      'dataset-1',
      { cursor: undefined, limit: undefined },
      expect.anything(),
    );

    await caller().analyses.byId({ jobId: 'job-1' });
    expect(api.fetchAnalysisJob).toHaveBeenCalledWith(orgId, 'job-1', expect.anything());

    await caller().analyses.result({ jobId: 'job-1' });
    expect(api.fetchAnalysisResult).toHaveBeenCalledWith(orgId, 'job-1', expect.anything());
  });

  it('submits an analysis with an idempotency key', async () => {
    api.submitAnalysis.mockResolvedValue({ id: 'job-1' });

    await caller().analyses.submit({
      datasetId: 'dataset-1',
      algorithmVersion: '1.0.0',
      parameters: { include_mean: true },
      idempotencyKey: 'key-1',
    });

    expect(api.submitAnalysis).toHaveBeenCalledWith(
      orgId,
      'dataset-1',
      {
        analysis_type: 'descriptive_statistics',
        algorithm_version: '1.0.0',
        parameters: { include_mean: true },
      },
      'key-1',
      expect.anything(),
    );
  });

  it('cancels an analysis', async () => {
    api.cancelAnalysis.mockResolvedValue({ id: 'job-1' });

    await caller().analyses.cancel({ jobId: 'job-1' });

    expect(api.cancelAnalysis).toHaveBeenCalledWith(orgId, 'job-1', expect.anything());
  });

  it('lists organisation analyses with filters', async () => {
    api.fetchOrganisationAnalyses.mockResolvedValue({
      items: [],
      next_cursor: null,
      has_more: false,
    });

    await caller().analyses.listOrganisation({
      status: 'failed',
      datasetId: 'dataset-1',
      studyId: 'study-1',
      cursor: 'c1',
      limit: 25,
    });

    expect(api.fetchOrganisationAnalyses).toHaveBeenCalledWith(
      orgId,
      {
        cursor: 'c1',
        limit: 25,
        datasetId: 'dataset-1',
        studyId: 'study-1',
        status: 'failed',
      },
      expect.anything(),
    );
  });
});

describe('signals router', () => {
  it('fetches a signal window scoped to the active organisation', async () => {
    api.fetchSignalWindow.mockResolvedValue({ channels: [], samples: [] });

    await caller().signals.window({
      datasetId: 'dataset-1',
      startSeconds: 0,
      endSeconds: 5,
      channels: [0, 2],
    });

    expect(api.fetchSignalWindow).toHaveBeenCalledWith(
      orgId,
      'dataset-1',
      { startSeconds: 0, endSeconds: 5, channels: [0, 2] },
      expect.anything(),
    );
  });
});

describe('summary router', () => {
  it('fetches the organisation dashboard summary', async () => {
    api.fetchOrganisationSummary.mockResolvedValue({ study_count: 0 });

    await caller().summary.organisation();

    expect(api.fetchOrganisationSummary).toHaveBeenCalledWith(orgId, expect.anything());
  });
});
