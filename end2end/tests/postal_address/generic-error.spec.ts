// e2e/tests/generic-error.spec.ts
import { test, expect } from "@playwright/test";
import {
  openNewForm,
  fillAllRequiredValid,
} from "../../helpers/postal_address";
import { selectors } from "../../helpers/selectors";

test.describe("Generic error handling saving address", () => {
  test("shows a generic error banner on 500 server error", async ({ page }) => {
    const PA = selectors(page).postalAddress;
    const BA = selectors(page).banners;

    // -------------------- Arrange: Intercept server response --------------------
    await page.route("/api/save_postal_address*", (route) => {
      route.fulfill({
        status: 500,
        contentType: "application/json",
        body: JSON.stringify({
          error: "Internal Server Error",
          message: "A simulated unexpected error occurred.",
        }),
      });
    });

    // -------------------- Act: Try to save a new address --------------------
    await openNewForm(page);
    await fillAllRequiredValid(page, `E2E GenericError ${Date.now()}`);
    await PA.form.btnSave.click();
    await page.waitForLoadState("domcontentloaded");

    // -------------------- Assert: Generic error banner is shown --------------------
    await expect(BA.acknowledgmentNavigate.root).toBeVisible();
    await expect(BA.acknowledgmentNavigate.root).toContainText(
      "An unexpected error occurred during saving:"
    );
    await expect(BA.acknowledgmentNavigate.btnAction).toBeVisible();
    await expect(BA.acknowledgmentNavigate.btnNavigate).toBeVisible();

    // form should be disabled
    await expect(PA.form.btnSave).toBeDisabled();

    // -------------------- Act: Dismiss the banner --------------------
    await BA.acknowledgmentNavigate.btnAction.click();

    // -------------------- Assert: Banner is gone --------------------
    await expect(BA.acknowledgmentNavigate.root).toBeHidden();

    // form should be enabled again
    await expect(PA.form.btnSave).toBeEnabled();

    // -------------------- Arrange: Intercept server response --------------------
    await page.route("/api/save_postal_address*", (route) => {
      route.fulfill({
        status: 500,
        contentType: "application/json",
        body: JSON.stringify({
          error: "Internal Server Error",
          message: "A simulated unexpected error occurred.",
        }),
      });
    });

    // -------------------- Act: Try to save again --------------------
    await PA.form.btnSave.click();

    // -------------------- Assert: Generic error banner is shown --------------------
    await expect(BA.acknowledgmentNavigate.root).toBeVisible();
    await expect(BA.acknowledgmentNavigate.root).toContainText(
      "An unexpected error occurred during saving:"
    );
    await expect(BA.acknowledgmentNavigate.btnAction).toBeVisible();
    await expect(BA.acknowledgmentNavigate.btnNavigate).toBeVisible();

    // form should be disabled
    await expect(PA.form.btnSave).toBeDisabled();

    // -------------------- Act: return to list --------------------
    await BA.acknowledgmentNavigate.btnNavigate.click();

    // -------------------- Assert: We are back on the main list page --------------------
    await expect(page).toHaveURL(`/postal-address`);
    await expect(PA.search.dropdown.input).toBeVisible();
  });
});
