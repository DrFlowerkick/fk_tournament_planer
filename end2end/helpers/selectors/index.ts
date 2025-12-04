import type { Page } from "@playwright/test";
import { getBannerSelectors, BANNER_IDS } from "./common";
import { getPostalSelectors, POSTAL_IDS } from "./postalAddress";
import { getSportSelectors, SPORT_IDS } from "./sportConfig";

export type { DropdownLocators } from "./common";

// Re-exporting T for backward compatibility
export const T = {
  postalAddress: POSTAL_IDS,
  sportConfig: SPORT_IDS,
  banners: BANNER_IDS,
} as const;

export function selectors(page: Page) {
  return {
    postalAddress: getPostalSelectors(page),
    sportConfig: getSportSelectors(page),
    banners: getBannerSelectors(page),
  };
}
