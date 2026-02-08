import { test, expect } from "@playwright/test";
import {
  fillFields,
  clickSave,
  expectPreviewShows,
  openPostalAddressList,
  searchAndOpenByNameOnCurrentPage,
  waitForAppHydration,
  selectors
} from "../../helpers";

test("Create Address (happy path): New → Fill → Save → Verify in search", async ({
  page,
}) => {
  const PA = selectors(page).postalAddress;

  // Use values that make assertions obvious and avoid trimming/casing ambiguity.
  const ts = Date.now();
  const initial = {
    name: `E2E Test Address ${ts}`,
    street: "Main Street 42",
    postal_code: "10555",
    locality: "Berlin",
    region: "BE",
    country: "DE",
  };

  await test.step("Open search and navigate to New", async () => {
    await openPostalAddressList(page);
    await PA.list.btnNew.click();
    await expect(PA.form.root).toBeVisible();
  });

  await test.step("Fill form", async () => {
    await fillFields(page, initial);
  });

  await test.step('Save with "save-as-new"', async () => {
    await clickSave(page);
    // The app may stay on the form or navigate; we normalize by going to search.
    await page.goto("/postal-address");

    // Wait for hydration after raw navigation
    await waitForAppHydration(page);

    await expect(PA.list.btnNew).toBeVisible();
  });

  await test.step("Find the created address via table and verify preview", async () => {
    const row = await searchAndOpenByNameOnCurrentPage(page, initial.name);
    await expectPreviewShows(row, initial);
  });
});
