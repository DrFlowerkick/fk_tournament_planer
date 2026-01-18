// e2e/tests/error-loading-non-existing-id.spec.ts
import { test, expect } from "@playwright/test";
import { openPostalAddressList } from "../../helpers/postal_address";
import { selectors } from "../../helpers/selectors";
import { waitForAppHydration } from "../../helpers/utils"; // ADDED IMPORT

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
    await page.goto(`/postal-address?address_id=${nonExistingId}`);

    // add hydration check after raw navigation
    await waitForAppHydration(page);

    // Assert that the error message is displayed
    await expect(BA.acknowledgmentNavigate.root).toBeVisible();
    await expect(BA.acknowledgmentNavigate.root).toContainText(
      "resource not found: Postal Address"
    );
    await expect(BA.acknowledgmentNavigate.btnAction).toBeVisible();
    await expect(BA.acknowledgmentNavigate.btnNavigate).toBeVisible();

    // -------------------- Act: Dismiss the banner --------------------
    await BA.acknowledgmentNavigate.btnNavigate.click();

    // -------------------- Assert: Banner is gone --------------------
    await expect(BA.acknowledgmentNavigate.root).toBeHidden();

    // Assert we are back on the list page without any id in the URL
    await expect(page).toHaveURL(/\/postal-address$/);
  });
});
