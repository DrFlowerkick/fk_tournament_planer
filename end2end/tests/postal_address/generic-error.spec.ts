// e2e/tests/generic-error.spec.ts
import { test, expect } from "@playwright/test";
import {
  openNewForm,
  fillAllRequiredValid,
} from "../../helpers/postal_address";
import { T } from "../../helpers/selectors";

test.describe("Generic error handling saving address", () => {
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
    await page.waitForLoadState("domcontentloaded");

    // -------------------- Assert: Generic error banner is shown --------------------
    await expect(
      page.getByTestId(T.banner.acknowledgmentNavigateBanner)
    ).toBeVisible();
    await expect(
      page.getByTestId(T.banner.acknowledgmentNavigateBanner)
    ).toContainText("An unexpected error occurred during saving:");
    await expect(
      page.getByTestId(T.banner.btnAcknowledgmentNavigateAction)
    ).toBeVisible();
    await expect(
      page.getByTestId(T.banner.btnAcknowledgmentNavigate)
    ).toBeVisible();

    // form should be disabled
    await expect(page.getByTestId(T.form.btnSave)).toBeDisabled();

    // -------------------- Act: Dismiss the banner --------------------
    await page.getByTestId(T.banner.btnAcknowledgmentNavigateAction).click();

    // -------------------- Assert: Banner is gone --------------------
    await expect(
      page.getByTestId(T.banner.acknowledgmentNavigateBanner)
    ).toBeHidden();

    // form should be enabled again
    await expect(page.getByTestId(T.form.btnSave)).toBeEnabled();

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
    await page.getByTestId(T.form.btnSave).click();

    // -------------------- Assert: Generic error banner is shown --------------------
    await expect(
      page.getByTestId(T.banner.acknowledgmentNavigateBanner)
    ).toBeVisible();
    await expect(
      page.getByTestId(T.banner.acknowledgmentNavigateBanner)
    ).toContainText("An unexpected error occurred during saving:");
    await expect(
      page.getByTestId(T.banner.btnAcknowledgmentNavigateAction)
    ).toBeVisible();
    await expect(
      page.getByTestId(T.banner.btnAcknowledgmentNavigate)
    ).toBeVisible();

    // form should be disabled
    await expect(page.getByTestId(T.form.btnSave)).toBeDisabled();

    // -------------------- Act: return to list --------------------
    await page.getByTestId(T.banner.btnAcknowledgmentNavigate).click();

    // -------------------- Assert: We are back on the main list page --------------------
    await expect(page).toHaveURL(`/postal-address`);
    await expect(page.getByTestId(T.search.dropdown.input)).toBeVisible();
  });
});
