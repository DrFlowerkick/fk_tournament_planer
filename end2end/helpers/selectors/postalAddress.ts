import type { Page } from "@playwright/test";
import { getDropdown } from "./common";

export const POSTAL_IDS = {
  search: {
    input: "address_id-search-input",
    suggestList: "address_id-search-suggest",
    suggestItem: "address_id-search-suggest-item",
    btnNew: "btn-new-address",
    btnEdit: "btn-edit-address",
    preview: {
      root: "address-preview",
      id: "preview-address-id",
      version: "preview-address-version",
      name: "preview-address-name",
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
    inputCountry: "input-country",
    btnSave: "btn-save",
    btnSaveAsNew: "btn-save-as-new",
    btnCancel: "btn-cancel",
  },
} as const;

export function getPostalSelectors(page: Page) {
  return {
    search: {
      dropdown: getDropdown(page, POSTAL_IDS.search),
      preview: {
        root: page.getByTestId(POSTAL_IDS.search.preview.root),
        id: page.getByTestId(POSTAL_IDS.search.preview.id),
        version: page.getByTestId(POSTAL_IDS.search.preview.version),
        name: page.getByTestId(POSTAL_IDS.search.preview.name),
        street: page.getByTestId(POSTAL_IDS.search.preview.street),
        postalLocality: page.getByTestId(
          POSTAL_IDS.search.preview.postalLocality
        ),
        postalCode: page.getByTestId(POSTAL_IDS.search.preview.postalCode),
        locality: page.getByTestId(POSTAL_IDS.search.preview.locality),
        region: page.getByTestId(POSTAL_IDS.search.preview.region),
        country: page.getByTestId(POSTAL_IDS.search.preview.country),
      },
      btnNew: page.getByTestId(POSTAL_IDS.search.btnNew),
      btnEdit: page.getByTestId(POSTAL_IDS.search.btnEdit),
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
      btnSave: page.getByTestId(POSTAL_IDS.form.btnSave),
      btnSaveAsNew: page.getByTestId(POSTAL_IDS.form.btnSaveAsNew),
      btnCancel: page.getByTestId(POSTAL_IDS.form.btnCancel),
    },
  };
}
