import { test, expect } from "@playwright/test";
import {
  openSportConfigurationList,
  clickNewSportConfig,
  searchAndOpenByNameOnCurrentPage,
  extractQueryParamFromUrl,
  selectors
} from "../../helpers";

test("Smoke: Select Plugin -> Search Config -> New -> Cancel", async ({
  page,
}) => {
  const SC = selectors(page).sportConfig;

  // 1. Open Sport Config List
  await openSportConfigurationList(page, "Generic Sport");

  const sport_id = extractQueryParamFromUrl(page.url(), "sport_id");
  expect(sport_id).toBeTruthy();
  expect(sport_id).toMatch(/^[0-9a-fA-F-]{36}$/);

  // 3. Verify new configuration button is visible
  await expect(SC.list.btnNew).toBeVisible();

  // 4. Navigate to New Form
  await clickNewSportConfig(page);

  // 5. Verify Sport-Specific Form is rendered
  await expect(SC.form.inputName).toBeVisible();

  // 6. Cancel back to search context
  await SC.form.btnCancel.click();

  // 7. Verify we are back at the list and sport_id is preserved
  const { pathname } = new URL(page.url());
  expect(pathname.startsWith("/sport-configurations")).toBeTruthy();
  expect(pathname).not.toContain("/new");
  expect(page.url()).toContain("sport_id=");
  expect(pathname).not.toContain("sport_config_id=");
});
