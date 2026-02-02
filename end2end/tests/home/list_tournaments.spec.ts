import { test, expect } from "@playwright/test";
import {
  openHomePage,
  selectSportPluginByName,
  goToListTournaments,
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
    const LIST = selectors(page).home.dashboard.tournamentsList;
    await expect(LIST.root).toBeVisible();
    await expect(LIST.filters.search).toBeEditable();
  });

  test("displays filter controls and table structure", async ({ page }) => {
    const LIST = selectors(page).home.dashboard.tournamentsList;

    await expect(page.locator("h2")).toHaveText("List Tournaments");
    await expect(LIST.filters.status).toHaveValue("Draft"); // Default according to Rust code
    await expect(LIST.filters.search).toBeEmpty();
  });

  test("finds seeded tournament via search", async ({ page }) => {
    const LIST = selectors(page).home.dashboard.tournamentsList;
    const targetName = SEED_DATA.DRAFT;

    // Execute search
    await fillAndBlur(LIST.filters.search, targetName);

    // Wait/Check
    // We expect exactly this entry in the table
    // Using a more specific selector to ensure we hit the table cell
    await expect(
      page.getByRole("cell", { name: targetName }).first(),
    ).toBeVisible({
      timeout: 10000,
    });
  });

  test("shows empty state when search finds no results", async ({ page }) => {
    const LIST = selectors(page).home.dashboard.tournamentsList;

    // Search for something impossible
    await fillAndBlur(LIST.filters.search, "X9Z9 NonExistent Tournament");

    // Expect empty message
    // Rust: data-testid="tournaments-list-empty"
    await expect(page.getByTestId("tournaments-list-empty")).toBeVisible();
    await expect(page.getByTestId("tournaments-list-empty")).toContainText(
      "No tournaments found",
    );
  });

  test("clicking a row reveals action buttons (Edit, Register, Show)", async ({
    page,
  }) => {
    const LIST = selectors(page).home.dashboard.tournamentsList;
    const targetName = SEED_DATA.DRAFT;

    await fillAndBlur(LIST.filters.search, targetName);

    // Find the row
    const cell = page.getByRole("cell", { name: targetName }).first();
    await expect(cell).toBeVisible();

    // Click to select the row (logic in tournaments.rs: signals `selected_id`)
    await cell.click();

    // Expect Action Row to appear
    // Rust: data-testid="row-actions"
    const actions = page.getByTestId("row-actions");
    await expect(actions).toBeVisible();

    // Verify Buttons for "Draft" status (as per Rust code: Register, Edit, Show)
    await expect(page.getByTestId("action-btn-register")).toBeVisible();
    await expect(page.getByTestId("action-btn-edit")).toBeVisible();
    await expect(page.getByTestId("action-btn-show")).toBeVisible();
  });

  test("edit button navigates to edit form", async ({ page }) => {
    const targetName = SEED_DATA.DRAFT;
    const LIST = selectors(page).home.dashboard.tournamentsList;

    await fillAndBlur(LIST.filters.search, targetName);
    const cell = page.getByRole("cell", { name: targetName }).first();
    await cell.click();

    // Click Edit
    await page.getByTestId("action-btn-edit").click();

    // Check we are on the Edit Page
    // Rust: data-testid="tournament-editor-title" -> "Edit Tournament"
    await expect(page.getByTestId("tournament-editor-title")).toHaveText(
      "Edit Tournament",
    );

    // Check pre-filled data
    // Rust: ValidatedTextInput name="tournament-name"
    await expect(page.locator('input[name="tournament-name"]')).toHaveValue(
      targetName,
    );
  });
});
