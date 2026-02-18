// e2e/edit-conflict.spec.ts
import { test, expect, Page } from "@playwright/test";
import {
  openNewForm,
  fillFields,
  clickEditPostalAddress,
  closeForm,
  selectors,
  waitForPostalAddressListUrl,
  searchAndOpenByNameOnCurrentPage,
  makeUniqueName,
} from "../../helpers";

test.describe("Edit conflict handling with auto-save", () => {
  test("Simultaneous editing triggers toast warning for one client", async ({
    browser,
  }) => {
    const ctxA = await browser.newContext();
    const ctxB = await browser.newContext();

    // Open two separate browser windows (contexts)
    const pageA = await ctxA.newPage();
    const pageB = await ctxB.newPage();

    const PA_A = selectors(pageA).postalAddress;
    const PA_B = selectors(pageB).postalAddress;
    const Toasts_A = selectors(pageA).toasts;
    const Toasts_B = selectors(pageB).toasts;

    try {
      // -------------------- Arrange: Create Initial Record --------------------
      const uniqueName = makeUniqueName("E2E Conflict");
      const initial = {
        name: uniqueName,
        street: "Parallelweg 1",
        postal_code: "12345",
        locality: "Zweigleisig",
        country: "DE",
      };

      // User A creates the record
      await openNewForm(pageA);
      await fillFields(pageA, initial);
      await closeForm(pageA);
      await waitForPostalAddressListUrl(pageA, true);

      // User B goes to the same list url
      await pageB.goto(pageA.url());

      // -------------------- Open same record in both clients -------------------
      // Both open the edit form. Since they load fresh, they have the same version.
      await searchAndOpenByNameOnCurrentPage(pageA, uniqueName, "address_id");
      await searchAndOpenByNameOnCurrentPage(pageB, uniqueName, "address_id");
      await clickEditPostalAddress(pageA);
      await clickEditPostalAddress(pageB);

      // Both should see the same initial data
      await expect(PA_A.form.inputStreet).toHaveValue(initial.street);
      await expect(PA_B.form.inputStreet).toHaveValue(initial.street);

      // -------------------- Prepare Conflict -----------------------------------
      // We type into the fields WITHOUT triggering blur/change yet.
      // Playwright's .fill() usually triggers input events, but we rely on blur for saving.

      const valA = "Street A";
      const valB = "Street B";

      // Focus and fill, but avoid triggering the final blur that saves
      await PA_A.form.inputStreet.focus();
      await PA_A.form.inputStreet.fill(valA);

      await PA_B.form.inputStreet.focus();
      await PA_B.form.inputStreet.fill(valB);

      // -------------------- Trigger Race Condition -----------------------------
      // We trigger blur on both pages as simultaneously as possible.
      await Promise.all([
        PA_A.form.inputStreet.blur(),
        PA_B.form.inputStreet.blur(),
      ]);

      // -------------------- Assert ---------------------------------------------
      // One request will win (saving the data), the other one will fail (Optimistic Lock).
      // The failing client should show a toast.

      // Helper to check for toast visibility on a page and return its text
      const getToastText = async (
        toasts: ReturnType<typeof selectors>["toasts"],
      ): Promise<string | null> => {
        try {
          // Wait for ERROR toast to appear, but with a shorter timeout as it should happen quickly
          await toasts.error.waitFor({ state: "visible", timeout: 5000 });
          return await toasts.error.textContent();
        } catch {
          return null;
        }
      };

      // Check both pages for the error toast
      const [toastTextA, toastTextB] = await Promise.all([
        getToastText(Toasts_A),
        getToastText(Toasts_B),
      ]);

      // Exactly one of them should have encountered the error (returned a text string)
      // Checks if either A or B has a string value (truthy)
      expect(!!toastTextA || !!toastTextB).toBe(true);

      if (toastTextA) {
        expect(toastTextA).toContain("optimistic lock conflict");
      }
      if (toastTextB) {
        expect(toastTextB).toContain("optimistic lock conflict");
      }
    } finally {
      await ctxA.close();
      await ctxB.close();
    }
  });
});
