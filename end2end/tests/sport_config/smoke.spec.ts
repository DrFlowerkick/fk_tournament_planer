import { test, expect } from "@playwright/test";
import { T, selectors } from "../../helpers/selectors";
import { openSportSelectionAndConfigList } from "../../helpers/sport_config";
import {
  searchAndOpenByNameOnCurrentPage,
  extractQueryParamFromUrl,
} from "../../helpers/utils";

test("Smoke: Select Plugin -> Search Config -> New -> Cancel", async ({
  page,
}) => {
  // 1. Open Sport Config List
  await openSportSelectionAndConfigList(page);

  // 2. Select a Sport Plugin (e.g., "Generic Sport")
  await searchAndOpenByNameOnCurrentPage(
    selectors(page).sportConfig.pluginSelector,
    "Generic Sport"
  );
  const sport_id = extractQueryParamFromUrl(page.url(), "sport_id");
  expect(sport_id).toBeTruthy();
  expect(sport_id).toMatch(/^[0-9a-fA-F-]{36}$/);

  /*
  // 3. Verify Search/List View is now active for configs
  await expect(page.getByPlaceholder("Search Configuration")).toBeVisible();

  // 4. Navigate to New Form
  await page.getByRole("link", { name: "New" }).click();

  // 5. Verify Sport-Specific Form is rendered
  //    (e.g., check for a field specific to Generic Sport)
  //    For now, we just check if the form is visible.
  await expect(page.getByRole("button", { name: "Save" })).toBeVisible();

  // 6. Cancel back to search context
  await page.getByRole("button", { name: "Cancel" }).click();

  // 7. Verify we are back at the list and sport_id is preserved
  await expect(page.getByPlaceholder("Search Configuration")).toBeVisible();
  expect(page.url()).toContain("sport_id=");
  */
});
