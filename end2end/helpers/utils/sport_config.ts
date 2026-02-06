// Shared helpers for sport config form flows
import { expect, Page } from "@playwright/test";
import {
  openHomePage,
  selectSportPluginByName,
  waitForAppHydration,
  selectors,
} from "../../helpers";

export const SC_ROUTES = {
  newConfig: "/sport-configurations/new",
  list: "/sport-configurations",
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
  // Wait for URL like /sport-configurations?sport_id=UUID&sport_config_id=UUID
  await page.waitForURL(
    /\/sport-configurations\?sport_id=[0-9a-f-]{36}&sport_config_id=[0-9a-f-]{36}$/,
  );
  // Extract ID from url query param like ?sport_config_id=UUID
  const url = page.url();
  const id = new URL(url).searchParams.get("sport_config_id");
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
 * Wait for navigation to a edit postal address page (UUID URL).
 */
export async function waitForSportConfigNewUrl(page: Page) {
  const SC = selectors(page).sportConfig;
  // Wait for URL like /sport-configurations/new?sport_id=UUID
  await page.waitForURL(/\/sport-configurations\/new\?sport_id=[0-9a-f-]{36}$/);

  // strict hydration check
  await waitForAppHydration(page);

  await expect(SC.form.root).toBeVisible();
  await expect(SC.form.btnSave).toBeVisible();
  await expect(SC.form.btnSaveAsNew).not.toBeVisible();
}

/**
 * Wait for navigation to a edit postal address page (UUID URL).
 */
export async function waitForSportConfigEditUrl(page: Page) {
  const SC = selectors(page).sportConfig;
  // Wait for URL like /sport-configurations/edit?sport_id=UUID&sport_config_id=UUID
  await page.waitForURL(
    /\/sport-configurations\/edit\?sport_id=[0-9a-f-]{36}&sport_config_id=[0-9a-f-]{36}$/,
  );

  // strict hydration check
  await waitForAppHydration(page);

  await expect(SC.form.root).toBeVisible();
  await expect(SC.form.btnSave).toBeVisible();
  await expect(SC.form.btnSaveAsNew).toBeVisible();
}
