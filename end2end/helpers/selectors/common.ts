import type { Page, Locator } from "@playwright/test";

export interface DropdownLocators {
  input: Locator;
  list: Locator;
  items: Locator;
}

export function getDropdown(
  page: Page,
  ids: { input: string; suggestList: string; suggestItem: string },
): DropdownLocators {
  return {
    input: page.getByTestId(ids.input),
    list: page.getByTestId(ids.suggestList),
    items: page.getByTestId(ids.suggestItem),
  };
}

export const BANNER_IDS = {
  globalErrorBanner: "global-error-banner",
  globalErrorBannerMsg: "global-error-banner-msg",
  btnRetry: "btn-retry-action",
  btnCancel: "btn-cancel-action",
} as const;

export function getBannerSelectors(page: Page) {
  return {
    globalErrorBanner: {
      root: page.getByTestId(BANNER_IDS.globalErrorBanner),
      msg: page.getByTestId(BANNER_IDS.globalErrorBannerMsg),
      btnRetry: page.getByTestId(BANNER_IDS.btnRetry),
      btnCancel: page.getByTestId(BANNER_IDS.btnCancel),
    },
  };
}

export const TOAST_IDS = {
  success: "toast-alert-success",
  error: "toast-alert-error",
  info: "toast-alert-info",
  warning: "toast-alert-warning",
} as const;

export function getToastSelectors(page: Page) {
  return {
    success: page.getByTestId(TOAST_IDS.success),
    error: page.getByTestId(TOAST_IDS.error),
    info: page.getByTestId(TOAST_IDS.info),
    warning: page.getByTestId(TOAST_IDS.warning),
  };
}
