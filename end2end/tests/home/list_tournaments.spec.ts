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

test.describe("Tournaments List Page", () => {
  let sportId: string;

  test.beforeEach(async ({ page }) => {
    await openHomePage(page);
    sportId = await selectSportPluginByName(page, PLUGINS.GENERIC);
    await goToListTournaments(page, sportId);
  });

  test("displays filter controls with default values", async ({ page }) => {
    const LIST = selectors(page).home.dashboard.tournamentsList;

    // Verify Title
    const header = page.locator("h2");
    await expect(header).toHaveText("List Tournaments");

    // Verify Filters exist and defaults
    await expect(LIST.filters.status).toBeVisible();
    await expect(LIST.filters.status).toHaveValue("Scheduling");

    await expect(LIST.filters.adhocToggle).toBeVisible();
    await expect(LIST.filters.adhocToggle).not.toBeChecked();

    await expect(LIST.filters.search).toBeVisible();
    await expect(LIST.filters.search).toBeEmpty();

    await expect(LIST.filters.limit).toBeVisible();
    await expect(LIST.filters.limit).toHaveValue("10");
  });

  test("displays empty state when no tournaments found (Initial State)", async ({
    page,
  }) => {
    const LIST = selectors(page).home.dashboard.tournamentsList;
    // Since DB is currently empty in E2E environment
    await expect(LIST.emptyState).toBeVisible();
    await expect(LIST.table.root).toBeHidden();
  });

  test("interacting with filters updates UI state", async ({ page }) => {
    const LIST = selectors(page).home.dashboard.tournamentsList;

    // Change Status
    await LIST.filters.status.selectOption("Finished");
    await expect(LIST.filters.status).toHaveValue("Finished");

    // Toggle Adhoc
    await LIST.filters.adhocToggle.check();
    await expect(LIST.filters.adhocToggle).toBeChecked();

    // Change Limit
    await LIST.filters.limit.selectOption("50");
    await expect(LIST.filters.limit).toHaveValue("50");
  });

  /*
   * NOTE: Requires DB Seeding (TODO when Write-API is ready)
   */
  test.describe("Row Interactions & Actions (Requires Seed Data)", () => {
    const pendingId = "mock-pending-id";
    const runningId = "mock-running-id";
    const resolvedId = "mock-resolved-id";

    test.skip("clicking a PENDING tournament row shows Edit/Show/Register", async ({
      page,
    }) => {
      const LIST = selectors(page).home.dashboard.tournamentsList;
      const row = LIST.table.rowById(pendingId);

      await row.click();
      await expect(LIST.table.actions.container).toBeVisible();

      await expect(LIST.table.actions.edit).toBeVisible();
      await expect(LIST.table.actions.show).toBeVisible();
      await expect(LIST.table.actions.register).toBeVisible();
      await expect(LIST.table.actions.results).toBeHidden();
    });

    test.skip("clicking a RUNNING tournament row shows Show only", async ({
      page,
    }) => {
      const LIST = selectors(page).home.dashboard.tournamentsList;
      await LIST.filters.status.selectOption("Running");

      const row = LIST.table.rowById(runningId);
      await row.click();

      await expect(LIST.table.actions.show).toBeVisible();
      await expect(LIST.table.actions.edit).toBeHidden();
      await expect(LIST.table.actions.register).toBeHidden();
    });

    test.skip("clicking a RESOLVED tournament row shows Results", async ({
      page,
    }) => {
      const LIST = selectors(page).home.dashboard.tournamentsList;
      await LIST.filters.status.selectOption("Finished");

      const row = LIST.table.rowById(resolvedId);
      await row.click();

      await expect(LIST.table.actions.edit).toBeHidden();
      await expect(LIST.table.actions.results).toBeVisible();
    });
  });
});
