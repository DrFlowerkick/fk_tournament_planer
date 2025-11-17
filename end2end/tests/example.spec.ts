import { test, expect } from "@playwright/test";

test("homepage has title and links to intro page", async ({ page }) => {
  await page.goto("http://localhost:3000/");

  await expect(page).toHaveTitle("FK Tournament Planer");

  await expect(page.locator("h1")).toHaveText("Welcome!");

  await expect(page.getByRole('button')).toHaveText("Click me: 0");

  await page.click('button');

  await expect(page.getByRole('button')).toHaveText("Click me: 1");
});
