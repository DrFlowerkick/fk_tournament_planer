import { test, expect } from "@playwright/test";
import {
  openHomePage,
  selectSportPluginByName,
  goToNewTournament,
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
    const FORM = selectors(page).home.dashboard.editTournament;

    // Single Stage is usually default, but let's be explicit
    await FORM.inputs.mode.selectOption({ label: "Single Stage" });

    // Expect Single Stage Link
    await expect(page.getByTestId("link-configure-single-stage")).toBeVisible();
    
    // Ensure others are hidden
    await expect(page.getByTestId("link-configure-pool-stage")).toBeHidden();
    await expect(page.getByTestId("link-configure-swiss-system")).toBeHidden();
  });

  test("shows correct configuration links for Pool and Final Stage", async ({
    page,
  }) => {
    const FORM = selectors(page).home.dashboard.editTournament;

    await FORM.inputs.mode.selectOption({ label: "Pool and Final Stage" });

    // Expect Two Links: Pool and Final
    await expect(page.getByTestId("link-configure-pool-stage")).toBeVisible();
    await expect(page.getByTestId("link-configure-final-stage")).toBeVisible();

    // Ensure Single/Swiss are hidden
    await expect(page.getByTestId("link-configure-single-stage")).toBeHidden();
  });

  test("shows correct configuration links for Swiss System", async ({ page }) => {
    const FORM = selectors(page).home.dashboard.editTournament;

    await FORM.inputs.mode.selectOption({ label: "Swiss System" });

    // Expect Swiss config link
    await expect(page.getByTestId("link-configure-swiss-system")).toBeVisible();

    // And the extra input for rounds
    await expect(FORM.inputs.num_rounds_swiss).toBeVisible();
  });
});