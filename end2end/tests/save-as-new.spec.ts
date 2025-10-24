// e2e/tests/save-as-new.spec.ts
import { test, expect } from "@playwright/test";
import {
  openNewForm,
  fillAll,
  clickSave,
  clickSaveAsNew,
  extractUuidFromUrl,
  typeThenBlur,
  waitForPostalAddressListLoadedWithUrl,
} from "../helpers/form";
import { T } from "../helpers/selectors";

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
    await fillAll(
      page,
      initial.name,
      initial.street,
      initial.postal_code,
      initial.locality,
      initial.region,
      initial.country
    );
    await clickSave(page);
    await waitForPostalAddressListLoadedWithUrl(page);
    const originalId = extractUuidFromUrl(page.url());

    // -------------------- Act: Edit and "Save as new" --------------------
    await page.goto(`/postal-address/${originalId}/edit`);

    // Ensure we are on the edit page for the original address
    await expect(page.getByTestId(T.form.hiddenId)).toHaveValue(originalId);
    await expect(page.getByTestId(T.form.inputName)).toHaveValue(initial.name);

    // Change the name and click "Save as new"
    const newName = `E2E SaveAsNew Copy ${Date.now()}`;
    await typeThenBlur(page, T.form.inputName, newName, T.form.btnSaveAsNew);
    await clickSaveAsNew(page);

    // -------------------- Assert: A new address was created --------------------
    // We should be on the view page of the *new* address
    await waitForPostalAddressListLoadedWithUrl(page);
    const newId = extractUuidFromUrl(page.url());

    // The new ID must be different from the original one
    expect(newId).not.toEqual(originalId);

    // The preview should show the new name
    await expect(page.getByTestId(T.search.preview.name)).toHaveText(newName);

    // -------------------- Assert: Original address is unchanged --------------------
    await page.goto(`/postal-address/${originalId}/edit`);
    await expect(page.getByTestId(T.form.inputName)).toHaveValue(initial.name);
  });
});
