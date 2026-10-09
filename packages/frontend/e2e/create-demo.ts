import { chromium } from '@playwright/test';
import { createLogger } from '../src/utils/log';

const log = createLogger('create-demo');

const baseUrl = process.env.DEMO_WEB_URL ?? 'http://localhost:3000';
const apiBaseUrl = process.env.DEMO_API_URL ?? 'http://localhost:8000/api/v1';
const recordingPath = process.env.DEMO_RECORDING!;
const videoDirectory = process.env.DEMO_VIDEO_DIRECTORY!;

const sleep = (milliseconds: number) => new Promise((resolve) => setTimeout(resolve, milliseconds));

interface ApiResponse<T = unknown> {
  ok: boolean;
  status: number;
  json(): Promise<T>;
  text(): Promise<string>;
}

interface OrganisationItem {
  id: string;
}

interface OrganisationsResponse {
  items: OrganisationItem[];
}

async function readJson<T = unknown>(response: ApiResponse<T>, action: string): Promise<T> {
  if (!response.ok) {
    throw new Error(`${action} failed (${response.status}): ${await response.text()}`);
  }
  return response.json();
}

async function main() {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    viewport: { width: 1440, height: 900 },
    recordVideo: {
      dir: videoDirectory,
      size: { width: 1440, height: 900 },
    },
  });
  const page = await context.newPage();
  page.setDefaultTimeout(30_000);
  page.setDefaultNavigationTimeout(30_000);

  try {
    log.info('Recording sign-in and workspace tour');
    await page.goto(`${baseUrl}/login`, { waitUntil: 'networkidle' });
    await sleep(1200);
    await page.getByLabel('Email address').fill('demo-researcher@eeg-lab.local');
    await sleep(500);
    await page.getByRole('button', { name: 'Sign in' }).click();
    await page.waitForURL('**/app/dashboard');
    await page.getByRole('heading', { name: 'Dashboard' }).waitFor();
    await sleep(1800);

    await page
      .getByRole('navigation', { name: 'Primary' })
      .getByRole('link', { name: 'Studies' })
      .click();
    await page.getByRole('heading', { name: 'Studies', exact: true }).first().waitFor();
    const demoStamp = new Date()
      .toISOString()
      .replace(/[-:.TZ]/g, '')
      .slice(0, 14);
    const studyName = `EEG Lab Demo Study ${demoStamp}`;
    await page.getByRole('button', { name: 'Create study' }).waitFor();
    const studyForm = page.locator('form').filter({
      has: page.getByRole('button', { name: 'Create study' }),
    });
    await studyForm.locator('input[name="name"]').fill(studyName);
    await studyForm
      .locator('input[name="description"]')
      .fill('Synthetic EEG recording used to demonstrate the EEG Lab research workflow.');
    await sleep(600);
    await page.getByRole('button', { name: 'Create study' }).click();
    await page.waitForURL(/\/app\/studies\/[0-9a-f-]+$/i);
    await page.getByRole('heading', { name: studyName }).waitFor();
    await sleep(1600);

    log.info('Registering and validating the synthetic dataset');
    await page.getByRole('button', { name: 'Register dataset' }).waitFor();
    const datasetForm = page.locator('form').filter({
      has: page.getByRole('button', { name: 'Register dataset' }),
    });
    await datasetForm.locator('input[name="name"]').fill('Resting-state EEG recording');
    await datasetForm
      .locator('textarea[name="description"]')
      .fill('Four-channel deterministic synthetic recording.');
    await datasetForm
      .locator('textarea[name="source_description"]')
      .fill('Synthetic data generated for this demonstration.');
    await sleep(500);
    await datasetForm.getByRole('button', { name: 'Register dataset' }).click();
    await page.waitForURL(/\/app\/studies\/[0-9a-f-]+\/datasets\/[0-9a-f-]+$/i);
    await page.getByRole('heading', { name: 'Resting-state EEG recording' }).waitFor();
    await sleep(1000);

    const uploadForm = page.locator('form').filter({
      has: page.getByRole('button', { name: 'Upload recording' }),
    });
    await uploadForm.locator('input[type="file"]').setInputFiles(recordingPath);
    await uploadForm.locator('input[name="sampling_frequency_hz"]').fill('256');
    await uploadForm.locator('textarea[name="channel_names"]').fill('Fp1\nFp2\nF3\nF4');
    await uploadForm.locator('textarea[name="channel_units"]').fill('uV\nuV\nuV\nuV');
    await sleep(800);
    await page.getByRole('button', { name: 'Upload recording' }).click();
    await page.getByText('Ready', { exact: true }).first().waitFor({ timeout: 90_000 });
    await sleep(1800);

    const cookies = await context.cookies(baseUrl);
    const session = cookies.find((cookie) => cookie.name === 'eeg-lab_session');
    if (!session) {
      throw new Error('The sign-in flow did not create a EEG Lab session cookie.');
    }

    const authHeaders = {
      Authorization: `Bearer ${session.value}`,
      'Content-Type': 'application/json',
    };
    const organisationsResponse = await fetch(`${apiBaseUrl}/organisations`, {
      headers: authHeaders,
    });
    const organisations = await readJson<OrganisationsResponse>(
      organisationsResponse,
      'Loading organisations',
    );
    if (!Array.isArray(organisations.items) || organisations.items.length === 0) {
      throw new Error('The signed-in demo user has no organisation.');
    }

    const studyId = page.url().match(/\/app\/studies\/([0-9a-f-]+)\/datasets\//i)?.[1];
    const datasetId = page.url().match(/\/datasets\/([0-9a-f-]+)$/i)?.[1];
    if (!studyId || !datasetId) {
      throw new Error('Could not identify the newly created study and dataset.');
    }

    // Navigate back to study and dataset
    await page
      .getByRole('navigation', { name: 'Primary' })
      .getByRole('link', { name: 'Studies' })
      .click();
    await page.getByRole('heading', { name: 'Studies', exact: true }).first().waitFor();
    await page.getByRole('link', { name: studyName }).click();
    await page.getByRole('heading', { name: 'Datasets', exact: true }).waitFor();
    await sleep(1500);
    await page.getByRole('link', { name: 'Resting-state EEG recording' }).click();
    await page.waitForURL(/\/app\/studies\/[0-9a-f-]+\/datasets\/[0-9a-f-]+$/i);
    await page.getByRole('heading', { name: 'Resting-state EEG recording' }).waitFor();
    await page.getByText('Ready', { exact: true }).first().waitFor();
    await sleep(1600);

    log.info('Exploring signal visualisation on dataset detail page');
    // Wait for signal visualisation card to load
    await page.getByRole('heading', { name: 'Signal visualisation' }).waitFor();
    await sleep(2000);

    // Interact with channel selector (exact match to avoid strict mode violation)
    await page.getByLabel('Channel', { exact: true }).click();
    await sleep(300);
    await page.getByRole('option', { name: 'Fp2' }).click();
    await sleep(2000);

    // Change time window
    const startInput = page.locator('input[type="number"]').first();
    const endInput = page.locator('input[type="number"]').nth(1);
    await startInput.fill('2');
    await sleep(200);
    await endInput.fill('6');
    await sleep(2000);

    // Navigate through channels with prev/next (use aria-label)
    await page.getByRole('button', { name: 'Next channel' }).click();
    await sleep(1500);
    await page.getByRole('button', { name: 'Next channel' }).click();
    await sleep(1500);
    await page.getByRole('button', { name: 'Previous channel' }).click();
    await sleep(1500);

    log.info('Submitting analysis via UI on dataset detail page');
    // Submit analysis using the new AnalysisActions component
    await page.getByRole('button', { name: 'Submit analysis' }).waitFor();
    await sleep(600);
    await page.getByRole('button', { name: 'Submit analysis' }).click();
    await sleep(5000);

    // Wait for analysis to appear in the list (look for status badge or short job ID)
    await page.getByRole('heading', { name: 'Analyses' }).waitFor();
    await page.locator('.font-mono').first().waitFor({ timeout: 30_000 });
    await sleep(2000);

    // Verify the analysis job is listed with a status badge (badge element)
    await page.locator('.badge').first().waitFor({ timeout: 10_000 });
    await sleep(1500);

    log.info('Demo completed successfully');
    await sleep(2500);
  } finally {
    await context.close();
    await browser.close();
  }
}

main().catch((error) => {
  console.error('Demo recording failed:', error);
  process.exit(1);
});
