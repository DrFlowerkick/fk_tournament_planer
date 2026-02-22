// e2e/create-duplicate-address.spec.ts
import { test, expect } from "@playwright/test";
import {
  openPostalAddressList,
  clickNewPostalAddress,
  fillFields,
  expectFieldValidity,
  closeForm,
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

    await openPostalAddressList(page);
    await clickNewPostalAddress(page);
    await fillFields(page, initial);
    await closeForm(page);
    await waitForPostalAddressListUrl(page, true);

    // -------------------- Act: Try to create duplicate --------------------
    await clickNewPostalAddress(page);
    const duplicate = {
      name: uniqueData.name, // Same name
      street: "Nebenstrasse 2", // Different street
      postal_code: uniqueData.postal_code, // Same postal code
      locality: uniqueData.locality, // Same locality
      region: "BY",
      country: "DE",
    };
    await fillFields(page, duplicate);

    // -------------------- Assert: Duplicate validation error appears --------------------
    await expectFieldValidity(PA.form.inputName, duplicate.name, /*invalid*/ true);

    // The form should still be open with the duplicate data (not navigated away)
    await expect(PA.form.inputName).toHaveValue(duplicate.name);
    await expect(PA.form.inputStreet).toHaveValue(duplicate.street);
    await expect(PA.form.inputPostalCode).toHaveValue(duplicate.postal_code);
    await expect(PA.form.inputLocality).toHaveValue(duplicate.locality);
    await expect(PA.form.inputRegion).toHaveValue(duplicate.region);
    await expect(PA.form.inputCountry).toHaveValue(duplicate.country);

  });
});
