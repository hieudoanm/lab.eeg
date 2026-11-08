import { expect, test } from '@playwright/test';

test('app shell renders in browser preview mode', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: /inspect and analyze neural data/i })).toBeVisible();
  await expect(page.getByText('Browser preview')).toBeVisible();
  await expect(page.getByLabel(/project/i).first()).toBeVisible();
  await expect(page.getByLabel(/modality/i)).toBeVisible();
});

test('backend-dependent controls are disabled without the Tauri runtime', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('button', { name: 'Inspect' })).toBeDisabled();
  const creates = page.getByRole('button', { name: 'Create', exact: true });
  await expect(creates).toHaveCount(2);
  for (const button of await creates.all()) {
    await expect(button).toBeDisabled();
  }
});

test('modality switch updates the path placeholder', async ({ page }) => {
  await page.goto('/');
  await page.getByLabel(/modality/i).selectOption('meg');
  await expect(page.getByPlaceholder(/recording-directory/i)).toBeVisible();
  await page.getByLabel(/modality/i).selectOption('eeg');
  await expect(page.getByPlaceholder('/path/to/file')).toBeVisible();
});
