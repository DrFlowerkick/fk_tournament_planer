// e2e/tests/generic-error.spec.ts
import { test, expect } from "@playwright/test";
import {
  openNewForm,
  openPostalAddressList,
  fillAllRequiredValid,
  extractQueryParamFromUrl,
  selectors,
} from "../../helpers/";

test.describe("Generic error handling saving address", () => {
  test("shows a generic error banner on 500 server error", async ({ page }) => {
    const PA = selectors(page).postalAddress;
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

test.describe("Generic error handling loading address", () => {
  test("shows a generic error banner on 500 server error", async ({ page }) => {
    const PA = selectors(page).postalAddress;
    const BA = selectors(page).banners;

    // ---------------- Arrange: get ID of existing address -------------------
    await openPostalAddressList(page);
    const FIRST_ROW = await PA.list.anyRow.first();
    FIRST_ROW.click();
    await expect(PA.list.btnEdit).toBeVisible();
    const addressId = extractQueryParamFromUrl(page.url(), "address_id");

    // ---------------- Arrange: Intercept server response --------------------
    await page.route(/\/api\/load_postal_address/, (route) => {
      route.fulfill({
        status: 500,
        contentType: "application/json",
        // The JSON structure must match the Rust Enum 'AppError::Other(String)'
        body: JSON.stringify({
          Other: "A simulated unexpected error occurred while loading.",
        }),
      });
    });

    // -------------------- Act: Try to load the address --------------------
    await PA.list.btnEdit.click();
    await page.waitForLoadState("domcontentloaded");

    // -------------------- Assert: error banner is shown --------------------
    await expect(BA.globalErrorBanner.root).toBeVisible();
    await expect(BA.globalErrorBanner.root).toContainText(
      "A simulated unexpected error occurred while loading.",
    );

    // -------------------- Assert: error banner has a retry button --------------------
    await expect(BA.globalErrorBanner.btnRetry).toBeVisible();

    // -------------------- Act: Click retry and assert banner disappears --------------------
    // IMPORTANT: Remove the route interception before retrying,
    // so the next request actually hits the (mocked) server or proceeds normally.
    await page.unroute(/\/api\/load_postal_address/);

    await BA.globalErrorBanner.btnRetry.click();

    // Now the retry should succeed, the banner should disappear, and the form should render.
    await expect(BA.globalErrorBanner.root).toBeHidden();
    await expect(page.url()).toContain(
      `/postal-address/edit?address_id=${addressId}`,
    );
    await expect(PA.form.root).toBeVisible();
  });
});
