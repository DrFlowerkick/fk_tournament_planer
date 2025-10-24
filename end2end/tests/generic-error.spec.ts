// e2e/tests/generic-error.spec.ts
import { test, expect } from "@playwright/test";
import { openNewForm, fillAllRequiredValid } from "../helpers/form";
import { T } from "../helpers/selectors";

test.describe("Generic error handling", () => {
  test("shows a generic error banner on 500 server error", async ({ page }) => {
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
    await page.getByTestId(T.form.btnSave).click();

    // -------------------- Assert: Generic error banner is shown --------------------
    await expect(page.getByTestId(T.form.genericErrorBanner)).toBeVisible();
    await expect(page.getByTestId(T.form.genericErrorBanner)).toContainText(
      "An unexpected error occurred"
    );
    await expect(page.getByTestId(T.form.btnGenericErrorDismiss)).toBeVisible();

    // form should be disabled
    await expect(page.getByTestId(T.form.btnSave)).toBeDisabled();

    // -------------------- Act: Dismiss the banner --------------------
    await page.getByTestId(T.form.btnGenericErrorDismiss).click();

    // -------------------- Assert: Banner is gone --------------------
    await expect(page.getByTestId(T.form.genericErrorBanner)).toBeHidden();

    // form should be enabled again
    await expect(page.getByTestId(T.form.btnSave)).toBeEnabled();
  });
});
