import type { Page } from "@playwright/test";
import { getDropdown } from "./common";

export const SPORT_IDS = {
  pluginSelect: {
    input: "sport_id-search-input",
    suggestList: "sport_id-search-suggest",
    suggestItem: "sport_id-search-suggest-item",
  },
  search: {
    input: "sport_config_id-search-input",
    suggestList: "sport_config_id-search-suggest",
    suggestItem: "sport_config_id-search-suggest-item",
    btnNew: "btn-new-sport-config",
    btnEdit: "btn-edit-sport-config",
    preview: "sport-config-preview",
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
    pluginSelector: getDropdown(page, SPORT_IDS.pluginSelect),
    search: {
      dropdown: getDropdown(page, SPORT_IDS.search),
      preview: page.getByTestId(SPORT_IDS.search.preview),
      btnNew: page.getByTestId(SPORT_IDS.search.btnNew),
      btnEdit: page.getByTestId(SPORT_IDS.search.btnEdit),
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
