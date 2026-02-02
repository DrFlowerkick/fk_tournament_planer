import { test, expect } from "@playwright/test";
import { openPostalAddressList, openNewForm, selectors } from "../../helpers";

test("Smoke: Search → New → Cancel", async ({ page }) => {
  const PA = selectors(page).postalAddress;

  await test.step("Open search page", async () => {
    await openPostalAddressList(page);
  });

  await test.step("Navigate to New form", async () => {
    await openNewForm(page);
  });

  await test.step("Cancel back to search/detail context", async () => {
    await PA.form.btnCancel.click();
    // Accept either /postal-address or /postal-address?address_id=UUID URL
    const { pathname } = new URL(page.url());
    expect(pathname.startsWith("/postal-address")).toBeTruthy();
    expect(pathname).not.toContain("/new_pa");
    expect(pathname).not.toContain("address_id=");
  });
});
