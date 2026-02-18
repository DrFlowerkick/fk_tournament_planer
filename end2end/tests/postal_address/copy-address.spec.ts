// e2e/tests/save-as-new.spec.ts
import { test, expect } from "@playwright/test";
import {
  openNewForm,
  fillFields,
  extractUuidFromUrl,
  openEditForm,
  waitForPostalAddressListUrl,
  waitForPostalAddressEditUrl,
  closeForm,
  fillAndBlur,
  selectors,
  searchAndOpenByNameOnCurrentPage,
} from "../../helpers";
import { POSTAL_IDS } from "../../helpers/selectors/postalAddress";
import exp from "constants";

test.describe('"Copy address" functionality', () => {
  test("creates a new address by copying an existing one", async ({ page }) => {
    const PA = selectors(page).postalAddress;

    // -------------------- Arrange: Create an initial address --------------------
    const initial = {
      name: `E2E CopyAddress Base ${Date.now()}`,
      street: "Original-Allee 1",
      postal_code: "11111",
      locality: "Startstadt",
      region: "AA",
      country: "DE",
    };

    await openNewForm(page);
    await fillFields(page, initial);
    await closeForm(page);

    await waitForPostalAddressListUrl(page, true);
    const originalId = extractUuidFromUrl(page.url());

    // -------------------- Act: Open "Copy address" form --------------------
    // The row should show the new name
    await searchAndOpenByNameOnCurrentPage(page, initial.name, "address_id");

    await expect(PA.list.btnCopy).toBeVisible();
    await PA.list.btnCopy.click();

    // Ensure we are on the edit page for the original address but with new id and empty name
    await expect(PA.form.hiddenId).not.toHaveValue(originalId);
    await expect(PA.form.inputStreet).toHaveValue(initial.street);
    await expect(PA.form.inputName).toHaveValue("");

    // Change the name and click "Copy address"
    const newName = `E2E CopyAddress Copy ${Date.now()}`;
    await fillAndBlur(PA.form.inputName, newName);

    // url should now switch to the new ID and edit
    await waitForPostalAddressEditUrl(page);

    // close form
    await closeForm(page);

    // -------------------- Assert: A new address was created --------------------
    // We should be on the view page of the *new* address
    await waitForPostalAddressListUrl(page, true);

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
