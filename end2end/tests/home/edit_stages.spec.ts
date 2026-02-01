import { test, expect } from "@playwright/test";
import {
  makeUniqueName,
  selectors,
  openHomePage,
  selectSportPluginByName,
  goToNewTournament,
  fillAndBlur,
} from "../../helpers";

const PLUGINS = {
  GENERIC: "Generic Sport",
};

test.describe("Configuration of Tournament Stages", () => {
  test.beforeEach(async ({ page }) => {
    // 1. Setup: Go to new tournament page
    await openHomePage(page);
    await selectSportPluginByName(page, PLUGINS.GENERIC);
    await goToNewTournament(page);
  });

  test("SingleStage: navigates transparently through stage editor to group editor", async ({
    page,
  }) => {
    const FORM = selectors(page).home.dashboard.editTournament;
    const STAGE = selectors(page).home.dashboard.editStage;
    const GROUP = selectors(page).home.dashboard.editGroup;

    // 1. Create Tournament with SingleStage
    await fillAndBlur(FORM.inputs.name, makeUniqueName("Single Stage Test"));
    await fillAndBlur(FORM.inputs.entrants, "16");
    await FORM.inputs.mode.selectOption({ label: "Single Stage" }); // Default, but explicit is better
    await FORM.actions.save.click();

    // 2. Wait for navigation links to appear
    await expect(FORM.links.configureSingleStage).toBeVisible({
      timeout: 10000,
    });

    // 3. Navigate to Stage/Group Config
    await FORM.links.configureSingleStage.click();

    // 4. Assertions
    // "EditTournamentStage" title should NOT be visible (transparent behavior)
    await expect(STAGE.title).toBeHidden();

    // "EditTournamentGroup" title SHOULD be visible
    await expect(GROUP.title).toBeVisible();
    await expect(GROUP.title).toHaveText("Edit Tournament Group");
  });

  test("SwissSystem: navigates transparently through stage editor to group editor", async ({
    page,
  }) => {
    const FORM = selectors(page).home.dashboard.editTournament;
    const STAGE = selectors(page).home.dashboard.editStage;
    const GROUP = selectors(page).home.dashboard.editGroup;

    // 1. Create Tournament with Swiss System
    await fillAndBlur(FORM.inputs.name, makeUniqueName("Swiss System Test"));
    await fillAndBlur(FORM.inputs.entrants, "16");
    await FORM.inputs.mode.selectOption({ label: "Swiss System" });
    await fillAndBlur(FORM.inputs.num_rounds_swiss, "5");
    await FORM.actions.save.click();

    // 2. Wait for navigation links to appear
    await expect(FORM.links.configureSwissSystem).toBeVisible({
      timeout: 10000,
    });

    // 3. Navigate to Stage/Group Config
    await FORM.links.configureSwissSystem.click();

    // 4. Assertions
    // "EditTournamentStage" title should NOT be visible (transparent behavior)
    await expect(STAGE.title).toBeHidden();

    // "EditTournamentGroup" title SHOULD be visible
    await expect(GROUP.title).toBeVisible();
    await expect(GROUP.title).toHaveText("Edit Tournament Group");
  });

  test("PoolAndFinalStage: Configure Pool Stage shows stage editor and group options", async ({
    page,
  }) => {
    const FORM = selectors(page).home.dashboard.editTournament;
    const STAGE = selectors(page).home.dashboard.editStage;
    const GROUP = selectors(page).home.dashboard.editGroup;

    // 1. Create Tournament with Pool And Final Stage
    await fillAndBlur(FORM.inputs.name, makeUniqueName("Pool Stage Test"));
    await fillAndBlur(FORM.inputs.entrants, "32"); // Enough entrants for groups
    await FORM.inputs.mode.selectOption({ label: "Pool and Final Stage" });
    await FORM.actions.save.click();

    // 2. Wait for navigation links to appear
    await expect(FORM.links.configurePoolStage).toBeVisible({
      timeout: 10000,
    });

    // 3. Navigate to Stage Config
    await FORM.links.configurePoolStage.click();

    // 4. Assertions
    // "EditTournamentStage" title SHOULD be visible
    await expect(STAGE.title).toBeVisible();
    // Use ignoreCase/regex to match title text
    await expect(STAGE.title).toHaveText(/Edit Pool Stage/i);

    // "EditTournamentGroup" title should NOT be visible (we are at stage level)
    await expect(GROUP.title).toBeHidden();

    // Input for number of groups should be visible
    await expect(STAGE.inputs.numGroups).toBeVisible();

    // If default is e.g. 1 group, link for group 0 should exist
    // (Assuming default behavior creates at least 1 group or test waits for interaction)
    // Note: We might need to fill the input if it's empty by default,
    // but typically Stages are initialized with defaults.
    // Let's assume we want at least Link 0 to verify list rendering.

    // Optional: Set groups to 4 to see if 4 links appear (Generic check)
    await fillAndBlur(STAGE.inputs.numGroups, "4");

    await expect(STAGE.groupLink(0)).toBeVisible();
    await expect(STAGE.groupLink(1)).toBeVisible();
    await expect(STAGE.groupLink(2)).toBeVisible();
    await expect(STAGE.groupLink(3)).toBeVisible();
  });
});
