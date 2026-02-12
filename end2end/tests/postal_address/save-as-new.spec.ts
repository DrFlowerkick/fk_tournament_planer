// e2e/tests/save-as-new.spec.ts
import { test, expect } from "@playwright/test";
import {
  openNewForm,
  fillFields,
  clickSave,
  clickSaveAsNew,
  extractUuidFromUrl,
  openEditForm,
  waitForPostalAddressListUrl,
  fillAndBlur,
  selectors,
  searchAndOpenByNameOnCurrentPage,
} from "../../helpers";
import { POSTAL_IDS } from "../../helpers/selectors/postalAddress";

test.describe('"Save as new" functionality', () => {
  test("creates a new address from an existing one", async ({ page }) => {
    const PA = selectors(page).postalAddress;

    // -------------------- Arrange: Create an initial address --------------------
    const initial = {
      name: `E2E SaveAsNew Base ${Date.now()}`,
      street: "Original-Allee 1",
      postal_code: "11111",
      locality: "Startstadt",
      region: "AA",
      country: "DE",
    };

    await openNewForm(page);
    await fillFields(page, initial);
    await clickSave(page);
    await waitForPostalAddressListUrl(page);
    const originalId = extractUuidFromUrl(page.url());

    // -------------------- Act: Edit and "Save as new" --------------------
    await openEditForm(page, originalId);

    // Ensure we are on the edit page for the original address
    await expect(PA.form.hiddenId).toHaveValue(originalId);
    await expect(PA.form.inputName).toHaveValue(initial.name);

    // Change the name and click "Save as new"
    const newName = `E2E SaveAsNew Copy ${Date.now()}`;
    await fillAndBlur(PA.form.inputName, newName);
    await clickSaveAsNew(page);

    // -------------------- Assert: A new address was created --------------------
    // We should be on the view page of the *new* address
    await waitForPostalAddressListUrl(page);

    // The row should show the new name
    await searchAndOpenByNameOnCurrentPage(page, newName, "address_id");
    // Extract ID after click on row, because if table is "full", the ID might be removed from URL
    const newId = extractUuidFromUrl(page.url());
    // The new ID must be different from the original one
    expect(newId).not.toEqual(originalId);

    await expect(PA.list.entryName(newId)).toHaveText(newName);

    // -------------------- Assert: Original address is unchanged --------------------
    await openEditForm(page, originalId);
    await expect(PA.form.inputName).toHaveValue(initial.name);
  });
});
