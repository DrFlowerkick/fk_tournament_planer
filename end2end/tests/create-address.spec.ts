// e2e/tests/create-address.spec.ts
import { test, expect } from '@playwright/test';
import { selectors } from '../helpers/selectors';

test('Create Address (happy path): New → Fill → Save → Verify in search', async ({ page }) => {
  const S = selectors(page);

  // Unique test data (avoid partial-unique collisions)
  const ts = Date.now();
  const name = `E2E Test Address ${ts}`;
  const street = 'Main Street 42';
  const postal = '10555';
  const locality = 'Berlin';
  const region = 'BE';
  const country = 'DE';

  await test.step('Open search and navigate to New', async () => {
    await page.goto('/postal-address');
    await expect(S.search.input).toBeVisible();
    await S.search.btnNew.click();
    await expect(S.form.root).toBeVisible();
  });

  await test.step('Fill form', async () => {
    await S.form.inputName.fill(name);
    await S.form.inputStreet.fill(street);
    await S.form.inputPostalCode.fill(postal);
    await S.form.inputLocality.fill(locality);
    await S.form.inputRegion.fill(region);
    await S.form.inputCountry.fill(country);
  });

  await test.step('Save with "save-as-new"', async () => {
    await expect(S.form.btnSaveAsNew).toBeVisible();
    await S.form.btnSaveAsNew.click();
    // The app may stay on the form or navigate; we normalize by going to search.
    await page.goto('/postal-address');
    await expect(S.search.input).toBeVisible();
  });

  await test.step('Find the created address via search', async () => {
    await S.search.input.fill('');
    await S.search.input.fill(name);
    await expect(S.search.suggestList).toBeAttached();
    await expect(S.search.suggestList).toHaveAttribute('aria-busy', 'false');
    const row = S.search.suggestItems.filter({ hasText: name });
    await expect(row.first()).toBeVisible();

    await expect(row).toHaveCount(1);
    await row.first().click();
  });

  await test.step('Verify preview shows the saved data', async () => {
    await expect(S.search.preview.root).toBeVisible();
    await expect(S.search.preview.name).toHaveText(name);
    await expect(S.search.preview.street).toHaveText(street);
    await expect(S.search.preview.postalLocality).toHaveText(`${postal} ${locality}`);
    await expect(S.search.preview.region).toHaveText(region);
    await expect(S.search.preview.country).toHaveText(country);
  });
});
