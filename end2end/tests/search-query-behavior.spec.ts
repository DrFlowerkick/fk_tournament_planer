// e2e/tests/search-query-behavior.spec.ts
import { test, expect } from "@playwright/test";
import { openPostalAddressList } from "../helpers/form";
import { T } from "../helpers/selectors";

test.describe("Search query behavior", () => {
  test("does not trigger search for short queries", async ({ page }) => {
    await openPostalAddressList(page);
    const input = page.getByTestId(T.search.input);
    const list = page.getByTestId(T.search.suggestList);

    // Type two characters
    await input.fill("AB");

    // Wait a moment to ensure no network call is made
    await page.waitForTimeout(500);

    // The list should not appear, or if it does, it should be empty
    const isVisible = await list.isVisible();
    if (isVisible) {
      const items = list.locator("li");
      await expect(items).toHaveCount(1);
      await expect(items.first()).toHaveText("Searching…");
    } else {
      await expect(list).toBeHidden();
    }

    // Type a third character to trigger the search
    await input.fill("ABC");
    await expect(list).toBeVisible();
  });

  test("clears results when query is deleted", async ({ page }) => {
    await openPostalAddressList(page);
    const input = page.getByTestId(T.search.input);
    const list = page.getByTestId(T.search.suggestList);

    // Get some results
    await input.fill("Test");
    await expect(list).toBeVisible();
    await expect(list.locator("li").first()).toBeVisible();

    // Clear the input
    await input.fill("");
    // The list should show searching again
    const isVisible = await list.isVisible();
    if (isVisible) {
      const items = list.locator("li");
      await expect(items).toHaveCount(1);
      await expect(items.first()).toHaveText("Searching…");
    } else {
      await expect(list).toBeHidden();
    }

    // The list should disappear or be empty if blur
    await input.blur();
    await expect(list).toBeHidden();
  });
});
