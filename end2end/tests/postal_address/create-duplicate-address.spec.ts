// e2e/create-duplicate-address.spec.ts
import { test, expect } from "@playwright/test";
import {
  openNewForm,
  fillFields,
  clickSave,
  waitForPostalAddressListUrl,
  selectors,
} from "../../helpers";

test.describe("Uniqueness constraint violation", () => {
  test("shows error banner on duplicate name, postal code, and locality", async ({
    page,
  }) => {
    const PA = selectors(page).postalAddress;
    const TOAST = selectors(page).toasts;

    // -------------------- Arrange: Create first address --------------------
    const uniqueData = {
      name: `E2E Unique ${Date.now()}`,
      postal_code: "54321",
      locality: "Doppelstadt",
    };
    const initial = {
      name: uniqueData.name,
      street: "Hauptstrasse 1",
      postal_code: uniqueData.postal_code,
      locality: uniqueData.locality,
      region: "BY",
      country: "DE",
    };

    await openNewForm(page);
    await fillFields(page, initial);
    await clickSave(page);
    await waitForPostalAddressListUrl(page);

    // -------------------- Act: Try to create duplicate --------------------
    await openNewForm(page);
    const duplicate = {
      name: uniqueData.name, // Same name
      street: "Nebenstrasse 2", // Different street
      postal_code: uniqueData.postal_code, // Same postal code
      locality: uniqueData.locality, // Same locality
      region: "BY",
      country: "DE",
    };
    await fillFields(page, duplicate);
    await clickSave(page);

    // -------------------- Assert: Duplicate error Toast appears --------------------
    // A toast should appear
    await expect(TOAST.error).toBeVisible();

    // The toast should contain a warning message.
    await expect(TOAST.error).toContainText(`A unique value is already in use`);

    // The form should still be open with the duplicate data (not navigated away)
    await expect(PA.form.inputName).toHaveValue(duplicate.name);
    await expect(PA.form.inputStreet).toHaveValue(duplicate.street);
    await expect(PA.form.inputPostalCode).toHaveValue(duplicate.postal_code);
    await expect(PA.form.inputLocality).toHaveValue(duplicate.locality);
    await expect(PA.form.inputRegion).toHaveValue(duplicate.region);
    await expect(PA.form.inputCountry).toHaveValue(duplicate.country);

    // Toast should disappear after some time
    await expect(TOAST.error).toBeHidden({ timeout: 10000 });
  });
});
