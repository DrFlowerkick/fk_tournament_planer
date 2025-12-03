// Shared helpers for address form flows
import { expect, Page } from "@playwright/test";
import { T } from "./selectors";
import {
  typeThenBlur,
  selectThenBlur,
  extractQueryParamFromUrl,
} from "./utils";

export const ROUTES = {
  newAddress: "/postal-address/new_pa",
  list: "/postal-address",
};

/**
 * Open the "Postal Address List".
 */
export async function openPostalAddressList(page: Page) {
  // Navigate to "list" route and assert elements exist
  await page.goto(ROUTES.list);
  await page.waitForLoadState("domcontentloaded");
  await expect(page.getByTestId(T.search.dropdown.input)).toBeVisible();
  await expect(page.getByTestId(T.search.btnNew)).toBeVisible();
  await expect(page.getByTestId(T.search.btnEdit)).toBeVisible();
  await expect(page.getByTestId(T.search.btnEdit)).toHaveAttribute("disabled");
}

/**
 * Wait for navigation to a postal address detail page (UUID URL).
 */
export async function waitForPostalAddressListUrl(page: Page) {
  await page.waitForURL(/\/postal-address\?address_id=[0-9a-f-]{36}$/);
  await page.waitForLoadState("domcontentloaded");
  await expect(page.getByTestId(T.search.dropdown.input)).toBeVisible();
  await expect(page.getByTestId(T.search.btnNew)).toBeVisible();
  await expect(page.getByTestId(T.search.btnEdit)).toBeVisible();
  await expect(page.getByTestId(T.search.btnEdit)).not.toHaveAttribute(
    "disabled"
  );
}

/**
 * Extracts the UUID from a /postal-address/<uuid> URL.
 */
export function extractUuidFromUrl(url: string): string {
  return extractQueryParamFromUrl(url, "address_id");
}

/**
 * Open the "New Postal Address" form directly.
 */
export async function openNewForm(page: Page) {
  // Navigate to "new_pa" route and assert the form exists
  await page.goto(ROUTES.newAddress);
  await page.waitForLoadState("domcontentloaded");
  await expect(page.getByTestId(T.form.root)).toBeVisible();
  await expect(page.getByTestId(T.form.btnSave)).toBeVisible();
  await expect(page.getByTestId(T.form.btnSaveAsNew)).toBeHidden();
}

/**
 * Enter edit mode from a detail page (if you have a dedicated edit button).
 */
export async function clickEditToOpenEditForm(page: Page) {
  await expect(page.getByTestId(T.search.btnEdit)).toBeVisible();
  await page.getByTestId(T.search.btnEdit).click();
  // Assert the form is shown again
  await waitForPostalAddressEditUrl(page);
}

/**
 * Enter edit mode directly by navigating to the edit URL.
 */
export async function openEditForm(page: Page, id: string) {
  await page.goto(`/postal-address/edit_pa?address_id=${id}`);
  // Assert the form is shown again
  await waitForPostalAddressEditUrl(page);
  await page.waitForLoadState("domcontentloaded");
}

/**
 * Wait for navigation to a edit postal address page (UUID URL).
 */
export async function waitForPostalAddressEditUrl(page: Page) {
  await page.waitForURL(/\/postal-address\/edit_pa\?address_id=[0-9a-f-]{36}$/);
  await page.waitForLoadState("domcontentloaded");
  await expect(page.getByTestId(T.form.root)).toBeVisible();
  await expect(page.getByTestId(T.form.btnSave)).toBeVisible();
  await expect(page.getByTestId(T.form.btnSaveAsNew)).toBeVisible();
}

/**
 * Expect that save actions are gated (disabled) while the form is invalid.
 */
export async function expectSavesDisabled(page: Page) {
  await expect(page.getByTestId(T.form.btnSave)).toBeDisabled();
  const save = page.getByTestId(T.form.btnSaveAsNew);
  if (await save.isVisible()) {
    await expect(page.getByTestId(T.form.btnSaveAsNew)).toBeDisabled();
  }
}

/**
 * Expect that save actions are allowed (enabled) when the form is valid.
 */
