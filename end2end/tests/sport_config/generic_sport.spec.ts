import { expect, Page, Locator } from "@playwright/test";
import { runSportConfigSharedTests, SportConfigTestAdapter } from "./shared";
import { fillAndBlur, selectors } from "../../helpers";

const genericSportAdapter: SportConfigTestAdapter = {
  sportName: "Generic Sport",
  generateData: () => {
    // Use random values to ensure we are testing updates correctly
    const random = Math.floor(Math.random() * 100);
    return {
      sets_to_win: 2 + (random % 3), // 2-4
      score_to_win: 11 + (random % 10), // 11-20
      win_by_margin: 2,
      hard_cap: 25 + (random % 5), // 25-29
      victory_points_win: 3,
      victory_points_draw: 1,
      expected_match_duration_minutes: 30 + (random % 30), // 30-59
    };
  },
  fillSpecificFields: async (page: Page, data: any) => {
    const SC = selectors(page).sportConfig;
    // We use the save button as a blur target to trigger validation/normalization
    const blurTarget = SC.form.btnSave;

    await fillAndBlur(
      page.getByTestId("input-sets_to_win"),
      data.sets_to_win.toString(),
    );
    await fillAndBlur(
      page.getByTestId("input-score_to_win"),
      data.score_to_win.toString(),
    );
    await fillAndBlur(
      page.getByTestId("input-win_by_margin"),
      data.win_by_margin.toString(),
    );
    await fillAndBlur(
      page.getByTestId("input-hard_cap"),
      data.hard_cap.toString(),
    );
    await fillAndBlur(
      page.getByTestId("input-victory_points_win"),
      data.victory_points_win.toString(),
    );
    await fillAndBlur(
      page.getByTestId("input-victory_points_draw"),
      data.victory_points_draw.toString(),
    );
    await fillAndBlur(
      page.getByTestId("input-expected_match_duration_minutes"),
      data.expected_match_duration_minutes.toString(),
    );
  },
  assertSpecificFields: async (row: Locator, data: any) => {
    // Check preview
    await expect(row.getByTestId("preview-sets-to-win")).toContainText(
      `Sets to win: ${data.sets_to_win}`
    );
    await expect(row.getByTestId("preview-score-to-win")).toContainText(
      `Score to win a set: ${data.score_to_win}`
    );
    await expect(row.getByTestId("preview-win-by-margin")).toContainText(
      `(win by ${data.win_by_margin})`
    );
    await expect(row.getByTestId("preview-hard-cap")).toContainText(
      `(hard cap ${data.hard_cap})`
    );
    await expect(row.getByTestId("preview-victory-points")).toContainText(
      `Victory Points - Win: ${data.victory_points_win}, Draw: ${data.victory_points_draw}`
    );
    await expect(row.getByTestId("preview-expected-duration")).toContainText(
      `Expected Match Duration: ${data.expected_match_duration_minutes} minutes`
    );
  },
};

runSportConfigSharedTests(genericSportAdapter);
