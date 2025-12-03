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
} from "../../helpers/postal_address";
import { T, selectors } from "../../helpers/selectors";
import { searchAndOpenByNameOnCurrentPage } from "../../helpers/utils";

test.describe("Cancel button navigation", () => {
  test("returns to the previous page (search list)", async ({ page }) => {
    // -------------------- Arrange: Create an address and find it --------------------
    const name = `E2E CancelNav ${Date.now()}`;
    await openNewForm(page);
    await fillAllRequiredValid(page, name);
    await clickSave(page);
    await waitForPostalAddressListUrl(page);
    const url = page.url();
    const uuid = extractUuidFromUrl(url);

    // Go to list and search for it to create a history entry
    await expect(page.getByTestId(T.search.dropdown.input)).toBeVisible();
    await searchAndOpenByNameOnCurrentPage(selectors(page).search.dropdown, name);

    // -------------------- Act: Go to edit and click cancel --------------------
    await page.getByTestId(T.search.btnEdit).click();
    await expect(page.getByTestId(T.form.root)).toBeVisible();
    await page.getByTestId(T.form.btnCancel).click();

    // -------------------- Assert: We are back on the search page with uuid in url--------------------
    // The URL should be the search/list URL, not the edit URL
    await expect(page).toHaveURL(url);
    // The search input should be visible again
    await expect(page.getByTestId(T.search.dropdown.input)).toBeVisible();

    // -------------------- Act: Open edit page directly and cancel --------------------
    await openPostalAddressList(page); // Ensure we have no uuid in current url
    await openEditForm(page, uuid);

    await expect(page.getByTestId(T.form.root)).toBeVisible();
    await page.getByTestId(T.form.btnCancel).click();

    // -------------------- Assert: We are on the main list page now with id in url --------------------
    await waitForPostalAddressListUrl(page);
    const url_after_cancel = page.url();
    const uuid_after_cancel = extractUuidFromUrl(url_after_cancel);
    expect(uuid_after_cancel).toBe(uuid);
  });
});