export async function expectSavesEnabled(page: Page) {
  await expect(page.getByTestId(T.form.btnSave)).toBeEnabled();
  const save = page.getByTestId(T.form.btnSaveAsNew);
  if (await save.isVisible()) {
    await expect(page.getByTestId(T.form.btnSaveAsNew)).toBeEnabled();
  }
}

/**
 * Fill all fields with given values.
 */
export async function fillFields(
  page: Page,
  fields: {
    name?: string;
    street?: string;
    postal_code?: string;
    locality?: string;
    region?: string;
    country?: string;
  }
) {
  // Name
  if (fields.name !== undefined) {
    await typeThenBlur(page, T.form.inputName, fields.name, T.form.inputStreet);
  }

  // Street
  if (fields.street !== undefined) {
    await typeThenBlur(
      page,
      T.form.inputStreet,
      fields.street,
      T.form.inputCountry
    );
  }

  // Country before postal code (for postal code validation)
  if (fields.country !== undefined) {
    await selectThenBlur(
      page,
      T.form.inputCountry,
      fields.country,
      T.form.inputPostalCode
    );
  }

  // Postal code
  if (fields.postal_code !== undefined) {
    await typeThenBlur(
      page,
      T.form.inputPostalCode,
      fields.postal_code,
      T.form.inputLocality
    );
  }

  // Locality
  if (fields.locality !== undefined) {
    await typeThenBlur(
      page,
      T.form.inputLocality,
      fields.locality,
      T.form.inputRegion
    );
  }

  // region
  if (fields.region !== undefined) {
    await typeThenBlur(
      page,
      T.form.inputRegion,
      fields.region,
      T.form.inputName
    );
  }
}

/**
 * Fill all required fields with valid normalized values (DE-specific ZIP example).
 */
export async function fillAllRequiredValid(page: Page, name: string) {
  await fillFields(page, {
    name,
    street: "Beispielstr. 1",
    postal_code: "10115",
    locality: "Berlin Mitte",
    region: "",
    country: "DE",
  });
}

/**
 * Save and expect we leave the form or see some Error message.
 */
export async function clickSave(page: Page) {
  await expectSavesEnabled(page);
  await page.getByTestId(T.form.btnSave).click();
}
/**
 * Save and expect we leave the form or see some Error message.
 */
export async function clickSaveAsNew(page: Page) {
  await expect(page.getByTestId(T.form.btnSaveAsNew)).toBeVisible();
  await expectSavesEnabled(page);
  await page.getByTestId(T.form.btnSaveAsNew).click();
}

// mapping of countries used in tests
const COUNTRY_CODE_TO_NAME: Record<string, string> = {
  DE: "Germany",
  US: "United States",
  FR: "France",
  // add more as needed
};

/**
 * Helper to resolve expected display text from input value.
 * Handles special cases like Country Codes -> Names.
 */
function resolveExpectedPreviewText(
  field: "country" | "other",
  value: string
): string {
  if (field === "country") {
    return COUNTRY_CODE_TO_NAME[value] || value; // Fallback auf Code, falls nicht im Mapping
  }
  return value;
}

/**
 * Assert preview view shows specific field values
 */
export async function expectPreviewShows(
  page: Page,
  expected: {
    name?: string;
    street?: string;
    postal_code?: string;
    locality?: string;
    region?: string;
    country?: string;
  }
) {
  await expect(page.getByTestId(T.search.preview.root)).toBeVisible();

  if (expected.name !== undefined) {
    await expect(page.getByTestId(T.search.preview.name)).toHaveText(
      expected.name!
    );
  }

  if (expected.street !== undefined) {
    await expect(page.getByTestId(T.search.preview.street)).toHaveText(
      expected.street!
    );
  }

  if (expected.postal_code !== undefined) {
    await expect(page.getByTestId(T.search.preview.postalCode)).toHaveText(
      expected.postal_code!
    );
  }

  if (expected.locality !== undefined) {
    await expect(page.getByTestId(T.search.preview.locality)).toHaveText(
      expected.locality!
    );
  }

  if (expected.region !== undefined) {
    await expect(page.getByTestId(T.search.preview.region)).toHaveText(
      expected.region!
    );
  }

  if (expected.country !== undefined) {
    const expectedText = resolveExpectedPreviewText(
      "country",
      expected.country
    );
    await expect(page.getByTestId(T.search.preview.country)).toHaveText(
      expectedText
    );
  }
}
