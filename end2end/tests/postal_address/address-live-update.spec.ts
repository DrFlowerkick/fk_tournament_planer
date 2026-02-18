import { test, expect, Page } from "@playwright/test";
import {
  openNewForm,
  fillFields,
  expectPreviewShows,
  extractUuidFromUrl,
  openEditForm,
  clickEditPostalAddress,
  openPostalAddressList,
  closeForm,
  waitForPostalAddressListUrl,
  makeUniqueName,
  fillAndBlur,
  waitForAppHydration,
  selectors,
  searchAndOpenByNameOnCurrentPage,
} from "../../helpers";
import { POSTAL_IDS } from "../../helpers/selectors/postalAddress";

// --- Test data ---------------------------------------------------------------
// Unique test data (avoid partial-unique collisions)
const ts = Date.now();
const uniqueName = makeUniqueName("E2E Test Address");
const initial = {
  // Use values that make assertions obvious and avoid trimming/casing ambiguity.
  name: uniqueName,
  street: "Main Street 42",
  postal_code: "10555",
  locality: "Berlin",
  region: "BE",
  country: "DE",
};

const edited = {
  // Only the name changes; other fields remain identical to ensure focused assertions.
  name: `${initial.name} (edited)`,
};

// --- Test --------------------------------------------------------------------

test.describe("postal address live update (Preview-only UI)", () => {
  test("editing in B updates Preview in A (no reload)", async ({ browser }) => {
    // Create two completely separate browser contexts to simulate two users.
    const ctxA = await browser.newContext();
    const ctxB = await browser.newContext();
    const pageA = await ctxA.newPage();
    const pageB = await ctxB.newPage();

    const PA_A = selectors(pageA).postalAddress;
    const PA_B = selectors(pageB).postalAddress;

    try {
      // -------------------- Arrange (A creates address) ----------------------
      await pageA.goto("/"); // baseURL is assumed to be configured in Playwright config.

      // Wait for hydration after raw navigation
      await waitForAppHydration(pageA);

      // Open and create a new, valid address.
      await openNewForm(pageA);
      await fillFields(pageA, initial);

      // close form to return to list view
      await closeForm(pageA);

      // Ensure we are back on the list page
      await waitForPostalAddressListUrl(pageA, true);

      // Ensure the preview shows the initial values and correct version
      await searchAndOpenByNameOnCurrentPage(pageA, initial.name, "address_id");
      await expectPreviewShows(pageA, initial);
      await expect(
        pageA.getByTestId(POSTAL_IDS.list.preview.version),
      ).toHaveText("0");

      // Extract ID from url
      const urlA = pageA.url();
      const id = extractUuidFromUrl(urlA);

      // ----------------------- Act (B edits & saves) -------------------------
      // Open the list page
      await openPostalAddressList(pageB);
      await searchAndOpenByNameOnCurrentPage(pageB, initial.name, "address_id");
      await clickEditPostalAddress(pageB);
      await expect(PA_B.form.hiddenId).toHaveValue(id);
      await expect(PA_B.form.hiddenVersion).toHaveValue("0");

      // Change just the name; other fields remain as-is.
      await expect(PA_B.form.inputName).toHaveValue(initial.name);
      await fillAndBlur(PA_B.form.inputName, edited.name);

      // save increments version to 1
      await expect(PA_B.form.hiddenVersion).toHaveValue("1");

      // ----------------------- Assert (A updates via SSE) --------------------
      // wait for new version of automatic save to be reflected in A's preview
      await expect(
        pageA.getByTestId(POSTAL_IDS.list.preview.version),
      ).toHaveText("1");

      // A's table entry should reflect the edited name.
      await expect(PA_A.list.entryName(id)).toHaveText(edited.name);

      // Optional sanity check: A did not navigate away (no hard reload).
      await expect(pageA).toHaveURL(urlA);
    } finally {
      // Cleanup contexts.
      await ctxA.close();
      await ctxB.close();
    }
  });
});
