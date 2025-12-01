// e2e/tests/error-loading-non-existing-id.spec.ts
import { test, expect } from "@playwright/test";
import { openPostalAddressList } from "../../helpers/postal_address";
import { T } from "../../helpers/selectors";

test.describe("Error loading non-existing postal address ID", () => {
  test("shows error message when navigating to non-existing ID", async ({
    page,
  }) => {
    // Open the postal address list first
    await openPostalAddressList(page);

    // Navigate to a non-existing postal address ID
    const nonExistingId = "00000000-0000-0000-0000-000000000000";
    await page.goto(`/postal-address?address_id=${nonExistingId}`);
    await page.waitForLoadState("domcontentloaded");

    // Assert that the error message is displayed
    await expect(
      page.getByTestId(T.banner.acknowledgmentNavigateBanner)
    ).toBeVisible();
    await expect(
      page.getByTestId(T.banner.acknowledgmentNavigateBanner)
    ).toContainText("Postal Address ID not found");
    await expect(
      page.getByTestId(T.banner.btnAcknowledgmentNavigateAction)
    ).toBeVisible();
    await expect(
      page.getByTestId(T.banner.btnAcknowledgmentNavigate)
    ).toBeVisible();

    // -------------------- Act: Dismiss the banner --------------------
    await page.getByTestId(T.banner.btnAcknowledgmentNavigate).click();

    // -------------------- Assert: Banner is gone --------------------
    await expect(
      page.getByTestId(T.banner.acknowledgmentNavigateBanner)
    ).toBeHidden();

    // Assert we are back on the list page without any id in the URL
    await expect(page).toHaveURL(/\/postal-address$/);
  });
});
