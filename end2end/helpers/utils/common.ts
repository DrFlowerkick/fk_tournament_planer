// shared utils for end2end tests
import { expect, Page, Locator } from "@playwright/test";
import { selectors, IDS } from "../selectors";

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
 * Helper to filter a list by name and select the row with the given name.
 * Assumes you're already on a page with a list using standard LIST_IDS.
 * Returns the locator for the name cell of the entry.
 */
export async function searchAndOpenByNameOnCurrentPage(
  page: Page,
  name: string,
  queryParamKey: string,
): Promise<Locator> {
  const list = selectors(page).list;
  await expect(list.filterName).toBeVisible();
  await list.filterName.fill(name);

  // Find the interactive entry by name
  const entry = list.previewByName(name);
  await expect(entry).toBeVisible();

  // 1. Extract UUID from the test-id
  const testId = await entry.getAttribute("data-testid");
  const idFromName = testId?.replace(IDS.list.entryPreviewPrefix, "");

  // 2. Extract selected UUID from URL
  const idFromUrl = extractQueryParamFromUrl(page.url(), queryParamKey);

  // 3. Click if not already selected
  if (!idFromUrl || idFromName !== idFromUrl) {
    await entry.click();
  }

  // Ensure navigation/UI update triggered the edit button visibility
  await expect(list.btnEdit).toBeVisible();
  return entry;
}

export async function waitForNavigationRowSelectionByName(
  page: Page,
  name: string,
  queryParamKey: string,
) {
  const LIST = selectors(page).list;

  // After save the row of "name" should be selected, when the url contains
  // the corresponding ID of the created object and the detailed preview should be visible.
  const preview = LIST.previewByName(name);
  
  // Extract the specific UUID from the data-testid before clicking
  const testId = await preview.getAttribute("data-testid");
  const expectedId = testId?.replace(IDS.list.entryPreviewPrefix, "");

  // Wait for the URL to contain exactly the ID of the row we just clicked.
  // This ensures Leptos has processed the correct navigation.
  if (expectedId) {
    await page.waitForURL(
      (url) => url.searchParams.get(queryParamKey) === expectedId,
    );
  }
}

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
 * Generates a unique name with a timestamp to avoid DB conflicts during parallel tests.
 */
export function makeUniqueName(base: string): string {
  return `${base} [${Date.now()}-${Math.floor(Math.random() * 1000)}]`;
}
