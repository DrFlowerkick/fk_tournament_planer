// Shared helpers for sport config form flows
import { expect, Page } from "@playwright/test";
import { T } from "./selectors";
import {
  typeThenBlur,
  selectThenBlur,
  extractQueryParamFromUrl,
} from "./utils";

export const ROUTES = {
  newAddress: "/sport/new_pa",
  list: "/sport",
};

/**
 * Open the "Sport Selection and Config List".
 */
export async function openSportSelectionAndConfigList(page: Page) {
  // Navigate to "list" route and assert elements exist
  await page.goto(ROUTES.list);
  await page.waitForLoadState("domcontentloaded");
  await expect(page.getByTestId(T.sportSelector.input)).toBeVisible();
}