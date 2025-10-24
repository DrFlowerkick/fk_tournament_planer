import { test, expect } from "@playwright/test";
import { selectors } from "../helpers/selectors";
import {
  searchAndOpenByNameOnCurrentPage,
  fillAll,
  clickSave,
  expectPreviewShows,
  openPostalAddressList,
} from "../helpers/form";

test("Create Address (happy path): New → Fill → Save → Verify in search", async ({
  page,
}) => {
  const S = selectors(page);

  // Unique test data (avoid partial-unique collisions)
  const ts = Date.now();
  const name = `E2E Test Address ${ts}`;
  const street = "Main Street 42";
  const postal = "10555";
  const locality = "Berlin";
  const region = "BE";
  const country = "DE";

  await test.step("Open search and navigate to New", async () => {
    await openPostalAddressList(page);
    await S.search.btnNew.click();
    await expect(S.form.root).toBeVisible();
  });

  await test.step("Fill form", async () => {
    await fillAll(page, name, street, postal, locality, region, country);
  });

  await test.step('Save with "save-as-new"', async () => {
    await clickSave(page);
    // The app may stay on the form or navigate; we normalize by going to search.
    await page.goto("/postal-address");
    await expect(S.search.input).toBeVisible();
  });

  await test.step("Find the created address via search", async () => {
    await searchAndOpenByNameOnCurrentPage(page, name, {
      clearFirst: true,
      expectUnique: true,
      waitAriaBusy: true,
    });
  });

  await test.step("Verify preview shows the saved data", async () => {
    await expectPreviewShows(page, {
      name: name,
      street: street,
      postal_code: postal,
      locality: locality,
      region: region,
      country: country,
    });
  });
});
