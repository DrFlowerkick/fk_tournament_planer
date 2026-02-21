import { test, expect } from "@playwright/test";
import { openPostalAddressList, clickNewPostalAddress, selectors } from "../../helpers";

test("Smoke: Search → New → Close", async ({ page }) => {
  const PA = selectors(page).postalAddress;

  await test.step("Open search page", async () => {
    await openPostalAddressList(page);
  });

  await test.step("Navigate to New form", async () => {
    await clickNewPostalAddress(page);
  });

  await test.step("Close form and return back to search/detail context", async () => {
    await PA.form.btnClose.click();
    // Accept either /postal-address or /postal-address?address_id=UUID URL
    const { pathname } = new URL(page.url());
    expect(pathname.startsWith("/postal-address")).toBeTruthy();
    expect(pathname).not.toContain("/new");
    expect(pathname).not.toContain("address_id=");
  });
});
