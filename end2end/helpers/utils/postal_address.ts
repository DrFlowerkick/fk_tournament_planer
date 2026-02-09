// Shared helpers for address form flows
import { expect, Locator, Page } from "@playwright/test";
import {
  fillAndBlur,
  selectThenBlur,
  extractQueryParamFromUrl,
  waitForAppHydration,
  selectors,
} from "../../helpers";
import { POSTAL_IDS } from "../selectors/postalAddress";

export const PA_ROUTES = {
  newAddress: "/postal-address/new",
  list: "/postal-address",
};

/**
 * Open the "Postal Address List".
 */
export async function openPostalAddressList(page: Page) {
  const PA = selectors(page).postalAddress;
  // Navigate to "list" route and assert elements exist
  await page.goto(PA_ROUTES.list);

  // strict hydration check
  await waitForAppHydration(page);

  await expect(PA.list.filterName).toBeVisible();
  await expect(PA.list.btnNew).toBeVisible();
}

/**
 * Wait for navigation to a postal address detail page (UUID URL).
 */
export async function waitForPostalAddressListUrl(page: Page) {
  const PA = selectors(page).postalAddress;
  // Wait for URL like /postal-address?address_id=UUID
  await page.waitForURL(/\/postal-address\?address_id=[0-9a-f-]{36}$/);

  // strict hydration check
  await waitForAppHydration(page);

  await expect(PA.list.filterName).toBeVisible();
  await expect(PA.list.btnNew).toBeVisible();
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
  const PA = selectors(page).postalAddress;
  // Navigate to "new_pa" route and assert the form exists
  await page.goto(PA_ROUTES.newAddress);

  // strict hydration check
  await waitForAppHydration(page);

  await expect(PA.form.root).toBeVisible();
  await expect(PA.form.btnSave).toBeVisible();
  await expect(PA.form.btnSaveAsNew).toBeHidden();
}

/**
 * Enter edit mode from a detail page (if you have a dedicated edit button).
 * Assumes you're already on a page with the edit button visible (e.g., after clicking on row from the list).
 */
export async function clickEditPostalAddress(page: Page) {
  const PA = selectors(page).postalAddress;
  await expect(PA.list.btnEdit).toBeVisible();
  await PA.list.btnEdit.click();
  // Assert the form is shown again
  await waitForPostalAddressEditUrl(page);
}

/**
 * Enter edit mode directly by navigating to the edit URL.
 */
export async function openEditForm(page: Page, id: string) {
  await page.goto(`/postal-address/edit?address_id=${id}`);
  // Assert the form is shown again

  // Note: waitForPostalAddressEditUrl internally waits for URL AND hydration
  // so we don't need an explicit wait here, but the original code had it.
  // The function call below handles the timing.
  await waitForPostalAddressEditUrl(page);
}

/**
 * Wait for navigation to a edit postal address page (UUID URL).
 */
export async function waitForPostalAddressEditUrl(page: Page) {
  const PA = selectors(page).postalAddress;
  // Wait for URL like /postal-address/edit?address_id=UUID
  await page.waitForURL(/\/postal-address\/edit\?address_id=[0-9a-f-]{36}$/);

  // strict hydration check
  await waitForAppHydration(page);

  await expect(PA.form.root).toBeVisible();
  await expect(PA.form.btnSave).toBeVisible();
  await expect(PA.form.btnSaveAsNew).toBeVisible();
}

/**
 * Expect that save actions are gated (disabled) while the form is invalid.
 */
export async function expectSavesDisabled(page: Page) {
  const PA = selectors(page).postalAddress;
  await expect(PA.form.btnSave).toBeDisabled();
  if (await PA.form.btnSaveAsNew.isVisible()) {
    await expect(PA.form.btnSaveAsNew).toBeDisabled();
  }
}

/**
 * Expect that save actions are allowed (enabled) when the form is valid.
 */
