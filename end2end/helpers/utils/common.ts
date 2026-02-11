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
 * Fills a value into an input field and immediately calls blur().
 * This ensures on:change events are fired for components relying on blur/change for committing state.
 */
export async function fillAndBlur(locator: Locator, value: string) {
  await expect(locator).toBeVisible();
  await locator.fill(value);
  await locator.blur();
}

/**
 * Select a value in a dropdown, then blur.
 * Necessary because .fill() does not work on <select> elements.
 */
export async function selectThenBlur(selectLocator: Locator, value: string) {
  await expect(selectLocator).toBeVisible();
  await selectLocator.selectOption(value);
  await selectLocator.blur();
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
 * Returns the value as a string or null if not found.
 */
export function extractQueryParamFromUrl(
  url: string,
  key: string,
): string | null {
  const u = new URL(url, "http://dummy"); // Base URL needed for relative URLs
  return u.searchParams.get(key);
}

/**
 * Generates a unique name with a timestamp to avoid DB conflicts during parallel tests.
 */
export function makeUniqueName(base: string): string {
  return `${base} [${Date.now()}-${Math.floor(Math.random() * 1000)}]`;
}
