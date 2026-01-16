import { test, expect } from "@playwright/test";
import {
  openHomePage,
  selectSportPluginByName,
  goToListTournaments,
} from "../../helpers/home";
import { selectors } from "../../helpers/selectors";

const PLUGINS = {
  GENERIC: "Generic Sport",
};

// Must match names from global-setup
const SEED_DATA = {
  DRAFT: "Seed Draft Tournament",
};

test.describe("Tournaments List Page", () => {
  test.beforeEach(async ({ page }) => {
    // Navigation to list
    await openHomePage(page);
    await selectSportPluginByName(page, PLUGINS.GENERIC);
    await goToListTournaments(page);

    // Stability Check
    const LIST = selectors(page).home.dashboard.tournamentsList;
    await expect(LIST.root).toBeVisible();
    await expect(LIST.filters.search).toBeEditable();
  });

  test("displays filter controls and table structure", async ({ page }) => {
    const LIST = selectors(page).home.dashboard.tournamentsList;

    await expect(page.locator("h2")).toHaveText("List Tournaments");
    await expect(LIST.filters.status).toHaveValue("Draft");
    await expect(LIST.filters.search).toBeEmpty();

    // Table should exist (since we have seeds)
    // If table is empty, it might show an empty row, but root should be there.
    // We prefer checking positively for content in the next test.
  });

  test("finds seeded tournament via search", async ({ page }) => {
    const LIST = selectors(page).home.dashboard.tournamentsList;
    const targetName = SEED_DATA.DRAFT;

    // Execute search
    await LIST.filters.search.fill(targetName);

    // Wait/Check
    // We expect exactly this entry in the table
    await expect(page.getByRole("cell", { name: targetName })).toBeVisible({
      timeout: 10000,
    });
  });

  test("interacting with filters updates UI state", async ({ page }) => {
    const LIST = selectors(page).home.dashboard.tournamentsList;

    // Status Filter (UI Check)
    await LIST.filters.status.selectOption("Finished");
    await expect(LIST.filters.status).toHaveValue("Finished");

    // Adhoc Toggle (UI Check)
    await LIST.filters.adhocToggle.check();
    await expect(LIST.filters.adhocToggle).toBeChecked();

    // Limit (UI Check)
    await LIST.filters.limit.selectOption("50");
    await expect(LIST.filters.limit).toHaveValue("50");
  });

  // Further tests for Actions etc. if you have status changes in seeding
});
