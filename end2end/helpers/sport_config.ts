// Shared helpers for sport config form flows
import { expect, Page } from "@playwright/test";
import { selectors } from "./selectors";
import {
  typeThenBlur,
  selectThenBlur,
  extractQueryParamFromUrl,
  waitForAppHydration,
} from "./utils";

export const ROUTES = {
  newAddress: "/sport/new_sc",
  list: "/sport",
};

/**
 * Open the "Sport Selection and Config List".
 */
export async function openSportSelectionAndConfigList(page: Page) {
  const SC = selectors(page).sportConfig;
  // Navigate to "list" route and assert elements exist
  await page.goto(ROUTES.list);

  // REPLACED: domcontentloaded -> strict hydration check
  await waitForAppHydration(page);

  await expect(SC.pluginSelector.input).toBeVisible();
}

/**
 * Enter new mode from a detail page (if you have a dedicated edit button).
 */
export async function clickNewToOpenEditForm(page: Page) {
  const SC = selectors(page).sportConfig;
  await expect(SC.search.btnNew).toBeVisible();
  await SC.search.btnNew.click();
  // Assert the form is shown again
  await waitForSportConfigNewUrl(page);
}

/**
 * Enter edit mode from a detail page.
 */
export async function clickEditToOpenEditForm(page: Page) {
  const SC = selectors(page).sportConfig;
  await expect(SC.search.btnEdit).toBeVisible();
  await SC.search.btnEdit.click();
  // Assert the form is shown again
  await waitForSportConfigEditUrl(page);
}

/**
 * Wait for navigation to a edit postal address page (UUID URL).
 */
export async function waitForSportConfigNewUrl(page: Page) {
  const SC = selectors(page).sportConfig;
  // Wait for URL like /sport/new_sc?sport_id=UUID
  await page.waitForURL(/\/sport\/new_sc\?sport_id=[0-9a-f-]{36}$/);

  // REPLACED: domcontentloaded -> strict hydration check
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
  // Wait for URL like /sport/edit_sc?sport_id=UUID&sport_config_id=UUID
  await page.waitForURL(
    /\/sport\/edit_sc\?sport_id=[0-9a-f-]{36}&sport_config_id=[0-9a-f-]{36}$/
  );

  // REPLACED: domcontentloaded -> strict hydration check
  await waitForAppHydration(page);

  await expect(SC.form.root).toBeVisible();
  await expect(SC.form.btnSave).toBeVisible();
  await expect(SC.form.btnSaveAsNew).toBeVisible();
}
