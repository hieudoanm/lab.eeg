import { expect, test } from '@playwright/test';

const apiBaseUrl = process.env.NEUROLAB_E2E_API_BASE;

test.describe.configure({ retries: 1 });

test.describe('full analysis workflow', () => {
  test.skip(apiBaseUrl === undefined, 'Set NEUROLAB_E2E_API_BASE to run against a live API');

  let token: string;
  let organisationId: string;
  let studyId: string;
  let datasetId: string;
  let jobId: string;

  test.beforeAll(async ({ request }) => {
    const response = await request.post(`${apiBaseUrl}/auth/token`, {
      data: {
        subject: 'e2e-analysis@example.org',
        display_name: 'E2E Analysis Researcher',
        email: 'e2e-analysis@eeg-lab.local',
      },
    });
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    token = data.access_token;
  });

  test('creates an organisation context', async ({ request }) => {
    const response = await request.get(`${apiBaseUrl}/organisations`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.length).toBeGreaterThan(0);
    organisationId = data[0].id;
  });

  test('creates a study', async ({ request }) => {
    const response = await request.post(`${apiBaseUrl}/organisations/${organisationId}/studies`, {
      headers: { Authorization: `Bearer ${token}` },
      data: { name: 'E2E Analysis Study', description: 'Study for end-to-end analysis test' },
    });
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.name).toBe('E2E Analysis Study');
    studyId = data.id;
  });

  test('registers a dataset', async ({ request }) => {
    const response = await request.post(
      `${apiBaseUrl}/organisations/${organisationId}/studies/${studyId}/datasets`,
      {
        headers: { Authorization: `Bearer ${token}` },
        data: {
          name: 'E2E Recording',
          description: 'Synthetic recording for analysis test',
          format: 'npy',
          source_description: 'Generated for E2E test',
        },
      },
    );
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.status).toBe('pending_upload');
    datasetId = data.id;
  });

  test('creates an upload session and uploads a synthetic recording', async ({ request }) => {
    // Create upload session
    const sessionResponse = await request.post(
      `${apiBaseUrl}/organisations/${organisationId}/datasets/${datasetId}/upload-sessions`,
      {
        headers: { Authorization: `Bearer ${token}` },
        data: {
          file_name: 'recording.npy',
          file_size_bytes: 1024,
          manifest: {
            sampling_frequency_hz: 256.0,
            channel_names: ['F3', 'F4'],
            channel_units: ['uV', 'uV'],
          },
        },
      },
    );
    expect(sessionResponse.ok()).toBeTruthy();
    const sessionData = await sessionResponse.json();
    const sessionId = sessionData.upload_session_id;

    // Generate synthetic NPY data (2 channels, 100 samples, float32)
    const channels = 2;
    const samples = 100;
    const data = new Float32Array(channels * samples);
    for (let c = 0; c < channels; c++) {
      for (let s = 0; s < samples; s++) {
        data[c * samples + s] =
          Math.sin(((2 * Math.PI * s) / 256) * (c + 1) * 10) + 0.1 * Math.random();
      }
    }
    const npyHeader = Buffer.alloc(128);
    npyHeader.writeUInt8(0x93, 0); // Magic byte
    npyHeader.writeUInt8(0x01, 1); // Major version
    npyHeader.writeUInt8(0x00, 2); // Minor version
    npyHeader.writeUInt8(0x00, 3); // Header length placeholder
    // Write actual header after calculating length
    const headerDict = {
      descr: '<f4',
      fortran_order: false,
      shape: [channels, samples],
    };
    const headerStr = JSON.stringify(headerDict).replace(/"/g, "'").replace(/, /g, ',');
    const headerBytes = Buffer.from(headerStr + ' ', 'ascii');
    const headerLen = 10 + headerBytes.length;
    const paddedLen = ((headerLen + 63) / 64) * 64;
    const padding = Buffer.alloc(paddedLen - headerLen);
    const fullHeader = Buffer.concat([
      Buffer.from([0x93, 0x4e, 0x55, 0x4d, 0x50, 0x59, 0x01, 0x00]),
      (() => {
        const buf = Buffer.alloc(2);
        buf.writeUInt16LE(paddedLen);
        return buf;
      })(),
      headerBytes,
      padding,
    ]);
    const npyData = Buffer.concat([fullHeader, Buffer.from(data.buffer)]);

    // Upload content
    const uploadResponse = await request.put(
      `${apiBaseUrl}/organisations/${organisationId}/datasets/${datasetId}/upload-sessions/${sessionId}/content`,
      {
        headers: {
          Authorization: `Bearer ${token}`,
          'Content-Type': 'application/octet-stream',
        },
        data: npyData,
      },
    );
    expect(uploadResponse.status()).toBe(204);

    // Finalise
    const finaliseResponse = await request.post(
      `${apiBaseUrl}/organisations/${organisationId}/datasets/${datasetId}/upload-sessions/${sessionId}/finalise`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(finaliseResponse.ok()).toBeTruthy();
    const finaliseData = await finaliseResponse.json();
    expect(finaliseData.dataset.status).toBe('ready');
  });

  test('submits a descriptive statistics analysis', async ({ request }) => {
    const idempotencyKey = `e2e-analysis-${Date.now()}`;
    const response = await request.post(
      `${apiBaseUrl}/organisations/${organisationId}/datasets/${datasetId}/analyses`,
      {
        headers: {
          Authorization: `Bearer ${token}`,
          'Idempotency-Key': idempotencyKey,
        },
        data: {
          analysis_type: 'descriptive_statistics',
          algorithm_version: '1.0.0',
          parameters: {
            include_mean: true,
            include_standard_deviation: true,
            include_min_max: true,
            include_peak_to_peak: true,
          },
        },
      },
    );
    expect(response.status()).toBe(202);
    const data = await response.json();
    expect(data.status).toBe('queued');
    expect(data.analysis_type).toBe('descriptive_statistics');
    jobId = data.id;
  });

  test('polls for job completion', async ({ request }) => {
    let status = 'queued';
    const maxAttempts = 30;
    for (let attempt = 0; attempt < maxAttempts; attempt++) {
      const response = await request.get(
        `${apiBaseUrl}/organisations/${organisationId}/analyses/${jobId}`,
        {
          headers: { Authorization: `Bearer ${token}` },
        },
      );
      expect(response.ok()).toBeTruthy();
      const data = await response.json();
      status = data.status;
      if (status === 'succeeded' || status === 'failed') break;
      await new Promise((resolve) => setTimeout(resolve, 2000));
    }
    expect(status).toBe('succeeded');
  });

  test('retrieves analysis results', async ({ request }) => {
    const response = await request.get(
      `${apiBaseUrl}/organisations/${organisationId}/analyses/${jobId}/results`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data.analysis_id).toBe(jobId);
    expect(data.channels.length).toBe(2);
    expect(data.channels[0].channel_name).toBe('F3');
    expect(data.channels[1].channel_name).toBe('F4');
    expect(data.channels[0].valid_sample_count).toBeGreaterThan(0);
    expect(data.provenance.input_checksum_sha256).toBeTruthy();
    expect(data.provenance.parameters.include_mean).toBe(true);
  });

  test('verifies idempotent submission returns same job', async ({ request }) => {
    const idempotencyKey = `e2e-analysis-${Date.now()}`;
    const first = await request.post(
      `${apiBaseUrl}/organisations/${organisationId}/datasets/${datasetId}/analyses`,
      {
        headers: { Authorization: `Bearer ${token}`, 'Idempotency-Key': idempotencyKey },
        data: { analysis_type: 'descriptive_statistics', algorithm_version: '1.0.0' },
      },
    );
    expect(first.status()).toBe(202);
    const firstData = await first.json();

    const second = await request.post(
      `${apiBaseUrl}/organisations/${organisationId}/datasets/${datasetId}/analyses`,
      {
        headers: { Authorization: `Bearer ${token}`, 'Idempotency-Key': idempotencyKey },
        data: { analysis_type: 'descriptive_statistics', algorithm_version: '1.0.0' },
      },
    );
    expect(second.status()).toBe(202);
    const secondData = await second.json();

    expect(secondData.id).toBe(firstData.id);
  });

  test('cancels a queued analysis', async ({ request }) => {
    // Submit a new analysis to cancel
    const idempotencyKey = `e2e-cancel-${Date.now()}`;
    const submit = await request.post(
      `${apiBaseUrl}/organisations/${organisationId}/datasets/${datasetId}/analyses`,
      {
        headers: { Authorization: `Bearer ${token}`, 'Idempotency-Key': idempotencyKey },
        data: { analysis_type: 'descriptive_statistics', algorithm_version: '1.0.0' },
      },
    );
    expect(submit.status()).toBe(202);
    const submitData = await submit.json();
    const cancelJobId = submitData.id;

    const cancelResponse = await request.post(
      `${apiBaseUrl}/organisations/${organisationId}/analyses/${cancelJobId}/cancel`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(cancelResponse.ok()).toBeTruthy();
    const cancelData = await cancelResponse.json();
    expect(cancelData.status).toBe('cancelled');

    // Second cancel should be idempotent
    const secondCancel = await request.post(
      `${apiBaseUrl}/organisations/${organisationId}/analyses/${cancelJobId}/cancel`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(secondCancel.ok()).toBeTruthy();
    const secondCancelData = await secondCancel.json();
    expect(secondCancelData.status).toBe('cancelled');
  });

  test('enforces tenant isolation', async ({ request }) => {
    // Create a second organisation and user
    const tokenResponse = await request.post(`${apiBaseUrl}/auth/token`, {
      data: { subject: 'e2e-other@example.org', display_name: 'Other Researcher' },
    });
    expect(tokenResponse.ok()).toBeTruthy();
    const { access_token: otherToken } = await tokenResponse.json();

    // Get other user's organisation
    const orgsResponse = await request.get(`${apiBaseUrl}/organisations`, {
      headers: { Authorization: `Bearer ${otherToken}` },
    });
    const otherOrgs = await orgsResponse.json();
    const otherOrgId = otherOrgs[0].id;

    // Try to access first user's analysis from second organisation
    const isolationResponse = await request.get(
      `${apiBaseUrl}/organisations/${otherOrgId}/analyses/${jobId}`,
      { headers: { Authorization: `Bearer ${otherToken}` } },
    );
    expect(isolationResponse.status()).toBe(404);
  });
});
