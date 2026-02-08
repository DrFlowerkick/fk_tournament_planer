import type { Page } from "@playwright/test";

export const POSTAL_IDS = {
  list: {
    root: "postal-address-list-root",
    filterName: "filter-name-search",
    filterLimit: "filter-limit-select",
    emptyList: "postal-address-list-empty",
    table: "postal-address-table",
    btnNew: "action-btn-new",
    btnEdit: "action-btn-edit",
    rowPrefix: "postal-address-row-",
    previewPrefix: "postal-address-preview-",
    // Reuse existing preview IDs for the inner card content
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
    inputCountry: "select-country",
    btnSave: "btn-save",
    btnSaveAsNew: "btn-save-as-new",
    btnCancel: "btn-cancel",
  },
} as const;

export function getPostalSelectors(page: Page) {
  return {
    list: {
      // 1. If you know the ID
      previewById: (id: string) =>
        page.getByTestId(`${POSTAL_IDS.list.previewPrefix}${id}`),

      // 2. If you know the name (Robust E2E approach)
      // Finds the row containing the name, then finds the preview inside that row
      previewByName: (name: string) =>
        page
          .getByRole("row")
          .filter({ hasText: name })
          .getByTestId(new RegExp(`^${POSTAL_IDS.list.previewPrefix}`)),

      // 3. Just get the first visible one
      anyPreview: page
        .locator(`[data-testid^="${POSTAL_IDS.list.previewPrefix}"]`)
        .first(),
      anyRow: page
        .locator(`[data-testid^="${POSTAL_IDS.list.rowPrefix}"]`)
        .first(),

      root: page.getByTestId(POSTAL_IDS.list.root),
      filterName: page.getByTestId(POSTAL_IDS.list.filterName),
      filterLimit: page.getByTestId(POSTAL_IDS.list.filterLimit),
      emptyList: page.getByTestId(POSTAL_IDS.list.emptyList),
      table: page.getByTestId(POSTAL_IDS.list.table),
      btnNew: page.getByTestId(POSTAL_IDS.list.btnNew),
      btnEdit: page.getByTestId(POSTAL_IDS.list.btnEdit),
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
