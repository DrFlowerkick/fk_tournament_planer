import { test, expect } from "@playwright/test";
import {
  openHomePage,
  selectSportPluginByName,
  goToNewTournament,
  goToListTournaments,
  fillAndBlur,
  waitForEditTournamentUrl,
  makeUniqueName,
  searchAndOpenByNameOnCurrentPage,
  expectFieldValidity,
  selectors,
  waitForNewTournamentUrl,
  selectAndBlur,
} from "../../helpers";

const PLUGINS = {
  GENERIC: "Generic Sport",
};

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
    const FORM = selectors(page).home.editTournament;

    await expect(FORM.title).toHaveText(/New Tournament|Plan Tournament/i);
    await expect(FORM.inputs.name).toBeEmpty();
    await expect(FORM.inputs.entrants).toBeVisible();
  });

  test("close button navigates back to dashboard or list", async ({
    page,
  }) => {
    const FORM = selectors(page).home.editTournament;
    const DASH = selectors(page).home.dashboard;

    await expect(FORM.inputs.name).toBeEditable();

    await FORM.actions.close.click();

    await expect(FORM.root).toBeHidden();
    await expect(DASH.nav.planNew).toBeVisible();
  });

  test("successfully creates a tournament and handles redirect", async ({
    page,
  }) => {
    const FORM = selectors(page).home.editTournament;
    const LIST = selectors(page).home.tournamentsList;
    const TOASTS = selectors(page).toasts;

    const tourneyName = makeUniqueName("E2E Create Success");

    await fillAndBlur(FORM.inputs.name, tourneyName);
    await fillAndBlur(FORM.inputs.entrants, "32");

    // 1. Wait for URL update (Creation usually redirects to Edit/View with ID)
    await waitForEditTournamentUrl(page);

    // 2. Verify Persistence via List
    await goToListTournaments(page);

    // Wait until list and filter name are ready
    await expect(LIST.root).toBeVisible({ timeout: 10000 });
    await expect(LIST.filterName).toBeEditable();

    await searchAndOpenByNameOnCurrentPage(page, tourneyName, "tournament_id");
  });

  test("full flow: create, navigate to list, edit existing, modify and save", async ({
    page,
  }) => {
    const FORM = selectors(page).home.editTournament;
    const LIST = selectors(page).home.tournamentsList;
    const TOASTS = selectors(page).toasts;

    // --- Step 1: Create initial tournament ---
    const initialName = makeUniqueName("Flow Initial");
    await fillAndBlur(FORM.inputs.name, initialName);
    await fillAndBlur(FORM.inputs.entrants, "16");

    // Wait for creation to finish
    await waitForEditTournamentUrl(page);

    // --- Step 2: Go to List and Find it ---
    await goToListTournaments(page);
    await searchAndOpenByNameOnCurrentPage(page, initialName, "tournament_id");

    // --- Step 3: Enter Edit Mode ---
    const editBtn = page.getByTestId("action-btn-edit");
    await expect(editBtn).toBeVisible();
    await editBtn.click();

    // Check we are back in the form
    await expect(FORM.title).toHaveText(/Edit Tournament/i);
    await expect(FORM.inputs.name).toHaveValue(initialName);

    // --- Step 4: Modify Data ---
    const updatedName = makeUniqueName("Flow Updated");
    await fillAndBlur(FORM.inputs.name, updatedName);
    // Optional: Change another field if needed
    // await FORM.inputs.entrants.fill("20");

    // --- Step 5: Verify Persistence ---
    // Verify update in List
    await goToListTournaments(page);
    await searchAndOpenByNameOnCurrentPage(page, updatedName, "tournament_id");
  });

  test.describe("Validation & Normalization", () => {
    test("save only happens if all required fields are valid", async ({
      page,
    }) => {
      const FORM = selectors(page).home.editTournament;

      await fillAndBlur(FORM.inputs.entrants, "16");
      await fillAndBlur(FORM.inputs.name, "");

      // we are still on the new tournament page, so no tournament_id in url, but we can check the form state
      await waitForNewTournamentUrl(page);

      await fillAndBlur(FORM.inputs.name, makeUniqueName("Valid Name Check"));

      // Now the form should be valid, which triggers automatic save and navigation to edit page
      await waitForEditTournamentUrl(page);
    });

    test("save button is disabled with fewer than 2 entrants", async ({
      page,
    }) => {
      const FORM = selectors(page).home.editTournament;

      await fillAndBlur(FORM.inputs.entrants, "1");
      await expectFieldValidity(FORM.inputs.entrants, "1", true);

      await fillAndBlur(FORM.inputs.entrants, "2");
      await expectFieldValidity(FORM.inputs.entrants, "2", false);
    });

    test("normalizes whitespace in tournament name", async ({ page }) => {
      const FORM = selectors(page).home.editTournament;
      const LIST = selectors(page).home.tournamentsList;

      const uniqueSuffix = `Trim Check ${Date.now()}`;
      const dirtyName = `  Chaos   ${uniqueSuffix}  `;
      const cleanName = `Chaos ${uniqueSuffix}`;

      await fillAndBlur(FORM.inputs.name, dirtyName);
      await fillAndBlur(FORM.inputs.entrants, "8");

      await goToListTournaments(page);

      await expect(LIST.root).toBeVisible();
      await expect(LIST.filterName).toBeEditable();

      await LIST.filterName.fill(cleanName);
      await searchAndOpenByNameOnCurrentPage(page, cleanName, "tournament_id");
    });

    test("validates swiss system specific fields", async ({ page }) => {
      const FORM = selectors(page).home.editTournament;

      await fillAndBlur(FORM.inputs.name, makeUniqueName("Swiss Validation"));
      await fillAndBlur(FORM.inputs.entrants, "10");

      await waitForEditTournamentUrl(page);

      await expect(FORM.inputs.mode).toBeVisible();
      await selectAndBlur(FORM.inputs.mode, "Swiss System (0 rounds)");

      await expect(FORM.inputs.num_rounds_swiss).toBeVisible();
      await expect(FORM.inputs.num_rounds_swiss).toBeEditable();

      // 1. Invalid Rounds (0)
      await fillAndBlur(FORM.inputs.num_rounds_swiss, "0");
      await expectFieldValidity(FORM.inputs.num_rounds_swiss, "0", true);

      // 2. Valid Rounds
      await fillAndBlur(FORM.inputs.num_rounds_swiss, "5");
      await expectFieldValidity(FORM.inputs.num_rounds_swiss, "5", false);
    });
  });
});
