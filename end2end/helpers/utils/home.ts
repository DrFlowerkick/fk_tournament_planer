import { expect, Page } from "@playwright/test";
import { selectors } from "../selectors";
import { extractQueryParamFromUrl, waitForAppHydration } from "./common";

export const HOME_ROUTES = {
  home: "/",
};

/**
 * Open the Home Page without any query parameters.
 */
export async function openHomePage(page: Page) {
  await page.goto(HOME_ROUTES.home);
  // Ensure app is ready/hydrated before interacting
  await waitForAppHydration(page);

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
  pluginName: string,
): Promise<string> {
  // Ensure app is ready/hydrated before interacting
  await waitForAppHydration(page);

  const HOME = selectors(page).home;
  const btn = HOME.sportSelection.pluginButtonByName(pluginName);

  await expect(btn).toBeVisible();

  // Robust interaction: Retry clicking until URL actually changes.
  // This solves issues where Firefox/WebKit might register the click
  // on a button before the event listener is fully attached by WASM.
  await expect(async () => {
    await btn.click();

    // Optimistic check: if URL already has sport_id, we are good.
    // If not, we throw to force a retry of the click.
    const url = page.url();
    if (!url.includes("sport_id=")) {
      throw new Error(
        "Click performed, but URL did not update yet. Retrying...",
      );
    }
  }).toPass({
    timeout: 5000, // Total time to retry
    intervals: [500], // Retry every 500ms
  });

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

/**
 * Verifies that the Sport Dashboard is visible and contains expected links and titles.
 */
export async function expectSportDashboardContent(
  page: Page,
  sportName: string,
) {
  const DASH = selectors(page).home.dashboard;

  // 1. Container visible
  await expect(DASH.root).toBeVisible();

  // 2. Title contains Sport Name (case insensitive check usually safer, but requirement was specific)
  // Expected: "<Sport Name> Tournament Planer"
  await expect(DASH.title).toHaveText(`${sportName} Tournament Planer`, {
    ignoreCase: true,
  });

  // 3. Description visible
  await expect(DASH.description).not.toBeEmpty();

  // 4. Check Navigation Links
  await expect(DASH.nav.tournaments).toBeVisible();
  await expect(DASH.nav.tournaments).toHaveText("Tournaments");

  await expect(DASH.nav.planNew).toBeVisible();
  await expect(DASH.nav.planNew).toHaveText("Plan New Tournament");

  await expect(DASH.nav.adhoc).toBeVisible();
  await expect(DASH.nav.adhoc).toHaveText("Start Adhoc Tournament");

  await expect(DASH.nav.config).toBeVisible();
  await expect(DASH.nav.config).toHaveText("Configurations");

  // About Link should contain Sport Name
  await expect(DASH.nav.about).toBeVisible();
  await expect(DASH.nav.about).toHaveText(`About ${sportName}`);
}

/**
 * Navigates to the "List Tournaments" page for a given sport ID.
 * Assumes the user is already on the sport dashboard.
 */
export async function goToListTournaments(page: Page) {
  await waitForAppHydration(page);
  const DASH = selectors(page).home.dashboard;

  // Navigate via dashboard link
  await expect(DASH.nav.tournaments).toBeVisible();
  await DASH.nav.tournaments.click();

  // Expect List Root to be visible
  await expect(DASH.tournamentsList.root).toBeVisible({ timeout: 10000 });
}

/**
 * Navigates to the "Plan New Tournament" page.
 * Assumes the user is already on the sport dashboard.
 */
export async function goToNewTournament(page: Page) {
  await waitForAppHydration(page);
  const DASH = selectors(page).home.dashboard;

  // Ensure Dashboard is active and Link is clickable
  await expect(DASH.root).toBeVisible();
  await expect(DASH.nav.planNew).toBeVisible();

  // Perform SPA Navigation
  await DASH.nav.planNew.click();

  // Expect URL to update accordingly:
  // start with /new-tournament and ensure no tournament_id param is present
  await page.waitForURL(/^.*\/new-tournament(?!.*tournament_id=).*$/, {
    timeout: 10000,
  });

  // Expect Start Page Root to be visible
  await expect(DASH.editTournament.root).toBeVisible({ timeout: 10000 });
  await expect(DASH.editTournament.form).toBeEnabled();
}
