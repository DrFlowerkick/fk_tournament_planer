// e2e/create-duplicate-address.spec.ts
import { test, expect } from "@playwright/test";
import {
  openNewForm,
  fillFields,
  clickSave,
  waitForPostalAddressListUrl,
  selectors
} from "../../helpers";

test.describe("Uniqueness constraint violation", () => {
  test("shows error banner on duplicate name, postal code, and locality", async ({
    page,
  }) => {
    const PA = selectors(page).postalAddress;
    const BA = selectors(page).banners;

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

    // -------------------- Assert: Duplicate error UI appears --------------------
    // A banner should appear, and the dismiss button should be visible.
    await expect(BA.acknowledgment.root).toBeVisible();
    await expect(BA.acknowledgment.btnAction).toBeVisible();

    // The banner should contain a warning message.
    await expect(BA.acknowledgment.root).toContainText(
      `An address with name '${uniqueData.name}' already exists in '${uniqueData.postal_code} ${uniqueData.locality}'.`
    );

    // Save must be disabled while the error is present.
    await expect(PA.form.btnSave).toBeDisabled();

    // -------------------- Resolve via dismiss --------------------
    await BA.acknowledgment.btnAction.click();

    // After dismiss, the banner should be gone.
    await expect(BA.acknowledgment.root).toBeHidden();

    // The form fields should be enabled again.
    await expect(PA.form.inputName).toBeEnabled();
    await expect(PA.form.btnSave).toBeEnabled();
  });
});
