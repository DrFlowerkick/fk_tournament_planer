// e2e/tests/keyboard-navigation-search.spec.ts
import { test, expect } from "@playwright/test";
import { openPostalAddressList } from "../helpers/postal_address";
import { T } from "../helpers/selectors";

test.describe("Search list keyboard navigation", () => {
  // We need to create three entries first to have something to navigate
  // We do this globally for all tests in this suite in ./end2end/global-setup.ts

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
