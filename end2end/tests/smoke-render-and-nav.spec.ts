import { test, expect } from "@playwright/test";
import { selectors } from "../helpers/selectors";

test("Smoke: Search → New → Cancel", async ({ page }) => {
  const S = selectors(page);

  await test.step("Open search page", async () => {
    await page.goto("/postal-address"); // relies on baseURL in playwright.config.ts
    await expect(S.search.input).toBeVisible();
  });

  await test.step("Navigate to New form", async () => {
    await Promise.all([
      page.waitForURL("**/postal-address/new"),
      S.search.btnNew.click(),
    ]);
    await expect(S.form.root).toBeVisible();
  });

  await test.step("Cancel back to search/detail context", async () => {
    await S.form.btnCancel.click();
    // Accept either /postal-address or /postal-address/{id}
    await expect(S.search.input).toBeVisible();
    const { pathname } = new URL(page.url());
    expect(pathname.startsWith("/postal-address")).toBeTruthy();
  });
});
