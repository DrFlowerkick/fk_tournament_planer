import type { Page } from "@playwright/test";
import {
  getBannerSelectors,
  BANNER_IDS,
  getToastSelectors,
  TOAST_IDS,
  getListSelectors,
  LIST_IDS,
} from "./common";
import { getPostalSelectors, POSTAL_IDS } from "./postalAddress";
import { getSportSelectors, SPORT_IDS } from "./sportConfig";
import { getHomeSelectors, HOME_IDS } from "./home";

// Re-exporting IDS for backward compatibility
export const IDS = {
  postalAddress: POSTAL_IDS,
  sportConfig: SPORT_IDS,
  banners: BANNER_IDS,
  toasts: TOAST_IDS,
  home: HOME_IDS,
  list: LIST_IDS,
} as const;

export function selectors(page: Page) {
  return {
    postalAddress: getPostalSelectors(page),
    sportConfig: getSportSelectors(page),
    banners: getBannerSelectors(page),
    toasts: getToastSelectors(page),
    home: getHomeSelectors(page),
    list: getListSelectors(page),
  };
}
