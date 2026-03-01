import { test, expect } from "@playwright/test";
import {
  openHomePage,
  selectSportPluginByName,
  selectAndBlur,
  fillAndBlur,
  goToNewTournament,
  waitForEditTournamentUrl,
  makeUniqueName,
  selectors
} from "../../helpers";

const PLUGINS = {
  GENERIC: "Generic Sport",
};

test.describe("Tournament Mode Configurations", () => {
  test.beforeEach(async ({ page }) => {
    await openHomePage(page);
    await selectSportPluginByName(page, PLUGINS.GENERIC);
    await goToNewTournament(page);
  });

  test("shows correct configuration links for Single Stage", async ({ page }) => {
    const FORM = selectors(page).home.editTournament;

    // Single Stage is usually default, but let's be explicit
    await selectAndBlur(FORM.inputs.mode, "Single Stage");

    // links are only visible when editing a saved tournament, so we need to fill in a name
    // and valid number of entrants to trigger the "save" state
    const uniqueName = makeUniqueName("Test Tournament");
    await fillAndBlur(FORM.inputs.name, uniqueName);
    await fillAndBlur(FORM.inputs.entrants, "16");

    await waitForEditTournamentUrl(page);

    // Expect Single Stage Link
    await expect(FORM.links.configureSingleStage).toBeVisible();
    
    // Ensure others are hidden
    await expect(FORM.links.configurePoolStage).toBeHidden();
    await expect(FORM.links.configureSwissSystem).toBeHidden();
  });

  test("shows correct configuration links for Pool and Final Stage", async ({
    page,
  }) => {
    const FORM = selectors(page).home.editTournament;

    await selectAndBlur(FORM.inputs.mode, "Pool and Final Stage");

    // links are only visible when editing a saved tournament, so we need to fill in a name
    // and valid number of entrants to trigger the "save" state
    const uniqueName = makeUniqueName("Test Tournament");
    await fillAndBlur(FORM.inputs.name, uniqueName);
    await fillAndBlur(FORM.inputs.entrants, "16");

    await waitForEditTournamentUrl(page);

    // Expect Two Links: Pool and Final
    await expect(FORM.links.configurePoolStage).toBeVisible();
    await expect(FORM.links.configureFinalStage).toBeVisible();

    // Ensure Single/Swiss are hidden
    await expect(FORM.links.configureSingleStage).toBeHidden();
    await expect(FORM.links.configureSwissSystem).toBeHidden();
  });

  test("shows correct configuration links for Swiss System", async ({ page }) => {
    const FORM = selectors(page).home.editTournament;

    await selectAndBlur(FORM.inputs.mode, "Swiss System (0 rounds)");

    // links are only visible when editing a saved tournament, so we need to fill in a name
    // and valid number of entrants to trigger the "save" state
    // Additionally, for Swiss System, the number of rounds input must be filled to trigger the save state
    const uniqueName = makeUniqueName("Test Tournament");
    await fillAndBlur(FORM.inputs.name, uniqueName);
    await fillAndBlur(FORM.inputs.entrants, "16");
    await fillAndBlur(FORM.inputs.num_rounds_swiss, "5");

    await waitForEditTournamentUrl(page);

    // Expect Swiss config link
    await expect(FORM.links.configureSwissSystem).toBeVisible();

    // And the extra input for rounds
    await expect(FORM.inputs.num_rounds_swiss).toBeVisible();
  });
});