import { expect, Page } from "@playwright/test";
import { selectors } from "./selectors";
import { extractQueryParamFromUrl } from "./utils";

export const ROUTES = {
  home: "/",
};

/**
 * Open the Home Page without any query parameters.
 */
export async function openHomePage(page: Page) {
  await page.goto(ROUTES.home);
  await page.waitForLoadState("domcontentloaded");

  const HOME = selectors(page).home;

  // Assert Hero section is visible
  await expect(HOME.hero.root).toBeVisible();

  // Assert Sport Selection Grid is visible initially (no sport selected)
  await expect(HOME.sportSelection.grid).toBeVisible();
}

/**
 * Select a specific sport plugin by its visible NAME.
 * Expects the grid to disappear and the URL to update.
 * Returns the detected sport_id from the URL.
 */
export async function selectSportPluginByName(
  page: Page,
  pluginName: string
): Promise<string> {
  const HOME = selectors(page).home;
  const btn = HOME.sportSelection.pluginButtonByName(pluginName);

  await expect(btn).toBeVisible();
  await btn.click();

  // Wait for ANY sport_id param to appear in URL
  await page.waitForURL(/sport_id=([0-9a-f-]{36})/);

  // Extract and validate ID
  const sportId = extractQueryParamFromUrl(page.url(), "sport_id");
  expect(sportId).toMatch(/^[0-9a-f-]{36}$/); // Assert valid UUID format

  // According to requirements: selection grid should not be visible anymore
  await expect(HOME.sportSelection.grid).not.toBeVisible();

  if (!sportId) throw new Error("No sport_id found in URL");
  return sportId;
}

/**
 * Ensure the View for a specific sport is active (based on URL and absence of grid).
 * (Details for the Sport View content to be added later)
 */
export async function expectSportViewActive(page: Page, pluginId: string) {
  const HOME = selectors(page).home;
  const url = new URL(page.url());
  expect(url.searchParams.get("sport_id")).toBe(pluginId);

  // The selection grid must be gone
  await expect(HOME.sportSelection.grid).toBeHidden();
}
