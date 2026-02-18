import { test, expect } from "@playwright/test";
import {
  fillFields,
  closeForm,
  expectPreviewShows,
  openPostalAddressList,
  searchAndOpenByNameOnCurrentPage,
  extractUuidFromUrl,
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

  await test.step('Close form after automatic save', async () => {
    await closeForm(page);

    await expect(PA.list.btnNew).toBeVisible();
  });

  await test.step("Find the created address via table and verify preview", async () => {
    await searchAndOpenByNameOnCurrentPage(page, initial.name, "address_id");
    await expectPreviewShows(page, initial);
  });
});
