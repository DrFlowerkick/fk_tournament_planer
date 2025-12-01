import { test, expect } from "@playwright/test";
import { selectors } from "../../helpers/selectors";
import {
  openPostalAddressList,
  openNewForm,
} from "../../helpers/postal_address";

test("Smoke: Search → New → Cancel", async ({ page }) => {
  const S = selectors(page);

  await test.step("Open search page", async () => {
    await openPostalAddressList(page);
  });

  await test.step("Navigate to New form", async () => {
    await openNewForm(page);
  });

  await test.step("Cancel back to search/detail context", async () => {
    await S.form.btnCancel.click();
    // Accept either /postal-address or /postal-address?address_id=UUID URL
    await expect(S.search.input).toBeVisible();
    const { pathname } = new URL(page.url());
    expect(pathname.startsWith("/postal-address")).toBeTruthy();
  });
});
