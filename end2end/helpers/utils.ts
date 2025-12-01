// shared utils for end2end tests
import { expect, Page } from "@playwright/test";

/**
 * Type a value into a field, then blur by focusing another field.
 * Simulates: focus → type → blur -> normalize -> validate for that field.
 */
export async function typeThenBlur(
  page: Page,
  inputTid: string,
  value: string,
  blurToTid: string
) {
  await expect(page.getByTestId(inputTid)).toBeVisible();
  await page.getByTestId(inputTid).fill(value);
  await page.getByTestId(blurToTid).focus();
}

/**
 * Select a value in a dropdown, then blur by focusing another field.
 * Necessary because .fill() does not work on <select> elements.
 */
export async function selectThenBlur(
  page: Page,
  selectTid: string,
  value: string,
  blurToTid: string
) {
  await expect(page.getByTestId(selectTid)).toBeVisible();
  // Playwright specific method for <select>
  await page.getByTestId(selectTid).selectOption(value);
  // Focus next element to trigger blur/validation
  await page.getByTestId(blurToTid).focus();
}

/**
 * Assert a field's normalized value and validation state using aria-invalid.
 */
export async function expectFieldValidity(
  page: Page,
  inputTid: string,
  expectedValue: string,
  isInvalid: boolean
) {
  const input = page.getByTestId(inputTid);
  await expect(input).toHaveValue(expectedValue);
  if (isInvalid) {
    await expect(input).toHaveAttribute("aria-invalid", "true");
  } else {
    const ariaInvalid = await input.getAttribute("aria-invalid");
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