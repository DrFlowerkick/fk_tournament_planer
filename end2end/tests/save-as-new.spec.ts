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
} from "../helpers/postal_address";
import { T } from "../helpers/selectors";
import { typeThenBlur } from "../helpers/utils";

test.describe('"Save as new" functionality', () => {
  test("creates a new address from an existing one", async ({ page }) => {
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
    await expect(page.getByTestId(T.form.hiddenId)).toHaveValue(originalId);
    await expect(page.getByTestId(T.form.inputName)).toHaveValue(initial.name);

    // Change the name and click "Save as new"
    const newName = `E2E SaveAsNew Copy ${Date.now()}`;
    await typeThenBlur(page, T.form.inputName, newName, T.form.btnSaveAsNew);
    await clickSaveAsNew(page);

    // -------------------- Assert: A new address was created --------------------
    // We should be on the view page of the *new* address
    await waitForPostalAddressListUrl(page);
    const newId = extractUuidFromUrl(page.url());

    // The new ID must be different from the original one
    expect(newId).not.toEqual(originalId);

    // The preview should show the new name
    await expect(page.getByTestId(T.search.preview.name)).toHaveText(newName);

    // -------------------- Assert: Original address is unchanged --------------------
    await openEditForm(page, originalId);
    await expect(page.getByTestId(T.form.inputName)).toHaveValue(initial.name);
  });
});
