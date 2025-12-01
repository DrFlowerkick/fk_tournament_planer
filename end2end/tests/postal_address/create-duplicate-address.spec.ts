// e2e/create-duplicate-address.spec.ts
import { test, expect } from "@playwright/test";
import {
  openNewForm,
  fillFields,
  clickSave,
  waitForPostalAddressListUrl,
} from "../../helpers/postal_address";
import { T } from "../../helpers/selectors";

test.describe("Uniqueness constraint violation", () => {
  test("shows error banner on duplicate name, postal code, and locality", async ({
    page,
  }) => {
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
    await expect(page.getByTestId(T.banner.acknowledgmentBanner)).toBeVisible();
    await expect(page.getByTestId(T.banner.btnAcknowledgment)).toBeVisible();

    // The banner should contain a warning message.
    await expect(page.getByTestId(T.banner.acknowledgmentBanner)).toContainText(
      `An address with name '${uniqueData.name}' already exists in '${uniqueData.postal_code} ${uniqueData.locality}'.`
    );

    // Save must be disabled while the error is present.
    await expect(page.getByTestId(T.form.btnSave)).toBeDisabled();

    // -------------------- Resolve via dismiss --------------------
    await page.getByTestId(T.banner.btnAcknowledgment).click();

    // After dismiss, the banner should be gone.
    await expect(page.getByTestId(T.banner.acknowledgmentBanner)).toBeHidden();

    // The form fields should be enabled again.
    await expect(page.getByTestId(T.form.inputName)).toBeEnabled();
    await expect(page.getByTestId(T.form.btnSave)).toBeEnabled();
  });
});
