import type { Page, Locator } from "@playwright/test";

export interface DropdownLocators {
  input: Locator;
  list: Locator;
  items: Locator;
}

export function getDropdown(
  page: Page,
  ids: { input: string; suggestList: string; suggestItem: string }
): DropdownLocators {
  return {
    input: page.getByTestId(ids.input),
    list: page.getByTestId(ids.suggestList),
    items: page.getByTestId(ids.suggestItem),
  };
}

export const BANNER_IDS = {
  acknowledgment: "acknowledgment-banner",
  btnAck: "btn-acknowledgment-action",
  ackNavigate: "acknowledgment-navigate-banner",
  btnAckNavAction: "btn-acknowledgment-navigate-action",
  btnAckNav: "btn-acknowledgment-navigate",
} as const;

export function getBannerSelectors(page: Page) {
  return {
    acknowledgment: {
      root: page.getByTestId(BANNER_IDS.acknowledgment),
      btnAction: page.getByTestId(BANNER_IDS.btnAck),
    },
    acknowledgmentNavigate: {
      root: page.getByTestId(BANNER_IDS.ackNavigate),
      btnAction: page.getByTestId(BANNER_IDS.btnAckNavAction),
      btnNavigate: page.getByTestId(BANNER_IDS.btnAckNav),
    },
  };
}
