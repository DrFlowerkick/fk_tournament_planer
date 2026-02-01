// shared utils for end2end tests
import { expect, Page, Locator } from "@playwright/test";
import { DropdownLocators } from "../selectors";

/**
 * Waits strictly until the Leptos/WASM app has signaled hydration complete.
 * AND ensures at least one animation frame has passed to guarantee interactivity.
 */
export async function waitForAppHydration(page: Page) {
  // 1. Wait for the signal from Rust
  await page.waitForSelector('body[data-hydrated="true"]', {
    state: "attached",
    timeout: 10000,
  });

  // 2. Force a wait for the next animation frame.
  // WebKit executes Microtasks (WASM) so aggressively that Playwright might interact
  // with the DOM before the Layout/Paint cycle is fully complete and Event Listeners are bound.
  await page.evaluate(() => {
    return new Promise((resolve) => requestAnimationFrame(resolve));
  });
  await page.waitForLoadState("domcontentloaded");
}

/**
 * Type a value into a field, then blur by focusing another field.
 * Simulates: focus → type → blur -> normalize -> validate for that field.
 */
export async function typeThenBlur(
  inputLocator: Locator,
  value: string,
  blurToLocator: Locator,
) {
  await expect(inputLocator).toBeVisible();
  await inputLocator.fill(value);
  await blurToLocator.focus();
}

/**
 * Fills a value into an input field and immediately calls blur().
 * This ensures on:change events are fired for components relying on blur/change for committing state.
 */
export async function fillAndBlur(locator: Locator, value: string) {
  await expect(locator).toBeVisible();
  await locator.fill(value);
  await locator.blur();
}

/**
 * Select a value in a dropdown, then blur by focusing another field.
 * Necessary because .fill() does not work on <select> elements.
 */
export async function selectThenBlur(
  selectLocator: Locator,
  value: string,
  blurToLocator: Locator,
) {
  await expect(selectLocator).toBeVisible();
  // Playwright specific method for <select>
  await selectLocator.selectOption(value);
  // Focus next element to trigger blur/validation
  await blurToLocator.focus();
}

/**
 * Assert a field's normalized value and validation state using aria-invalid.
 */
export async function expectFieldValidity(
  inputLocator: Locator,
  expectedValue: string,
  isInvalid: boolean,
) {
  await expect(inputLocator).toHaveValue(expectedValue);
  if (isInvalid) {
    await expect(inputLocator).toHaveAttribute("aria-invalid", "true");
  } else {
    const ariaInvalid = await inputLocator.getAttribute("aria-invalid");
    expect(ariaInvalid === null || ariaInvalid === "false").toBeTruthy();
  }
}

/**
 * Extracts a query parameter (e.g., UUID) from any URL.
 * Returns the value as a string or throws an error if not found.
 */
export function extractQueryParamFromUrl(url: string, key: string): string {
  const u = new URL(url, "http://dummy"); // Base URL needed for relative URLs
  const value = u.searchParams.get(key);
  if (!value) throw new Error(`No value for key "${key}" found in URL: ${url}`);
  return value;
}

/**
 * Search on the *current page* and open the unique match.
 * - Does not navigate.
 * - Optionally clears the input before typing.
 * - If your dropdown uses aria-busy, we wait for it to be "false".
 */
export async function searchAndOpenByNameOnCurrentPage(
  dropdown: DropdownLocators,
  term: string,
  opts: {
    clearFirst?: boolean;
    expectUnique?: boolean;
    waitAriaBusy?: boolean;
  } = {},
) {
  const { input, list, items } = dropdown;
  const { clearFirst = true, expectUnique = true, waitAriaBusy = true } = opts;

  await expect(input).toBeVisible();

  // Clear input if requested
  if (clearFirst) {
    await input.fill("");
  }

  // Type the search term
  await input.fill(term);

  // Ensure list is present
  await expect(list).toBeAttached();

  // If your dropdown marks loading via aria-busy, wait until it's finished
  if (waitAriaBusy) {
    await expect(list).toHaveAttribute("aria-busy", "false");
  }

  // Filter rows by visible text
  const row = items.filter({ hasText: term });

  // Option A: enforce uniqueness (assert exactly one)
  if (expectUnique) {
    await expect(row.first()).toBeVisible();
    await expect(row).toHaveCount(1);
    await row.first().click();
    return;
  }

  // Option B: just take the first visible match
  await expect(row.first()).toBeVisible();
  await row.first().click();
}

/**
 * Generates a unique name with a timestamp to avoid DB conflicts during parallel tests.
 */
export function makeUniqueName(base: string): string {
  return `${base} [${Date.now()}-${Math.floor(Math.random() * 1000)}]`;
}
