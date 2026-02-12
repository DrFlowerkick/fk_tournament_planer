// e2e/tests/error-loading-non-existing-id.spec.ts
import { test, expect } from "@playwright/test";
import {
  openPostalAddressList,
  selectors,
  waitForAppHydration,
} from "../../helpers";
import { PA_QUERY_KEYS, PA_ROUTES } from "../../helpers/utils/postal_address";

test.describe("Error loading non-existing postal address ID", () => {
  test("shows error message when navigating to non-existing ID", async ({
    page,
  }) => {
    const PA = selectors(page).postalAddress;
    const BA = selectors(page).banners;

    // -------------------- Arrange: Ensure we are on the list page --------------------
    // Open the postal address list first (Helper is safe)
    await openPostalAddressList(page);

    // Navigate to a non-existing postal address ID
    const nonExistingId = "00000000-0000-0000-0000-000000000000";
    await page.goto(`${PA_ROUTES.editAddress}?${PA_QUERY_KEYS.addressId}=${nonExistingId}`);

    // add hydration check after raw navigation
    await waitForAppHydration(page);

    // Assert that the error message is displayed
    await expect(BA.globalErrorBanner.root).toBeVisible();
    await expect(BA.globalErrorBanner.root).toContainText(
      "'Postal Address' could not be found.",
    );
    await expect(BA.globalErrorBanner.btnRetry).toBeVisible();
    await expect(BA.globalErrorBanner.btnCancel).toBeVisible();

    // -------------------- Act: Dismiss the banner --------------------
    await BA.globalErrorBanner.btnCancel.click();

    // -------------------- Assert: Banner is gone --------------------
    await expect(BA.globalErrorBanner.root).toBeHidden();

    // Assert we are back on the list page with any address_id query param
    await expect(page).toHaveURL(/\/postal-address\?address_id=.+$/);
  });
});
