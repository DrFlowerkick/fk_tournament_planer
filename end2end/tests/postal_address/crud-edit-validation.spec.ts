import { test } from "@playwright/test";
import {
  openNewForm,
  fillAllRequiredValid,
  closeForm,
  clickEditPostalAddress,
  expectPreviewShows,
  waitForPostalAddressListUrl,
  extractUuidFromUrl,
  fillAndBlur,
  expectFieldValidity,
  searchAndOpenByNameOnCurrentPage,
  selectors,
} from "../../helpers";

/**
 * Flow:
 * 1) Create a new address (all fields valid, normalized) and save.
 * 2) Open the newly created address, enter edit mode.
 * 3) Make one field invalid (DE-specific ZIP example) → Save buttons must be disabled.
 * 4) Fix the field → Save buttons become enabled; save edit.
 * 5) Verify the edited address is shown with updated values.
 */
test.describe("Create → Edit → Invalid forbids save → Fix → Save → Verify edited address", () => {
  test("end-to-end edit validation gate and final save", async ({ page }) => {
    const PA = selectors(page).postalAddress;

    // Step 1: Create new valid address and save
    const ts = Date.now();
    const name = `E2E Test Address ${ts}`;
    await openNewForm(page);
    await fillAllRequiredValid(page, name);
    await closeForm(page);

    // After save, either you land on detail page or back to list.
    await waitForPostalAddressListUrl(page, true);
    await searchAndOpenByNameOnCurrentPage(page, name, "address_id");
    
    // Extract ID after click on row, because if table is "full", the ID might be removed from URL
    const uuid = extractUuidFromUrl(page.url());

    await expectPreviewShows(page, {
      name: name,
      street: "Beispielstr. 1",
      postal_code: "10115",
      locality: "Berlin Mitte",
      country: "DE",
    });

    // Step 2: Enter edit mode
    await clickEditPostalAddress(page);

    // Step 3: Make a field invalid → save buttons must be disabled
    /**
     * NOTE (DE-specific):
     * The next assertion uses a German postal code rule
     * (exactly 5 digits after normalization). This is not generic for all countries.
     */
    await fillAndBlur(PA.form.inputStreet, "");
    await expectFieldValidity(PA.form.inputStreet, "", /*invalid*/ true);

    // Step 4: Fix invalid field, which will be automatically saved
    await fillAndBlur(PA.form.inputStreet, "   Beispielstr.    3   ");
    await expectFieldValidity(
      PA.form.inputStreet,
      "Beispielstr. 3",
      /*invalid*/ false,
    );
    await closeForm(page);

    // Step 5: Verify that edited address is displayed with updated values
    await waitForPostalAddressListUrl(page, true);
    const uuid_edited = extractUuidFromUrl(page.url());
    test.expect(uuid_edited).toBe(uuid); // same id
    await expectPreviewShows(page, {
      street: "Beispielstr. 3",
    });
  });
});
