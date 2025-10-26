// e2e/tests/cancel-navigation.spec.ts
import { test, expect } from "@playwright/test";
import {
  openNewForm,
  fillAllRequiredValid,
  clickSave,
  openPostalAddressList,
  searchAndOpenByNameOnCurrentPage,
} from "../helpers/form";
import { T } from "../helpers/selectors";
import exp from "constants";

test.describe("Cancel button navigation", () => {
  test("returns to the previous page (search list)", async ({ page }) => {
    // -------------------- Arrange: Create an address and find it --------------------
    const name = `E2E CancelNav ${Date.now()}`;
    await openNewForm(page);
    await fillAllRequiredValid(page, name);
    await clickSave(page);
    await page.waitForURL(/\/postal-address\/[0-9a-f-]{36}$/);
    const url = page.url();

    // Go to list and search for it to create a history entry
    await expect(page.getByTestId(T.search.input)).toBeVisible();
    await searchAndOpenByNameOnCurrentPage(page, name);

    // -------------------- Act: Go to edit and click cancel --------------------
    await page.getByTestId(T.search.btnModify).click();
    await expect(page.getByTestId(T.form.root)).toBeVisible();
    await page.getByTestId(T.form.btnCancel).click();

    // -------------------- Assert: We are back on the search page with uuid in url--------------------
    // The URL should be the search/list URL, not the edit URL
    await expect(page).toHaveURL(url);
    // The search input should be visible again
    await expect(page.getByTestId(T.search.input)).toBeVisible();

    // -------------------- Act: Open edit page directly and cancel --------------------
    await openPostalAddressList(page); // Ensure we have no uuid in current url
    await page.goto(`${url}/edit`);
    await expect(page.getByTestId(T.form.root)).toBeVisible();
    await page.getByTestId(T.form.btnCancel).click();
    
    // -------------------- Assert: We are on the main list page --------------------
    await expect(page).toHaveURL("/postal-address");
    await expect(page.getByTestId(T.search.input)).toBeVisible();
  });
});
