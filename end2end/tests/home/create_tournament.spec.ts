import { test, expect } from "@playwright/test";
import {
  openHomePage,
  selectSportPluginByName,
  goToNewTournament,
  goToListTournaments,
} from "../../helpers/home";
import { selectors } from "../../helpers/selectors";

const PLUGINS = {
  GENERIC: "Generic Sport",
};

/**
 * Generates a unique tournament name to avoid DB conflicts during parallel tests.
 */
function makeUniqueName(base: string): string {
  return `${base} [${Date.now()}-${Math.floor(Math.random() * 1000)}]`;
}

test.describe("Create New Tournament", () => {
  let sportId: string;

  test.beforeEach(async ({ page }) => {
    // 1. Navigation
    await openHomePage(page);
    sportId = await selectSportPluginByName(page, PLUGINS.GENERIC);

    // Navigation to creation (without sportId, as context is already present)
    await goToNewTournament(page);
  });

  test("displays the creation form with correct input fields", async ({
    page,
  }) => {
    const FORM = selectors(page).home.dashboard.editTournament;

    await expect(FORM.title).toHaveText(/New Tournament|Plan Tournament/i);
    await expect(FORM.inputs.name).toBeEmpty();
    await expect(FORM.inputs.entrants).toBeVisible();
    await expect(FORM.actions.save).toBeVisible();
  });

  test("cancel button navigates back to dashboard or list", async ({
    page,
  }) => {
    const FORM = selectors(page).home.dashboard.editTournament;
    const DASH = selectors(page).home.dashboard;

    await expect(FORM.inputs.name).toBeEditable();
    await FORM.inputs.name.fill("To Be Cancelled");

    await FORM.actions.cancel.click();

    await expect(FORM.root).toBeHidden();
    await expect(DASH.nav.planNew).toBeVisible();
  });

  test("successfully creates a tournament and redirects to list", async ({
    page,
  }) => {
    const FORM = selectors(page).home.dashboard.editTournament;
    const LIST = selectors(page).home.dashboard.tournamentsList;

    const tourneyName = makeUniqueName("E2E Success");

    await FORM.inputs.name.fill(tourneyName);
    await FORM.inputs.entrants.fill("32");

    await expect(FORM.actions.save).toBeEnabled();
    await FORM.actions.save.click();

    await page.waitForURL(/tournament_id=/, { timeout: 20000 });
    // Direct navigation to tournament list
    await goToListTournaments(page);

    // Wait until list and search are ready
    await expect(LIST.root).toBeVisible({ timeout: 10000 });
    await expect(LIST.filters.search).toBeEditable();

    await LIST.filters.search.fill(tourneyName);

    await expect(page.getByRole("cell", { name: tourneyName })).toBeVisible({
      timeout: 10000,
    });
  });

  test.describe("Validation & Normalization", () => {
    test("save button is disabled without a tournament name", async ({
      page,
    }) => {
      const FORM = selectors(page).home.dashboard.editTournament;

      await FORM.inputs.entrants.fill("16");
      await FORM.inputs.name.fill("");
      await FORM.inputs.name.blur();

      await expect(FORM.actions.save).toBeDisabled();

      await FORM.inputs.name.fill(makeUniqueName("Valid Name Check"));
      await FORM.inputs.name.blur();

      await expect(FORM.actions.save).toBeEnabled();
    });

    test("save button is disabled with fewer than 2 entrants", async ({
      page,
    }) => {
      const FORM = selectors(page).home.dashboard.editTournament;

      await FORM.inputs.name.fill(makeUniqueName("Entrant Valid Check"));

      await FORM.inputs.entrants.fill("1");
      await FORM.inputs.entrants.blur();

      await expect(FORM.actions.save).toBeDisabled();

      await FORM.inputs.entrants.fill("2");
      await FORM.inputs.entrants.blur();

      await expect(FORM.actions.save).toBeEnabled();
    });

    test("normalizes whitespace in tournament name", async ({ page }) => {
      const FORM = selectors(page).home.dashboard.editTournament;
      const LIST = selectors(page).home.dashboard.tournamentsList;

      const uniqueSuffix = `Trim Check ${Date.now()}`;
      const dirtyName = `  Chaos   ${uniqueSuffix}  `;
      const cleanName = `Chaos ${uniqueSuffix}`;

      await FORM.inputs.name.fill(dirtyName);
      await FORM.inputs.entrants.fill("8");

      await expect(FORM.actions.save).toBeEnabled();
      await FORM.actions.save.click();

      await page.waitForURL(/tournament_id=/, { timeout: 20000 });

      // DIREKTE Navigation zur Liste
      await goToListTournaments(page);

      await expect(LIST.root).toBeVisible();
      await expect(LIST.filters.search).toBeEditable();

      await LIST.filters.search.fill(cleanName);

      const cell = page.getByRole("cell", { name: cleanName });
      await expect(cell).toBeVisible();
      await expect(cell).toHaveText(cleanName);
    });

    test("validates swiss system specific fields", async ({ page }) => {
      const FORM = selectors(page).home.dashboard.editTournament;

      await FORM.inputs.name.fill(makeUniqueName("Swiss Mode"));
      await FORM.inputs.entrants.fill("10");

      await expect(FORM.inputs.mode).toBeVisible();
      await FORM.inputs.mode.selectOption({ label: "Swiss System" });

      await expect(FORM.inputs.num_rounds_swiss).toBeVisible({
        timeout: 10000,
      });
      await expect(FORM.inputs.num_rounds_swiss).toBeEditable();

      // 1. Invalid Rounds (0)
      await FORM.inputs.num_rounds_swiss.fill("0");
      await FORM.inputs.num_rounds_swiss.blur();
      await expect(FORM.actions.save).toBeDisabled();

      // 2. Valid Rounds
      await FORM.inputs.num_rounds_swiss.fill("5");
      await FORM.inputs.num_rounds_swiss.blur();

      await expect(FORM.actions.save).toBeEnabled();
    });
  });
});
