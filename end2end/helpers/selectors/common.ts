import type { Page, Locator } from "@playwright/test";


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

export const LIST_IDS = {
  filterName: "filter-name-search",
  table: "table-list",
  header: "table-list-header",
  btnNew: "action-btn-new",
  btnEdit: "action-btn-edit",
  entryNamePrefix: "table-entry-name-",
  entryPreviewPrefix: "table-entry-preview-",
  detailedPreview: "table-entry-detailed-preview",
} as const;

export function getListSelectors(page: Page) {
  const ids = LIST_IDS;
  return {
    filterName: page.getByTestId(ids.filterName),
    table: page.getByTestId(ids.table),
    header: page.getByTestId(ids.header),
    btnNew: page.getByTestId(ids.btnNew),
    btnEdit: page.getByTestId(ids.btnEdit),
    detailedPreview: page.getByTestId(ids.detailedPreview),
    entryName: (id: string) => page.getByTestId(`${ids.entryNamePrefix}${id}`),
    // Dynamic row/preview selectors
    previewById: (id: string) => page.getByTestId(`${ids.entryPreviewPrefix}${id}`),
    previewByName: (name: string) =>
      page
        .getByRole("row")
        .filter({ hasText: name })
        .getByTestId(new RegExp(`^${ids.entryPreviewPrefix}`)),
    anyPreview: page.locator(`[data-testid^="${ids.entryPreviewPrefix}"]`).first(),
    anyRow: page.locator(`tr:has([data-testid^="${ids.entryNamePrefix}"])`).first(),
  };
}
