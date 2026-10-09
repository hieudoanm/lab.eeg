import { expect, test } from '@playwright/test';

test('presents the public landing page and links to sign in', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByRole('heading', { level: 1 })).toContainText(
    'Organise studies, explore EEG signals',
  );

  const signIn = page.getByRole('link', { name: 'Sign in' }).first();
  await expect(signIn).toHaveAttribute('href', '/login');
  await signIn.click();
  await expect(page).toHaveURL(/\/login$/);
  await expect(page.getByRole('heading', { name: 'Sign in' })).toBeVisible();
});

test('shows the sign-in experience for unauthenticated visitors', async ({ page }) => {
  await page.goto('/app/dashboard');

  await expect(page).toHaveURL(/\/login$/);
  await expect(page.getByRole('heading', { name: 'Sign in' })).toBeVisible();
  await expect(page.getByLabel(/Email address/)).toBeVisible();
  await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
});

test('shows a friendly not-found page for unknown routes', async ({ page }) => {
  await page.goto('/app/this-route-does-not-exist');

  await expect(page.getByRole('heading', { name: 'Page not found' })).toBeVisible();
});

const apiBaseUrl = process.env.NEUROLAB_E2E_API_BASE;

test.describe('authenticated workspace', () => {
  test.skip(apiBaseUrl === undefined, 'Set NEUROLAB_E2E_API_BASE to run against a live API');

  test('renders the workspace shell and supports navigation', async ({
    page,
    request,
    context,
  }) => {
    const response = await request.post(`${apiBaseUrl}/auth/token`, {
      data: { subject: 'e2e@example.org', display_name: 'E2E Researcher' },
    });
    expect(response.ok()).toBeTruthy();
    const { access_token: token } = (await response.json()) as { access_token: string };
    await context.addCookies([
      { name: 'eeg-lab_session', value: token, domain: '127.0.0.1', path: '/' },
    ]);

    await page.goto('/app/dashboard');

    await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible();
    await expect(page.getByText('EEG Lab', { exact: true })).toBeVisible();

    await page.getByRole('link', { name: 'Studies' }).click();
    await expect(page).toHaveURL(/\/app\/studies$/);
    await expect(page.getByRole('heading', { name: 'Studies', exact: true })).toBeVisible();

    await page.getByRole('link', { name: 'Dashboard' }).click();
    await expect(page).toHaveURL(/\/app\/dashboard$/);
  });
});
