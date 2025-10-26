// e2e/edit-conflict.spec.ts
import { test, expect } from "@playwright/test";
import {
  openNewForm,
  fillAll,
  clickSave,
  typeThenBlur,
  extractUuidFromUrl,
} from "../helpers/form";
import { T } from "../helpers/selectors";

test.describe("Edit conflict shows proper fallback reaction", () => {
  test("A on stale version gets conflict banner and disabled save", async ({
    browser,
  }) => {
    const ctxA = await browser.newContext();
    const ctxB = await browser.newContext();
    const pageA = await ctxA.newPage();
    const pageB = await ctxB.newPage();

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
      await fillAll(
        pageA,
        initial.name,
        initial.street,
        initial.postal_code,
        initial.locality,
        initial.region,
        initial.country
      );
      await clickSave(pageA);

      await pageA.waitForURL(/\/postal-address\/[0-9a-f-]{36}$/);
      const id = extractUuidFromUrl(pageA.url());

      // A opens edit for this id. Expect form-version "0".
      await pageA.goto(`/postal-address/${id}/edit`);
      // The version is in a hidden input field. We check its value attribute.
      await expect(pageA.locator('input[name="version"]')).toHaveValue("0");

      // -------------------- B updates first -----------------------
      await pageB.goto(`/postal-address/${id}/edit`);
      const editedByB = `${initial.name} (B)`;
      await typeThenBlur(
        pageB,
        T.form.inputName,
        editedByB,
        T.form.inputStreet
      );
      await clickSave(pageB); // server -> version 1

      // -------------------- A edits stale & tries to save ---------
      const editedByA = `${initial.name} (A)`;
      await typeThenBlur(
        pageA,
        T.form.inputName,
        editedByA,
        T.form.inputStreet
      );
      await pageA.getByTestId(T.form.btnSave).click(); // expect 409 and conflict UI

      // -------------------- Assert minimal conflict UI ------------
      // A banner should appear, and the reload button should be visible.
      await expect(pageA.getByTestId(T.banner.acknowledgmentBanner)).toBeVisible();
      await expect(pageA.getByTestId(T.banner.btnAcknowledgment)).toBeVisible();

      // The banner should contain a warning message.
      await expect(pageA.getByTestId(T.banner.acknowledgmentBanner)).toContainText(
        "A newer version of this address exists. Reloading will discard your changes."
      );

      // Save must be disabled while the conflict is unresolved.
      await expect(pageA.getByTestId(T.form.btnSave)).toBeDisabled();

      // -------------------- Resolve via reload --------------------
      await pageA.getByTestId(T.banner.btnAcknowledgment).click();

      // After reload, the banner should be gone and the form-version should bump to "1".
      await expect(pageA.getByTestId(T.banner.acknowledgmentBanner)).toBeHidden();
      await expect(pageA.getByTestId(T.form.hiddenVersion)).toHaveValue("1");

      // The name input should now reflect B's saved value.
      await expect(pageA.getByTestId(T.form.inputName)).toHaveValue(editedByB);

      // Save becomes enabled again.
      await expect(pageA.getByTestId(T.form.btnSave)).toBeEnabled();
    } finally {
      await ctxA.close();
      await ctxB.close();
    }
  });
});
