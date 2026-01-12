import { test, expect } from "@playwright/test";
import {
  openHomePage,
  selectSportPluginByName,
  goToNewTournament,
} from "../../helpers/home";
import { selectors } from "../../helpers/selectors";

const PLUGINS = {
  GENERIC: "Generic Sport",
};

test.describe("Create New Tournament", () => {
  let sportId: string;

  test.beforeEach(async ({ page }) => {
    await openHomePage(page);
    sportId = await selectSportPluginByName(page, PLUGINS.GENERIC);
    await goToNewTournament(page, sportId);
  });

  test("displays the creation form with correct input fields", async ({
    page,
  }) => {
    const FORM = selectors(page).home.dashboard.newTournament;

    await expect(FORM.root).toBeVisible();

    // Verify Title
    await expect(page.locator("h2")).toHaveText(
      /New Tournament|Plan Tournament/i
    );

    // Verify Inputs
    await expect(FORM.inputs.name).toBeVisible();
    await expect(FORM.inputs.name).toBeEmpty();

    await expect(FORM.inputs.entrants).toBeVisible();
    // Since default is technically 0 in struct, but UI might have a placeholder or minimum Default (e.g. 16)
    // We just check visibility for now or a reasonable default if implemented later.

    // Verify Actions
    await expect(FORM.actions.save).toBeVisible();
    await expect(FORM.actions.cancel).toBeVisible();
  });

  test("cancel button navigates back to dashboard or list", async ({
    page,
  }) => {
    const FORM = selectors(page).home.dashboard.newTournament;
    const DASH = selectors(page).home.dashboard;

    await FORM.inputs.name.fill("Cancelled Tournament");
    await FORM.actions.cancel.click();

    // Expectation: Form is gone, we are likely back at Dashboard root or List
    // Assuming redirects to Tournaments List by default for Cancel
    await expect(FORM.root).toBeHidden();
    await expect(DASH.nav.planNew).toBeVisible();
  });

  test("successfully creates a tournament and redirects to list", async ({
    page,
  }) => {
    const FORM = selectors(page).home.dashboard.newTournament;
    const LIST = selectors(page).home.dashboard.tournamentsList;

    const tourneyName = `E2E Cup ${Date.now()}`;

    // Fill Form
    await FORM.inputs.name.fill(tourneyName);
    await FORM.inputs.entrants.fill("32"); // Specific valid number

    // Save
    await FORM.actions.save.click();

    // Expectation: Redirect to List
    await expect(FORM.root).toBeHidden();
    await expect(LIST.root).toBeVisible();

    // NOTE: This verifies integration. It requires the Backend to actually SAVE and LIST the item.
    // Step 1: Filter list to find our specific item (good practice in shared envs)
    await LIST.filters.search.fill(tourneyName);

    // Step 2: Check if row exists
    // We wait for the table rows to update
    await expect(page.getByRole("cell", { name: tourneyName })).toBeVisible();
  });

  test.describe("Validation & Normalization", () => {
    test("save button is disabled without a tournament name", async ({
      page,
    }) => {
      const FORM = selectors(page).home.dashboard.newTournament;

      // Fill only entrants, leave name empty
      await FORM.inputs.entrants.fill("16");
      await FORM.inputs.name.fill(""); // Ensure empty

      // Expectation: Save button is disabled
      await expect(FORM.actions.save).toBeDisabled();

      // Make it valid to verify toggle
      await FORM.inputs.name.fill("Valid Name");
      await expect(FORM.actions.save).toBeEnabled();
    });

    test("save button is disabled with fewer than 2 entrants", async ({
      page,
    }) => {
      const FORM = selectors(page).home.dashboard.newTournament;

      await FORM.inputs.name.fill("Invalid Entrants Cup");

      // Invalid case: 1 entrant
      await FORM.inputs.entrants.fill("1");
      await expect(FORM.actions.save).toBeDisabled();

      // Valid case: 2 entrants (min)
      await FORM.inputs.entrants.fill("2");
      await expect(FORM.actions.save).toBeEnabled();
    });

    test("normalizes whitespace in tournament name", async ({ page }) => {
      const FORM = selectors(page).home.dashboard.newTournament;
      const LIST = selectors(page).home.dashboard.tournamentsList;

      const dirtyName = "  Chaos   Spacing   Cup  ";
      const cleanName = "Chaos Spacing Cup"; // Expected result from normalize_ws logic

      await FORM.inputs.name.fill(dirtyName);
      await FORM.inputs.entrants.fill("8");

      await FORM.actions.save.click();

      // Verify redirection to list
      await expect(FORM.root).toBeHidden();
      await expect(LIST.root).toBeVisible();

      // Verify data is stored normalized
      await LIST.filters.search.fill(cleanName);
      await expect(page.getByRole("cell", { name: cleanName })).toBeVisible();

      // Double check: searching for the exact dirty string typically shouldn't match exact cell text
      // unless the search itself is fuzzy. But the cell text MUST be clean.
      const cell = page.getByRole("cell", { name: cleanName });
      await expect(cell).toHaveText(cleanName);
    });
  });
});
