import { test, expect } from "@playwright/test";
import {
  openHomePage,
  selectSportPluginByName,
  goToNewTournament,
  goToListTournaments,
  fillAndBlur,
  makeUniqueName,
  selectors,
} from "../../helpers";
import { getToastSelectors } from "../../helpers/selectors/common";

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
    await fillAndBlur(FORM.inputs.name, "To Be Cancelled");

    await FORM.actions.cancel.click();

    await expect(FORM.root).toBeHidden();
    await expect(DASH.nav.planNew).toBeVisible();
  });

  test("successfully creates a tournament, shows toast and handles redirect", async ({
    page,
  }) => {
    const FORM = selectors(page).home.dashboard.editTournament;
    const LIST = selectors(page).home.dashboard.tournamentsList;
    const TOASTS = getToastSelectors(page);

    const tourneyName = makeUniqueName("E2E Create Success");

    await fillAndBlur(FORM.inputs.name, tourneyName);
    await fillAndBlur(FORM.inputs.entrants, "32");

    await expect(FORM.actions.save).toBeEnabled();
    await FORM.actions.save.click();

    // 1. Verify Toast appears
    await expect(TOASTS.success).toBeVisible();
    await expect(TOASTS.success).toContainText(/saved/i);

    // 2. Wait for URL update (Creation usually redirects to Edit/View with ID)
    await page.waitForURL(/tournament_id=/, { timeout: 20000 });

    // 3. Verify Persistence via List
    await goToListTournaments(page);

    // Wait until list and search are ready
    await expect(LIST.root).toBeVisible({ timeout: 10000 });
    await expect(LIST.filters.search).toBeEditable();

    await fillAndBlur(LIST.filters.search, tourneyName);

    await expect(page.getByRole("cell", { name: tourneyName })).toBeVisible({
      timeout: 10000,
    });
  });

  test("full flow: create, navigate to list, edit existing, modify and save", async ({
    page,
  }) => {
    const FORM = selectors(page).home.dashboard.editTournament;
    const LIST = selectors(page).home.dashboard.tournamentsList;
    const TOASTS = getToastSelectors(page);

    // --- Step 1: Create initial tournament ---
    const initialName = makeUniqueName("Flow Initial");
    await fillAndBlur(FORM.inputs.name, initialName);
    await fillAndBlur(FORM.inputs.entrants, "16");
    await FORM.actions.save.click();

    // Wait for creation to finish (Toast & URL)
    await expect(TOASTS.success).toBeVisible();
    await page.waitForURL(/tournament_id=/);

    // --- Step 2: Go to List and Find it ---
    await goToListTournaments(page);
    await fillAndBlur(LIST.filters.search, initialName);

    const rowCell = page.getByRole("cell", { name: initialName }).first();
    await expect(rowCell).toBeVisible();

    // --- Step 3: Enter Edit Mode ---
    await rowCell.click();
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

    // --- Step 5: Save Changes ---
    await FORM.actions.save.click();

    // --- Step 6: Verify Success Toast & Persistence ---
    // Note: If the previous toast is still there (animations), this might pass immediately.
    // For now, checking visibility is usually sufficient as 'save' triggers a re-render/new toast.
    await expect(TOASTS.success).toBeVisible();
    // Assuming backend returns "Tournament saved"
    await expect(TOASTS.success).toContainText(/saved/i);

    // Verify update in List
    await goToListTournaments(page);
    await fillAndBlur(LIST.filters.search, updatedName);
    await expect(page.getByRole("cell", { name: updatedName })).toBeVisible();
    await expect(
      page.getByRole("cell", { name: initialName }),
    ).not.toBeVisible();
  });

  test.describe("Validation & Normalization", () => {
    test("save button is disabled without a tournament name", async ({
      page,
    }) => {
      const FORM = selectors(page).home.dashboard.editTournament;

      await fillAndBlur(FORM.inputs.entrants, "16");
      await fillAndBlur(FORM.inputs.name, "");

      await expect(FORM.actions.save).toBeDisabled();

      await fillAndBlur(FORM.inputs.name, makeUniqueName("Valid Name Check"));

      await expect(FORM.actions.save).toBeEnabled();
    });

    test("save button is disabled with fewer than 2 entrants", async ({
      page,
    }) => {
      const FORM = selectors(page).home.dashboard.editTournament;

      await fillAndBlur(
        FORM.inputs.name,
        makeUniqueName("Entrant Valid Check"),
      );

      await fillAndBlur(FORM.inputs.entrants, "1");

      await expect(FORM.actions.save).toBeDisabled();

      await fillAndBlur(FORM.inputs.entrants, "2");

      await expect(FORM.actions.save).toBeEnabled();
    });

    test("normalizes whitespace in tournament name", async ({ page }) => {
      const FORM = selectors(page).home.dashboard.editTournament;
      const LIST = selectors(page).home.dashboard.tournamentsList;

      const uniqueSuffix = `Trim Check ${Date.now()}`;
      const dirtyName = `  Chaos   ${uniqueSuffix}  `;
      const cleanName = `Chaos ${uniqueSuffix}`;

      await fillAndBlur(FORM.inputs.name, dirtyName);
      await fillAndBlur(FORM.inputs.entrants, "8");

      await expect(FORM.actions.save).toBeEnabled();
      await FORM.actions.save.click();

      await page.waitForURL(/tournament_id=/, { timeout: 20000 });

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
