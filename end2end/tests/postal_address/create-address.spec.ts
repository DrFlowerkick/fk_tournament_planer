import { test, expect } from "@playwright/test";
import { selectors } from "../../helpers/selectors";
import {
  fillFields,
  clickSave,
  expectPreviewShows,
  openPostalAddressList,
} from "../../helpers/postal_address";
import { searchAndOpenByNameOnCurrentPage } from "../../helpers/utils";

test("Create Address (happy path): New → Fill → Save → Verify in search", async ({
  page,
}) => {
  const S = selectors(page);

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
    await S.search.btnNew.click();
    await expect(S.form.root).toBeVisible();
  });

  await test.step("Fill form", async () => {
    await fillFields(page, initial);
  });

  await test.step('Save with "save-as-new"', async () => {
    await clickSave(page);
    // The app may stay on the form or navigate; we normalize by going to search.
    await page.goto("/postal-address");
    await expect(S.search.dropdown.input).toBeVisible();
  });

  await test.step("Find the created address via search", async () => {
    await searchAndOpenByNameOnCurrentPage(selectors(page).search.dropdown, initial.name, {
      clearFirst: true,
      expectUnique: true,
      waitAriaBusy: true,
    });
  });

  await test.step("Verify preview shows the saved data", async () => {
    await expectPreviewShows(page, initial);
  });
});
