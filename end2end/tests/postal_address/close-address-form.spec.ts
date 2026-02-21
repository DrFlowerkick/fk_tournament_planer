// e2e/tests/cancel-navigation.spec.ts
import { test, expect } from "@playwright/test";
import {
  fillAllRequiredValid,
  closeForm,
  extractUuidFromUrl,
  clickNewPostalAddress,
  openPostalAddressList,
  waitForPostalAddressListUrl,
  searchAndOpenByNameOnCurrentPage,
  selectors
} from "../../helpers";

test.describe("Close postal address editor form", () => {
  test("returns to the previous page (search list)", async ({ page }) => {
    const PA = selectors(page).postalAddress;
    
    // -------------------- Arrange: Create an address and find it --------------------
    const name = `E2E CancelNav ${Date.now()}`;
    await openPostalAddressList(page);
    await clickNewPostalAddress(page);
    await fillAllRequiredValid(page, name);
    await closeForm(page);
    await waitForPostalAddressListUrl(page, true);
    const url = page.url();
    const uuid = extractUuidFromUrl(url);

    // Go to list and select the created address to enable the edit button
    await searchAndOpenByNameOnCurrentPage(page, name, "address_id");
    await expect(PA.list.btnEdit).toBeVisible();

    // -------------------- Act: Go to edit and click close --------------------
    await PA.list.btnEdit.click();
    await closeForm(page);

    // -------------------- Assert: We are back on the search page with uuid in url--------------------
    // The URL should be the search/list URL, not the edit URL
    await waitForPostalAddressListUrl(page, true);
  });
});
