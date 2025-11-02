// e2e/helpers/selectors.ts
import type { Page, Locator } from "@playwright/test";

/**
 * Central registry of data-testid values aligned with your current components.
 * (search.rs, edit.rs)
 */
export const T = {
  // search.rs
  search: {
    root: "search-address", // optional wrapper (div/section)
    input: "search-input", // <input type="text" ...> (query)
    suggestList: "search-suggest", // <ul id="addr-suggest" ...>
    suggestItem: "search-suggest-item", // each <li> entry
    // The right-hand "preview" (current selected/loaded address)
    preview: {
      root: "address-preview",
      id: "preview-id",
      version: "preview-version",
      name: "preview-name",
      street: "preview-street",
      postalLocality: "preview-postal_locality", // "10555 Berlin"
      postalCode: "preview-postal_code",
      locality: "preview-locality",
      region: "preview-region",
      country: "preview-country",
    },
    // Actions
    btnNew: "btn-new-address", // <a href="/postal-address/new">New</a>
    btnModify: "btn-modify-address", // <button>Modify</button>
  },

  // edit.rs (AddressForm)
  form: {
    root: "form-address", // <ActionForm ...>
    hiddenId: "hidden-id", // <input type="hidden" name="id
    hiddenVersion: "hidden-version", // <input type="hidden" name="version"
    inputName: "input-name", // name="name"
    inputStreet: "input-street", // name="street"
    inputPostalCode: "input-postal_code", // name="postal_code"
    inputLocality: "input-locality", // name="locality"
    inputRegion: "input-region", // name="region"
    inputCountry: "input-country", // name="country"
    btnSave: "btn-save", // primary update
    btnSaveAsNew: "btn-save-as-new", // value="create"
    btnCancel: "btn-cancel",
  },

  // banners (global)
  banner: {
    acknowledgmentBanner: "acknowledgment-banner",
    btnAcknowledgment: "btn-acknowledgment-action",
    acknowledgmentNavigateBanner: "acknowledgment-navigate-banner",
    btnAcknowledgmentNavigateAction: "btn-acknowledgment-navigate-action",
    btnAcknowledgmentNavigate: "btn-acknowledgment-navigate",
  },
} as const;

export const byTestId = (id: string) => `[data-testid="${id}"]`;

export function selectors(page: Page) {
  const within = (root: Locator) => ({
    search: {
      root,
      input: root.getByTestId(T.search.input),
      suggestList: root.getByTestId(T.search.suggestList),
      suggestItems: root.getByTestId(T.search.suggestItem),
      preview: {
        root: root.getByTestId(T.search.preview.root),
        id: root.getByTestId(T.search.preview.id),
        version: root.getByTestId(T.search.preview.version),
        name: root.getByTestId(T.search.preview.name),
        street: root.getByTestId(T.search.preview.street),
        postalLocality: root.getByTestId(T.search.preview.postalLocality),
        region: root.getByTestId(T.search.preview.region),
        country: root.getByTestId(T.search.preview.country),
      },
      btnNew: root.getByTestId(T.search.btnNew),
      btnModify: root.getByTestId(T.search.btnModify),
    },
    form: {
      root,
      hiddenId: root.getByTestId(T.form.hiddenId),
      hiddenVersion: root.getByTestId(T.form.hiddenVersion),
      inputName: root.getByTestId(T.form.inputName),
      inputStreet: root.getByTestId(T.form.inputStreet),
      inputPostalCode: root.getByTestId(T.form.inputPostalCode),
      inputLocality: root.getByTestId(T.form.inputLocality),
      inputRegion: root.getByTestId(T.form.inputRegion),
      inputCountry: root.getByTestId(T.form.inputCountry),
      btnSave: root.getByTestId(T.form.btnSave),
      btnSaveAsNew: root.getByTestId(T.form.btnSaveAsNew),
      btnCancel: root.getByTestId(T.form.btnCancel),
    },
    banner: {
      acknowledgmentBanner: root.getByTestId(T.banner.acknowledgmentBanner),
      btnAcknowledgment: root.getByTestId(T.banner.btnAcknowledgment),
      acknowledgmentNavigateBanner: root.getByTestId(T.banner.acknowledgmentNavigateBanner),
      btnAcknowledgmentNavigateAction: root.getByTestId(T.banner.btnAcknowledgmentNavigateAction),
      btnAcknowledgmentNavigate: root.getByTestId(T.banner.btnAcknowledgmentNavigate),
    },
  });

  return {
    // search.rs (global access)
    search: {
      input: page.getByTestId(T.search.input),
      suggestList: page.getByTestId(T.search.suggestList),
      suggestItems: page.getByTestId(T.search.suggestItem),
      preview: {
        root: page.getByTestId(T.search.preview.root),
        id: page.getByTestId(T.search.preview.id),
        version: page.getByTestId(T.search.preview.version),
        name: page.getByTestId(T.search.preview.name),
        street: page.getByTestId(T.search.preview.street),
        postalLocality: page.getByTestId(T.search.preview.postalLocality),
        region: page.getByTestId(T.search.preview.region),
        country: page.getByTestId(T.search.preview.country),
      },
      btnNew: page.getByTestId(T.search.btnNew),
      btnModify: page.getByTestId(T.search.btnModify),
    },

    // edit.rs
    form: {
      root: page.getByTestId(T.form.root),
      hiddenId: page.getByTestId(T.form.hiddenId),
      hiddenVersion: page.getByTestId(T.form.hiddenVersion),
      inputName: page.getByTestId(T.form.inputName),
      inputStreet: page.getByTestId(T.form.inputStreet),
      inputPostalCode: page.getByTestId(T.form.inputPostalCode),
      inputLocality: page.getByTestId(T.form.inputLocality),
      inputRegion: page.getByTestId(T.form.inputRegion),
      inputCountry: page.getByTestId(T.form.inputCountry),
      btnSave: page.getByTestId(T.form.btnSave),
      btnSaveAsNew: page.getByTestId(T.form.btnSaveAsNew),
      btnCancel: page.getByTestId(T.form.btnCancel),
    },

    // banners (global)
    banner: {
      acknowledgmentBanner: page.getByTestId(T.banner.acknowledgmentBanner),
      btnAcknowledgment: page.getByTestId(T.banner.btnAcknowledgment),
      acknowledgmentNavigateBanner: page.getByTestId(T.banner.acknowledgmentNavigateBanner),
      btnAcknowledgmentNavigateAction: page.getByTestId(T.banner.btnAcknowledgmentNavigateAction),
      btnAcknowledgmentNavigate: page.getByTestId(T.banner.btnAcknowledgmentNavigate),
    },

    // scopers
    within,
    withinSearch: () => within(page.locator("body")).search,
    withinForm: () => within(page.locator("body")).form,
  };
}
