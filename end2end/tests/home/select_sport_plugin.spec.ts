import { test, expect } from "@playwright/test";
import {
  openHomePage,
  selectSportPluginByName,
  expectSportViewActive,
} from "../../helpers/home";
import { selectors } from "../../helpers/selectors";

// We use names instead of hardcoded UUIDs
const PLUGINS = {
  GENERIC: "Generic Sport",
  DDC: "Double Disc Court (DDC)",
};

test.describe("Home Page - Sport Plugin Selection", () => {
  test("displays app description and sport selection grid when no sport is selected", async ({
    page,
  }) => {
    await openHomePage(page);

    const HOME = selectors(page).home;

    // Check static content (using regex to match partial text if needed, or exact string)
    await expect(HOME.hero.title).toHaveText(/Welcome|Tournament Planner/i);
    await expect(HOME.hero.description).not.toBeEmpty();

    // Grid should exist
    await expect(HOME.sportSelection.grid).toBeVisible();

    // Check availability by Name
    await expect(
      HOME.sportSelection.pluginButtonByName(PLUGINS.GENERIC)
    ).toBeVisible();
    await expect(
      HOME.sportSelection.pluginButtonByName(PLUGINS.DDC)
    ).toBeVisible();
  });

  test("clicking a sport plugin button (by name) updates URL and hides selection grid", async ({
    page,
  }) => {
    await openHomePage(page);

    // Perform selection by Name - this dynamically fetches the UUID the app is using
    const detectedId = await selectSportPluginByName(page, PLUGINS.GENERIC);

    // Verify state after selection using the detected ID
    await expectSportViewActive(page, detectedId);
  });

  test("visiting home with sport_id query param directly hides selection grid", async ({
    page,
  }) => {
    // 1. We first need to find out a valid ID. In a real scenario, we might know one,
    // but here we "learn" it from the UI first to keep the test robust.
    await openHomePage(page);
    const validId = await selectSportPluginByName(page, PLUGINS.DDC);

    // 2. Now we test the direct navigation feature
    await page.goto(`/?sport_id=${validId}`);
    await page.waitForLoadState("domcontentloaded");

    const HOME = selectors(page).home;

    // Hero might still be there or not, depending on design decision,
    // but Grid specifically must be hidden as per requirements.
    await expect(HOME.sportSelection.grid).toBeHidden();
  });
});