export async function expectSavesEnabled(page: Page) {
  const PA = selectors(page).postalAddress;
  await expect(PA.form.btnSave).toBeEnabled();
  if (await PA.form.btnSaveAsNew.isVisible()) {
    await expect(PA.form.btnSaveAsNew).toBeEnabled();
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
  },
) {
  const PA = selectors(page).postalAddress;
  // Name
  if (fields.name !== undefined) {
    await fillAndBlur(PA.form.inputName, fields.name);
  }

  // Street
  if (fields.street !== undefined) {
    await fillAndBlur(PA.form.inputStreet, fields.street);
  }

  // Country before postal code (for postal code validation)
  if (fields.country !== undefined) {
    await selectThenBlur(PA.form.inputCountry, fields.country);
  }

  // Postal code
  if (fields.postal_code !== undefined) {
    await fillAndBlur(PA.form.inputPostalCode, fields.postal_code);
  }

  // Locality
  if (fields.locality !== undefined) {
    await fillAndBlur(PA.form.inputLocality, fields.locality);
  }

  // region
  if (fields.region !== undefined) {
    await fillAndBlur(PA.form.inputRegion, fields.region);
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
  const PA = selectors(page).postalAddress;
  await expectSavesEnabled(page);
  await PA.form.btnSave.click();
}
/**
 * Save and expect we leave the form or see some Error message.
 */
export async function clickSaveAsNew(page: Page) {
  const PA = selectors(page).postalAddress;
  await expect(PA.form.btnSaveAsNew).toBeVisible();
  await expectSavesEnabled(page);
  await PA.form.btnSaveAsNew.click();
}

// mapping of countries used in tests
const COUNTRY_CODE_TO_NAME: Record<string, string> = {
  DE: "Germany (DE)",
  US: "United States (US)",
  FR: "France (FR)",
  // add more as needed
};

/**
 * Helper to filter address list by name and to select row with given name.
 * Assumes you're already on the list page.
 * Returns the row locator for further assertions.
 */
export async function searchAndOpenByNameOnCurrentPage(page: Page, name: string): Promise<Locator> {
  const PA = selectors(page).postalAddress;
  await expect(PA.list.filterName).toBeVisible();
  await PA.list.filterName.fill(name);
  // Assuming the list updates reactively, we can now select the row.
  const row = PA.list.previewByName(name);
  await expect(row).toBeVisible();
  await row.click();
  await expect(PA.list.btnEdit).toBeVisible();
  return row;
}

/**
 * Helper to resolve expected display text from input value.
 * Handles special cases like Country Codes -> Names.
 */
function resolveExpectedPreviewText(
  field: "country" | "other",
  value: string,
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
  row: Locator,
  expected: {
    name?: string;
    street?: string;
    postal_code?: string;
    locality?: string;
    region?: string;
    country?: string;
  },
) {
  // "preview-" prefix locator for fields inside the preview component
  const PA_PREVIEW = POSTAL_IDS.list.preview;
  // check preview fields
  await expect(row.getByTestId(PA_PREVIEW.root)).toBeVisible();

  if (expected.name !== undefined) {
    await expect(row.getByTestId(PA_PREVIEW.name)).toHaveText(expected.name!);
  }

  if (expected.street !== undefined) {
    await expect(row.getByTestId(PA_PREVIEW.street)).toHaveText(expected.street!);
  }

  if (expected.postal_code !== undefined) {
    await expect(row.getByTestId(PA_PREVIEW.postalCode)).toHaveText(
      expected.postal_code!,
    );
  }

  if (expected.locality !== undefined) {
    await expect(row.getByTestId(PA_PREVIEW.locality)).toHaveText(expected.locality!);
  }

  if (expected.region !== undefined) {
    await expect(row.getByTestId(PA_PREVIEW.region)).toHaveText(expected.region!);
  }

  if (expected.country !== undefined) {
    const expectedText = resolveExpectedPreviewText(
      "country",
      expected.country,
    );
    await expect(row.getByTestId(PA_PREVIEW.country)).toHaveText(expectedText);
  }
}
