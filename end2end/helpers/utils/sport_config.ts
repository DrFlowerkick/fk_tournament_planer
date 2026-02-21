// Shared helpers for sport config form flows
import { expect, Page } from "@playwright/test";
import {
  openHomePage,
  selectSportPluginByName,
  waitForAppHydration,
  selectors,
} from "../../helpers";

const UUID_REGEX =
  /^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/;

export const SC_ROUTES = {
  newConfig: "/sport-configurations/new",
  editConfig: "/sport-configurations/edit",
  copyConfig: "/sport-configurations/copy",
  list: "/sport-configurations",
};

export const SC_QUERY_KEYS = {
  sportId: "sport_id",
  sportConfigId: "sport_config_id",
};

/**
 * Open the "Sport Selection and Config List".
 */
export async function openSportConfigurationList(
  page: Page,
  sport_name: string,
) {
  // 1. Navigation
  await openHomePage(page);

  // strict hydration check
  await waitForAppHydration(page);

  const sportId = await selectSportPluginByName(page, sport_name);

  const DASH = selectors(page).home.dashboard;
  // Ensure Dashboard is active and Link is clickable
  await expect(DASH.root).toBeVisible();
  await expect(DASH.nav.config).toBeVisible();

  // Perform SPA Navigation
  await DASH.nav.config.click();

  const SC = selectors(page).sportConfig;
  await expect(SC.list.filterName).toBeVisible();
}

/**
 * Click the first row in sport configuration list to open a detail page.
 * Returns the ID of the opened config (extracted from the row's data-testid).
 */
export async function clickFirstRowAndExtractId(page: Page) {
  const SC = selectors(page).sportConfig;
  const firstRow = SC.list.anyRow;
  await expect(firstRow).toBeVisible();
  await firstRow.click();
  // Extract ID from url query param like ?sport_config_id=UUID
  const url = page.url();
  const id = new URL(url).searchParams.get(SC_QUERY_KEYS.sportConfigId);
  if (!id) {
    throw new Error("Could not extract sport_config_id from URL");
  }
  return id;
}

/**
 * Enter new mode from a detail page (if you have a dedicated edit button).
 */
export async function clickNewSportConfig(page: Page) {
  const SC = selectors(page).sportConfig;
  await expect(SC.list.btnNew).toBeVisible();
  await SC.list.btnNew.click();
  // Assert the form is shown again
  await waitForSportConfigNewUrl(page);
}

/**
 * Enter edit mode from a detail page.
 * For this to work, a row has to be clicked first to show the edit button (as in the current UI).
 */
export async function clickEditSportConfig(page: Page) {
  const SC = selectors(page).sportConfig;
  await expect(SC.list.btnEdit).toBeVisible();
  await SC.list.btnEdit.click();
  // Assert the form is shown again
  await waitForSportConfigEditUrl(page);
}

/**
 * Enter edit mode from a detail page.
 * For this to work, a row has to be clicked first to show the edit button (as in the current UI).
 */
export async function clickCopySportConfig(page: Page) {
  const SC = selectors(page).sportConfig;
  await expect(SC.list.btnCopy).toBeVisible();
  await SC.list.btnCopy.click();
  // Assert the form is shown again
  await waitForSportConfigCopyUrl(page);
}

/**
 * Wait for navigation to a edit sport configuration page (UUID URL).
 */
export async function waitForSportConfigNewUrl(page: Page) {
  const SC = selectors(page).sportConfig;
  // Wait for URL path /sport-configurations/new and valid sport_id query param
  await page.waitForURL((url) => {
    const isCorrectPath = url.pathname === SC_ROUTES.newConfig;
    const sportId = url.searchParams.get(SC_QUERY_KEYS.sportId);
    return isCorrectPath && !!sportId && UUID_REGEX.test(sportId);
  });

  // strict hydration check
  await waitForAppHydration(page);

  await expect(SC.form.root).toBeVisible();
}

/**
 * Wait for navigation to a edit sport configuration page (UUID URL).
 */
export async function waitForSportConfigEditUrl(page: Page) {
  const SC = selectors(page).sportConfig;
  // Wait for URL path /sport-configurations/edit and valid sport_config_id query param
  await page.waitForURL((url) => {
    const isCorrectPath = url.pathname === SC_ROUTES.editConfig;
    const sportId = url.searchParams.get(SC_QUERY_KEYS.sportId);
    const sportConfigId = url.searchParams.get(SC_QUERY_KEYS.sportConfigId);
    return (
      isCorrectPath &&
      !!sportId &&
      UUID_REGEX.test(sportId) &&
      !!sportConfigId &&
      UUID_REGEX.test(sportConfigId)
    );
  });

  // strict hydration check
  await waitForAppHydration(page);

  await expect(SC.form.root).toBeVisible();
}

/**
 * Wait for navigation to a copy sport configuration page (UUID URL).
 */
export async function waitForSportConfigCopyUrl(page: Page) {
  const SC = selectors(page).sportConfig;
  // Wait for URL path /sport-configurations/copy and valid sport_config_id query param
  await page.waitForURL((url) => {
    const isCorrectPath = url.pathname === SC_ROUTES.copyConfig;
    const sportId = url.searchParams.get(SC_QUERY_KEYS.sportId);
    const sportConfigId = url.searchParams.get(SC_QUERY_KEYS.sportConfigId);
    return (
      isCorrectPath &&
      !!sportId &&
      UUID_REGEX.test(sportId) &&
      !!sportConfigId &&
      UUID_REGEX.test(sportConfigId)
    );
  });

  // strict hydration check
  await waitForAppHydration(page);

  await expect(SC.form.root).toBeVisible();
}
