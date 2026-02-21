import { expect, Page, Locator } from "@playwright/test";
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

    // Select Custom Sets to Win
    // We find the option that starts with "Custom" dynamically
    const setsSelect = page.getByTestId("select-sets_cfg");
    const customSetsValue = await setsSelect
      .locator("option")
      .filter({ hasText: /^Custom/ })
      .first()
      .getAttribute("value");

    if (customSetsValue) {
      await setsSelect.selectOption(customSetsValue);
    }

    await fillAndBlur(
      page.getByTestId("input-num_sets"),
      data.sets_to_win.toString(),
    );
    // Select Custom Set Winning Configuration
    // We find the option that starts with "Custom" dynamically
    const winningSelect = page.getByTestId("select-set_winning_cfg");
    const customWinningValue = await winningSelect
      .locator("option")
      .filter({ hasText: /^Custom/ })
      .first()
      .getAttribute("value");

    if (customWinningValue) {
      await winningSelect.selectOption(customWinningValue);
    }

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
  assertSpecificFields: async (row: Locator, data: any) => {
    // Check preview
    await expect(row.getByTestId("preview-set-config")).toContainText(
      new RegExp(
        `(Sets to win|Custom sets to win|Custom total sets): ${data.sets_to_win}`,
      ),
    );
    await expect(row.getByTestId("preview-set-winning-config")).toContainText(
      new RegExp(
        `(Score|Custom score): ${data.score_to_win} \\(\\+${data.win_by_margin}, Cap ${data.hard_cap}\\)`,
      ),
    );
    await expect(row.getByTestId("preview-victory-points-win")).toContainText(
      `${data.victory_points_win}`,
    );
    await expect(row.getByTestId("preview-victory-points-draw")).toContainText(
      `${data.victory_points_draw}`,
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

    await expect(row.getByTestId("preview-expected-duration")).toContainText(
      `~${expected_minutes} min`,
    );
  },
};

runSportConfigSharedTests(ddcSportAdapter);
