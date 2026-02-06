import type { Page } from "@playwright/test";
import { getDropdown } from "./common";

export const SPORT_IDS = {
  list: {
    filterName: "filter-name-search",
    emptyList: "sport-configs-list-empty",
    table: "sport-configs-table",
    btnNew: "action-btn-new",
    btnEdit: "action-btn-edit",
    previewPrefix: "sport-configs-preview-",
    rowPrefix: "sport-configs-row-",
  },
  form: {
    root: "form-sport-config",
    hiddenId: "hidden-id",
    hiddenVersion: "hidden-version",
    inputName: "input-name",
    btnSave: "btn-save",
    btnSaveAsNew: "btn-save-as-new",
    btnCancel: "btn-cancel",
  },
} as const;

export function getSportSelectors(page: Page) {
  return {
    list: {
      // 1. If you know the ID (e.g. from extracts)
      previewById: (id: string) =>
        page.getByTestId(`${SPORT_IDS.list.previewPrefix}${id}`),

      // 2. If you know the name (Robust E2E approach)
      // Finds the row containing the name, then finds the preview inside that row
      previewByName: (name: string) =>
        page
          .getByRole("row")
          .filter({ hasText: name })
          .getByTestId(new RegExp(`^${SPORT_IDS.list.previewPrefix}`)),

      // 3. Just get the first visible one (useful if list is filtered to 1 result)
      anyPreview: page
        .locator(`[data-testid^="${SPORT_IDS.list.previewPrefix}"]`)
        .first(),
      anyRow: page
        .locator(`[data-testid^="${SPORT_IDS.list.rowPrefix}"]`)
        .first(),
      filterName: page.getByTestId(SPORT_IDS.list.filterName),
      emptyList: page.getByTestId(SPORT_IDS.list.emptyList),
      table: page.getByTestId(SPORT_IDS.list.table),
      btnNew: page.getByTestId(SPORT_IDS.list.btnNew),
      btnEdit: page.getByTestId(SPORT_IDS.list.btnEdit),
    },
    form: {
      root: page.getByTestId(SPORT_IDS.form.root),
      hiddenId: page.getByTestId(SPORT_IDS.form.hiddenId),
      hiddenVersion: page.getByTestId(SPORT_IDS.form.hiddenVersion),
      inputName: page.getByTestId(SPORT_IDS.form.inputName),
      btnSave: page.getByTestId(SPORT_IDS.form.btnSave),
      btnSaveAsNew: page.getByTestId(SPORT_IDS.form.btnSaveAsNew),
      btnCancel: page.getByTestId(SPORT_IDS.form.btnCancel),
    },
  };
}
