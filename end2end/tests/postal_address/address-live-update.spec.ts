import { test, expect, Page } from "@playwright/test";
import {
  openNewForm,
  fillFields,
  clickSave,
  expectPreviewShows,
  extractUuidFromUrl,
  expectSavesDisabled,
  expectSavesEnabled,
  openEditForm,
  waitForPostalAddressListUrl,
  typeThenBlur,
  waitForAppHydration,
  selectors
} from "../../helpers";

// --- Test data ---------------------------------------------------------------
// Unique test data (avoid partial-unique collisions)
const ts = Date.now();
const initial = {
  // Use values that make assertions obvious and avoid trimming/casing ambiguity.
  name: `E2E Test Address ${ts}`,
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
      // as long as other required fields are empty/invalid, saving must remain disabled
      await expectSavesDisabled(pageA);

      await fillFields(pageA, initial);
      await clickSave(pageA);

      // After save, route should be /postal-address/<uuid>
      await waitForPostalAddressListUrl(pageA);
      const urlA = pageA.url();
      const id = extractUuidFromUrl(urlA);

      // Ensure the preview shows the initial values and correct version
      await expectPreviewShows(pageA, initial);
      await expect(PA_A.search.preview.version).toHaveText("0");

      // ----------------------- Act (B edits & saves) -------------------------
      // B opens the edit route directly for the same UUID.
      await openEditForm(pageB, id);
      // now save button should be enabled
      await expectSavesEnabled(pageB);

      // Change just the name; other fields remain as-is.
      await typeThenBlur(
        PA_B.form.inputName,
        edited.name,
        PA_B.form.inputStreet
      );

      await clickSave(pageB);

      // ----------------------- Assert (A updates via SSE) --------------------
      // wait for new version
      await expect(PA_A.search.preview.version).toHaveText("1");
      // A's preview should reflect the edited name.
      await expect(PA_A.search.preview.name).toHaveText(edited.name);

      // Optional sanity check: A did not navigate away (no hard reload).
      await expect(pageA).toHaveURL(urlA);
    } finally {
      // Cleanup contexts.
      await ctxA.close();
      await ctxB.close();
    }
  });
});
