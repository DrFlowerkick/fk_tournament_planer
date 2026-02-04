// e2e/tests/generic-error.spec.ts
import { test, expect } from "@playwright/test";
import { openNewForm, fillAllRequiredValid, selectors } from "../../helpers/";

test.describe("Generic error handling saving address", () => {
  test("shows a generic error banner on 500 server error", async ({ page }) => {
    const PA = selectors(page).postalAddress;
    const BA = selectors(page).banners;
    const TO = selectors(page).toasts;

    // -------------------- Arrange: Intercept server response --------------------
    await page.route("/api/save_postal_address*", (route) => {
      route.fulfill({
        status: 500,
        contentType: "application/json",
        // The JSON structure must match the Rust Enum 'AppError::Other(String)'
        body: JSON.stringify({
          Other: "A simulated unexpected error occurred.",
        }),
      });
    });

    // -------------------- Act: Try to save a new address --------------------
    await openNewForm(page);
    await fillAllRequiredValid(page, `E2E GenericError ${Date.now()}`);
    await PA.form.btnSave.click();
    await page.waitForLoadState("domcontentloaded");

    // -------------------- Assert: toast error is shown --------------------
    await expect(TO.error).toBeVisible();
    await expect(TO.error).toContainText(
      "A simulated unexpected error occurred.",
    );

    // -------------------- Assert: toast error is gone after a few seconds --------------------
    // Increase timeout to 10 seconds to allow enough time for the toast to auto-dismiss
    await expect(TO.error).toBeHidden({ timeout: 10000 });
  });
});
