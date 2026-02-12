import type { Page } from "@playwright/test";
import { getListSelectors } from "./common";

export const SPORT_IDS = {
  list: {
    emptyList: "sport-configs-list-empty",
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
      ...getListSelectors(page),
      emptyList: page.getByTestId(SPORT_IDS.list.emptyList),
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
