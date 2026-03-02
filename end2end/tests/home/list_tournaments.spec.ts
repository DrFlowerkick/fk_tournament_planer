import { test, expect } from "@playwright/test";
import {
  openHomePage,
  selectSportPluginByName,
  goToListTournaments,
  searchAndOpenByNameOnCurrentPage,
  fillAndBlur,
  selectors,
} from "../../helpers";

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
    const LIST = selectors(page).home.tournamentsList;
    await expect(LIST.root).toBeVisible();
    await expect(LIST.filterName).toBeEditable();
  });

  test("displays filter controls and table structure", async ({ page }) => {
    const LIST = selectors(page).home.tournamentsList;

    await expect(page.locator("h2")).toHaveText("List Tournaments");
    await expect(LIST.filters.status).toHaveValue("Draft"); // Default according to Rust code
    await expect(LIST.filterName).toBeEmpty();
  });

  test("finds seeded tournament via search", async ({ page }) => {
    const LIST = selectors(page).home.tournamentsList;
    const targetName = SEED_DATA.DRAFT;

    // Execute search
    await fillAndBlur(LIST.filterName, targetName);

    // Wait/Check
    // We expect exactly this entry in the table
    // Using a more specific selector to ensure we hit the table cell
    await searchAndOpenByNameOnCurrentPage(page, targetName, "tournament_id");
  });

  test("shows empty state when search finds no results", async ({ page }) => {
    const LIST = selectors(page).home.tournamentsList;

    // Search for something impossible
    await fillAndBlur(LIST.filterName, "X9Z9 NonExistent Tournament");

    // Expect empty message
    // Rust: data-testid="tournaments-list-empty"
    await expect(page.getByTestId("tournaments-list-empty")).toBeVisible();
    await expect(page.getByTestId("tournaments-list-empty")).toContainText(
      "No tournaments found",
    );
  });

  test("clicking a row reveals action buttons (Edit, Copy)", async ({
    page,
  }) => {
    const LIST = selectors(page).home.tournamentsList;
    const targetName = SEED_DATA.DRAFT;

    await fillAndBlur(LIST.filterName, targetName);

    // Find the row
    const cell = await searchAndOpenByNameOnCurrentPage(page, targetName, "tournament_id");
    await expect(cell).toBeVisible();

    // Verify Buttons for "Draft" status (as per Rust code: Copy, Edit)
    await expect(page.getByTestId("action-btn-copy")).toBeVisible();
    await expect(page.getByTestId("action-btn-edit")).toBeVisible();
  });

  test("edit button navigates to edit form", async ({ page }) => {
    const targetName = SEED_DATA.DRAFT;
    const LIST = selectors(page).home.tournamentsList;
    const FORM = selectors(page).home.editTournament;

    const cell = await searchAndOpenByNameOnCurrentPage(page, targetName, "tournament_id");

    // Click Edit
    await page.getByTestId("action-btn-edit").click();

    // Check we are on the Edit Page
    // Rust: data-testid="tournament-editor-title" -> "Edit Tournament"
    await expect(page.getByTestId("tournament-editor-title")).toHaveText(
      "Edit Tournament",
    );

    // Check pre-filled data
    await expect(FORM.inputs.name).toHaveValue(targetName);
  });
});
