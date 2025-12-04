import type { Page } from "@playwright/test";
import { getDropdown } from "./common";

export const SPORT_IDS = {
  pluginSelect: {
    input: "sport_id-search-input",
    suggestList: "sport_id-search-suggest",
    suggestItem: "sport_id-search-suggest-item",
  },
} as const;

export function getSportSelectors(page: Page) {
  return {
    pluginSelector: getDropdown(page, SPORT_IDS.pluginSelect),
  };
}
