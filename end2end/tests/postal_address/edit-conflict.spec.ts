// e2e/edit-conflict.spec.ts
import { test, expect } from "@playwright/test";
import {
  openNewForm,
  fillFields,
  clickSave,
  extractUuidFromUrl,
  expectSavesDisabled,
  openEditForm,
  waitForPostalAddressListUrl,
  fillAndBlur,
  selectors,
} from "../../helpers";

test.describe("Edit conflict shows proper fallback reaction", () => {
  test("A on stale version gets conflict banner and disabled save", async ({
    browser,
  }) => {
    const ctxA = await browser.newContext();
    const ctxB = await browser.newContext();
    const pageA = await ctxA.newPage();
    const pageB = await ctxB.newPage();
    const PA_A = selectors(pageA).postalAddress;
    const BA_A = selectors(pageA).banners;
    const PA_B = selectors(pageB).postalAddress;

    try {
      // -------------------- Arrange: A creates --------------------
      const initial = {
        name: `E2E Conflict ${Date.now()}`,
        street: "Parallelweg 1",
        postal_code: "12345",
        locality: "Zweigleisig",
        region: "BE",
        country: "DE",
      };

      await openNewForm(pageA);
      // as long as other required fields are empty/invalid, saving must remain disabled
      await expectSavesDisabled(pageA);

      await fillFields(pageA, initial);
      await clickSave(pageA);

      await waitForPostalAddressListUrl(pageA);
      const id = extractUuidFromUrl(pageA.url());

      // A opens edit for this id. Expect form-version "0".
      await openEditForm(pageA, id);
      // The version is in a hidden input field. We check its value attribute.
      await expect(PA_A.form.hiddenVersion).toHaveValue("0");

      // -------------------- B updates first -----------------------
      await openEditForm(pageB, id);
      // The version is in a hidden input field. We check its value attribute.
      await expect(PA_B.form.hiddenVersion).toHaveValue("0");

      const editedByB = `${initial.name} (B)`;
      await fillAndBlur(PA_B.form.inputName, editedByB);
      await clickSave(pageB); // server -> version 1

      // -------------------- A edits stale & tries to save ---------
      const editedByA = `${initial.name} (A)`;
      await fillAndBlur(PA_A.form.inputName, editedByA);
      await PA_A.form.btnSave.click(); // expect 409 and conflict UI

      // -------------------- Assert minimal conflict UI ------------
      // A banner should appear, and the reload button should be visible.
      await expect(BA_A.globalErrorBanner.root).toBeVisible();
      await expect(BA_A.globalErrorBanner.btnRetry).toBeVisible();
      await expect(BA_A.globalErrorBanner.btnCancel).toBeVisible();

      // The banner should contain a warning message.
      await expect(BA_A.globalErrorBanner.root).toContainText(
        "The record has been modified in the meantime. Please reload. Any unsaved changes will be lost.",
      );

      // Verify that the main content is inert
      await expect(pageA.locator("main")).toHaveAttribute("inert");

      // -------------------- Cancel should keep stale data and remove banner -----------
      await BA_A.globalErrorBanner.btnCancel.click();

      // After cancel, the banner should be gone and the form-version should still be "0".
      await expect(BA_A.globalErrorBanner.root).toBeHidden();
      await expect(PA_A.form.hiddenVersion).toHaveValue("0");

      // The name input should still reflect A's unsaved value.
      await expect(PA_A.form.inputName).toHaveValue(editedByA);

      // -------------------- Try to save again, then reload to resolve -----------
      await PA_A.form.btnSave.click(); // expect conflict UI again
      // A banner should appear, and the reload button should be visible.
      await expect(BA_A.globalErrorBanner.root).toBeVisible();
      await expect(BA_A.globalErrorBanner.btnRetry).toBeVisible();
      await expect(BA_A.globalErrorBanner.btnCancel).toBeVisible();

      // -------------------- Resolve via reload --------------------
      await BA_A.globalErrorBanner.btnRetry.click();

      // After reload, the banner should be gone and the form-version should bump to "1".
      await expect(BA_A.globalErrorBanner.root).toBeHidden();
      await expect(PA_A.form.hiddenVersion).toHaveValue("1");

      // The name input should now reflect B's saved value.
      await expect(PA_A.form.inputName).toHaveValue(editedByB);

      // Verify that the main content is not inert anymore
      await expect(pageA.locator("main")).not.toHaveAttribute("inert");
    } finally {
      await ctxA.close();
      await ctxB.close();
    }
  });
});
