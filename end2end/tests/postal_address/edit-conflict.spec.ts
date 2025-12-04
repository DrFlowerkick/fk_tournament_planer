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
} from "../../helpers/postal_address";
import { selectors } from "../../helpers/selectors";
import { typeThenBlur } from "../../helpers/utils";

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
      await expect(pageA.locator('input[name="version"]')).toHaveValue("0");

      // -------------------- B updates first -----------------------
      await openEditForm(pageB, id);
      // The version is in a hidden input field. We check its value attribute.
      await expect(pageB.locator('input[name="version"]')).toHaveValue("0");

      const editedByB = `${initial.name} (B)`;
      await typeThenBlur(PA_B.form.inputName, editedByB, PA_B.form.inputStreet);
      await clickSave(pageB); // server -> version 1

      // -------------------- A edits stale & tries to save ---------
      const editedByA = `${initial.name} (A)`;
      await typeThenBlur(PA_A.form.inputName, editedByA, PA_A.form.inputStreet);
      await PA_A.form.btnSave.click(); // expect 409 and conflict UI

      // -------------------- Assert minimal conflict UI ------------
      // A banner should appear, and the reload button should be visible.
      await expect(BA_A.acknowledgment.root).toBeVisible();
      await expect(BA_A.acknowledgment.btnAction).toBeVisible();

      // The banner should contain a warning message.
      await expect(BA_A.acknowledgment.root).toContainText(
        "A newer version of this address exists. Reloading will discard your changes."
      );

      // Save must be disabled while the conflict is unresolved.
      await expect(PA_A.form.btnSave).toBeDisabled();

      // -------------------- Resolve via reload --------------------
      await BA_A.acknowledgment.btnAction.click();

      // After reload, the banner should be gone and the form-version should bump to "1".
      await expect(
        BA_A.acknowledgment.root
      ).toBeHidden();
      await expect(PA_A.form.hiddenVersion).toHaveValue("1");

      // The name input should now reflect B's saved value.
      await expect(PA_A.form.inputName).toHaveValue(editedByB);

      // Save becomes enabled again.
      await expect(PA_A.form.btnSave).toBeEnabled();
    } finally {
      await ctxA.close();
      await ctxB.close();
    }
  });
});
