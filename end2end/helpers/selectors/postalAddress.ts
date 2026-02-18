import type { Page } from "@playwright/test";
import { getListSelectors } from "./common";

export const POSTAL_IDS = {
  list: {
    root: "postal-address-list-root",
    filterLimit: "filter-limit-select",
    emptyList: "postal-address-list-empty",
    // Reuse existing preview IDs for the inner card content
    preview: {
      id: "preview-address-id",
      version: "preview-address-version",
      street: "preview-street",
      postalLocality: "preview-postal_locality",
      postalCode: "preview-postal_code",
      locality: "preview-locality",
      region: "preview-region",
      country: "preview-country",
    },
  },
  form: {
    root: "form-address",
    hiddenId: "hidden-id",
    hiddenVersion: "hidden-version",
    inputName: "input-name",
    inputStreet: "input-street",
    inputPostalCode: "input-postal_code",
    inputLocality: "input-locality",
    inputRegion: "input-region",
    inputCountry: "select-country",
    btnClose: "action-btn-close",
  },
} as const;

export function getPostalSelectors(page: Page) {
  return {
    list: {
      ...getListSelectors(page),
      root: page.getByTestId(POSTAL_IDS.list.root),
      filterLimit: page.getByTestId(POSTAL_IDS.list.filterLimit),
      emptyList: page.getByTestId(POSTAL_IDS.list.emptyList),
    },
    form: {
      root: page.getByTestId(POSTAL_IDS.form.root),
      hiddenId: page.getByTestId(POSTAL_IDS.form.hiddenId),
      hiddenVersion: page.getByTestId(POSTAL_IDS.form.hiddenVersion),
      inputName: page.getByTestId(POSTAL_IDS.form.inputName),
      inputStreet: page.getByTestId(POSTAL_IDS.form.inputStreet),
      inputPostalCode: page.getByTestId(POSTAL_IDS.form.inputPostalCode),
      inputLocality: page.getByTestId(POSTAL_IDS.form.inputLocality),
      inputRegion: page.getByTestId(POSTAL_IDS.form.inputRegion),
      inputCountry: page.getByTestId(POSTAL_IDS.form.inputCountry),
      btnClose: page.getByTestId(POSTAL_IDS.form.btnClose),
    },
  };
}
