// e2e/tests/keyboard-navigation-search.spec.ts
import { test, expect } from "@playwright/test";
import { openPostalAddressList, openNewForm, fillAll, clickSave, waitForPostalAddressListLoadedWithUrl } from "../helpers/form";
import { T } from "../helpers/selectors";

test.describe("Search list keyboard navigation", () => {
  // We need to create some data first to have something to navigate
  test.beforeAll(async ({ browser }) => {
    const page = await (await browser.newContext()).newPage();
    await openNewForm(page);
    const names = ["Alpha", "Beta", "Gamma"];
    for (const name of names) {
      await fillAll(page, `E2E Nav ${name} ${Date.now()}`, "Teststr. 1", "12345", "Teststadt", "", "DE");
      await clickSave(page);
      await waitForPostalAddressListLoadedWithUrl(page);
      await openNewForm(page);
    }
    await page.close();
  });

  test("navigates with arrow keys, selects with Enter, closes with Escape", async ({
    page,
  }) => {
    await openPostalAddressList(page);
    const input = page.getByTestId(T.search.input);
    const list = page.getByTestId(T.search.suggestList);

    // Type to get results
    await input.fill("E2E Nav");
    await expect(list).toBeVisible();
    const items = list.locator("li");
    await expect(items).toHaveCount(3);

    // --- ArrowDown navigation ---
    await input.press("ArrowDown"); // -> Alpha
    await expect(items.nth(0)).toHaveClass(/active/);
    await expect(items.nth(1)).not.toHaveClass(/active/);

    await input.press("ArrowDown"); // -> Beta
    await expect(items.nth(0)).not.toHaveClass(/active/);
    await expect(items.nth(1)).toHaveClass(/active/);

    await input.press("ArrowDown"); // -> Gamma
    await expect(items.nth(2)).toHaveClass(/active/);

    await input.press("ArrowDown"); // wrap to Alpha
    await expect(items.nth(0)).toHaveClass(/active/);

    // --- ArrowUp navigation ---
    await input.press("ArrowUp"); // wrap to Gamma
    await expect(items.nth(2)).toHaveClass(/active/);

    await input.press("ArrowUp"); // -> Beta
    await expect(items.nth(1)).toHaveClass(/active/);

    // --- Select with Enter ---
    await input.press("Enter");
    await expect(list).toBeHidden();
    await expect(page.getByTestId(T.search.preview.name)).toHaveText(
      /^E2E Nav Beta/
    );

    // --- Re-open and close with Escape ---
    await input.fill("E2E Nav");
    await expect(list).toBeVisible();
    await input.press("Escape");
    await expect(list).toBeHidden();
  });
});
