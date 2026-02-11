// e2e/tests/cancel-navigation.spec.ts
import { test, expect } from "@playwright/test";
import {
  openNewForm,
  fillAllRequiredValid,
  clickSave,
  extractUuidFromUrl,
  openEditForm,
  openPostalAddressList,
  waitForPostalAddressListUrl,
  searchAndOpenByNameOnCurrentPage,
  selectors
} from "../../helpers";

test.describe("Cancel button navigation", () => {
  test("returns to the previous page (search list)", async ({ page }) => {
    const PA = selectors(page).postalAddress;
    
    // -------------------- Arrange: Create an address and find it --------------------
    const name = `E2E CancelNav ${Date.now()}`;
    await openNewForm(page);
    await fillAllRequiredValid(page, name);
    await clickSave(page);
    await waitForPostalAddressListUrl(page);
    const url = page.url();
    const uuid = extractUuidFromUrl(url);

    // Go to list and select the created address to enable the edit button
    const row = await searchAndOpenByNameOnCurrentPage(page, name, "address_id");
    await expect(PA.list.btnEdit).toBeVisible();

    // -------------------- Act: Go to edit and click cancel --------------------
    await PA.list.btnEdit.click();
    await expect(PA.form.root).toBeVisible();
    await PA.form.btnCancel.click();

    // -------------------- Assert: We are back on the search page with uuid in url--------------------
    // The URL should be the search/list URL, not the edit URL
    await expect(page).toHaveURL(url);

    // -------------------- Act: Open edit page directly and cancel --------------------
    await openPostalAddressList(page); // Ensure we have no uuid in current url
    await openEditForm(page, uuid);

    await expect(PA.form.root).toBeVisible();
    await PA.form.btnCancel.click();

    // -------------------- Assert: We are on the main list page now with id in url --------------------
    await waitForPostalAddressListUrl(page);
    const url_after_cancel = page.url();
    const uuid_after_cancel = extractUuidFromUrl(url_after_cancel);
    expect(uuid_after_cancel).toBe(uuid);
  });
});
