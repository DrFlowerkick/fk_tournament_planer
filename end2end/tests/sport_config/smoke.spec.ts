import { test, expect } from "@playwright/test";
import { selectors } from "../../helpers/selectors";
import {
  openSportSelectionAndConfigList,
  clickNewToOpenEditForm,
} from "../../helpers/sport_config";
import {
  searchAndOpenByNameOnCurrentPage,
  extractQueryParamFromUrl,
} from "../../helpers/utils";

test("Smoke: Select Plugin -> Search Config -> New -> Cancel", async ({
  page,
}) => {
  const SC = selectors(page).sportConfig;

  // 1. Open Sport Config List
  await openSportSelectionAndConfigList(page);

  // 2. Select a Sport Plugin (e.g., "Generic Sport")
  await searchAndOpenByNameOnCurrentPage(SC.pluginSelector, "Generic Sport");
  const sport_id = extractQueryParamFromUrl(page.url(), "sport_id");
  expect(sport_id).toBeTruthy();
  expect(sport_id).toMatch(/^[0-9a-fA-F-]{36}$/);

  // 3. Verify Search/List View is now active for configs
  await expect(SC.search.dropdown.input).toBeVisible();

  // 4. Navigate to New Form
  await clickNewToOpenEditForm(page);

  // 5. Verify Sport-Specific Form is rendered
  await expect(SC.form.inputName).toBeVisible();

  // 6. Cancel back to search context
  await SC.form.btnCancel.click();

  // 7. Verify we are back at the list and sport_id is preserved
  const { pathname } = new URL(page.url());
  expect(pathname.startsWith("/sport")).toBeTruthy();
  expect(pathname).not.toContain("/new_pa");
  expect(page.url()).toContain("sport_id=");
  expect(pathname).not.toContain("sport_config_id=");
});
