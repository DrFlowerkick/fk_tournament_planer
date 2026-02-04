import { expect, Page } from "@playwright/test";
import { runSportConfigSharedTests, SportConfigTestAdapter } from "./shared";
import { fillAndBlur, selectors } from "../../helpers";

const ddcSportAdapter: SportConfigTestAdapter = {
  sportName: "Double Disc Court (DDC)",
  generateData: () => {
    const random = Math.floor(Math.random() * 100);
    return {
      sets_to_win: 2 + (random % 3), // 2-4
      score_to_win: 11 + (random % 10), // 11-20
      win_by_margin: 2,
      hard_cap: 25 + (random % 5), // 25-29
      victory_points_win: 3,
      victory_points_draw: 1,
      expected_rally_duration_seconds: 10 + (random % 20), // 10-29
    };
  },
  fillSpecificFields: async (page: Page, data: any) => {
    const SC = selectors(page).sportConfig;
    const blurTarget = SC.form.btnSave;

    // Select Custom Sets to Win
    await page.getByTestId("select-sets_cfg").selectOption("Custom: Sets to Win");
    await fillAndBlur(
      page.getByTestId("input-num_sets"),
      data.sets_to_win.toString(),
    );
    // Select Custom Set Winning Configuration
    await page.getByTestId("select-set_winning_cfg").selectOption("Custom Set Winning Configuration");
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
      page.getByTestId("input-expected_rally_duration_seconds"),
      data.expected_rally_duration_seconds.toString(),
    );
  },
  assertSpecificFields: async (page: Page, data: any) => {
    // Check preview
    await expect(page.getByTestId("preview-set-config")).toContainText(
      `Sets to Win: ${data.sets_to_win}`
    );
    await expect(page.getByTestId("preview-set-winning-config")).toContainText(
      `Score to Win: ${data.score_to_win}, Hard Cap: ${data.hard_cap}, Win by Margin: ${data.win_by_margin}`
    );
    await expect(page.getByTestId("preview-victory-points")).toContainText(
      `Victory Points - Win: ${data.victory_points_win}, Draw: ${data.victory_points_draw}`
    );

    // Calculate expected match duration
    // sets_to_play = sets_to_win * 2 - 1 (for CustomSetsToWin)
    const max_sets = data.sets_to_win * 2 - 1;
    // max_num_rallies = score_to_win + (score_to_win - win_by_margin)
    const max_rallies_per_set =
      data.score_to_win + (data.score_to_win - data.win_by_margin);
    const total_seconds =
      data.expected_rally_duration_seconds * max_sets * max_rallies_per_set;
    const expected_minutes = Math.floor(total_seconds / 60);

    await expect(page.getByTestId("preview-expected-duration")).toContainText(
      `Expected Match Duration: ${expected_minutes} minutes`
    );
  },
};

runSportConfigSharedTests(ddcSportAdapter);
