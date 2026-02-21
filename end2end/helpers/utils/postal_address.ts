// Shared helpers for address form flows
import { expect, Locator, Page } from "@playwright/test";
import {
  fillAndBlur,
  selectThenBlur,
  extractQueryParamFromUrl,
  waitForAppHydration,
  IDS,
  selectors,
} from "../../helpers";

const UUID_REGEX =
  /^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/;

export const PA_ROUTES = {
  newAddress: "/postal-address/new",
  editAddress: "/postal-address/edit",
  copyAddress: "/postal-address/copy",
  list: "/postal-address",
};

export const PA_QUERY_KEYS = {
  addressId: "address_id",
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
 * @param page Playwright Page object
 * @param shouldHaveId If true (default), expects a valid address_id in query. If false, expects NO address_id.
 */
export async function waitForPostalAddressListUrl(
  page: Page,
  shouldHaveId: boolean,
) {
  const PA = selectors(page).postalAddress;
  // Wait for URL path /postal-address and address_id query param according to option
  await page.waitForURL((url) => {
    const isCorrectPath = url.pathname === PA_ROUTES.list;
    const addressId = url.searchParams.get(PA_QUERY_KEYS.addressId);

    if (shouldHaveId) {
      return isCorrectPath && !!addressId && UUID_REGEX.test(addressId);
    } else {
      return isCorrectPath && !addressId;
    }
  });

  // strict hydration check
  await waitForAppHydration(page);

  await expect(PA.list.filterName).toBeVisible();
  await expect(PA.list.btnNew).toBeVisible();
}

/**
 * Extracts the UUID from a /postal-address/<uuid> URL.
 */
export function extractUuidFromUrl(url: string): string {
  const value = extractQueryParamFromUrl(url, PA_QUERY_KEYS.addressId);
  if (!value)
    throw new Error(`No value for key "address_id" found in URL: ${url}`);
  return value;
}

/**
 * Enter new mode from a detail page (if you have a dedicated new button).
 * Assumes you're already on a page with the new button visible (e.g., after clicking on row from the list).
 */
export async function clickNewPostalAddress(page: Page) {
  const PA = selectors(page).postalAddress;
  await expect(PA.list.btnNew).toBeVisible();
  await PA.list.btnNew.click();
  // Assert the form is shown again
  await waitForPostalAddressNewUrl(page);
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
 * Enter copy mode from a detail page (if you have a dedicated copy button).
 * Assumes you're already on a page with the copy button visible (e.g., after clicking on row from the list).
 */
export async function clickCopyPostalAddress(page: Page) {
  const PA = selectors(page).postalAddress;
  await expect(PA.list.btnCopy).toBeVisible();
  await PA.list.btnCopy.click();
  // Assert the form is shown again
  await waitForPostalAddressCopyUrl(page);
}

/**
 * Wait for navigation to create a new postal address page (UUID URL).
 */
export async function waitForPostalAddressNewUrl(page: Page) {
  const PA = selectors(page).postalAddress;
  // Wait for URL path /postal-address/new and valid address_id query param
  await page.waitForURL((url) => {
    const isCorrectPath = url.pathname === PA_ROUTES.newAddress;
    const addressId = url.searchParams.get(PA_QUERY_KEYS.addressId);
    return isCorrectPath && !!addressId && UUID_REGEX.test(addressId);
  });

  // strict hydration check
  await waitForAppHydration(page);

  await expect(PA.form.root).toBeVisible();
}

/**
 * Wait for navigation to a edit postal address page (UUID URL).
 */
export async function waitForPostalAddressEditUrl(page: Page) {
  const PA = selectors(page).postalAddress;
  // Wait for URL path /postal-address/edit and valid address_id query param
  await page.waitForURL((url) => {
    const isCorrectPath = url.pathname === PA_ROUTES.editAddress;
    const addressId = url.searchParams.get(PA_QUERY_KEYS.addressId);
    return isCorrectPath && !!addressId && UUID_REGEX.test(addressId);
  });

  // strict hydration check
  await waitForAppHydration(page);

  await expect(PA.form.root).toBeVisible();
}

/**
 * Wait for navigation to a copy postal address page (UUID URL).
 */
export async function waitForPostalAddressCopyUrl(page: Page) {
  const PA = selectors(page).postalAddress;
  // Wait for URL path /postal-address/copy and valid address_id query param
  await page.waitForURL((url) => {
    const isCorrectPath = url.pathname === PA_ROUTES.copyAddress;
    const addressId = url.searchParams.get(PA_QUERY_KEYS.addressId);
    return isCorrectPath && !!addressId && UUID_REGEX.test(addressId);
  });

  // strict hydration check
  await waitForAppHydration(page);

  await expect(PA.form.root).toBeVisible();
}

/**
 * Close the form by clicking the "Close" button (if available).
 */
export async function closeForm(page: Page) {
  const PA = selectors(page).postalAddress;
  await expect(PA.form.btnClose).toBeVisible();
  await PA.form.btnClose.click();
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
  // we start with region, since it is not part of validation
  if (fields.region !== undefined) {
    await fillAndBlur(PA.form.inputRegion, fields.region);
  }

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

// mapping of countries used in tests
const COUNTRY_CODE_TO_NAME: Record<string, string> = {
  DE: "Germany (DE)",
  US: "United States (US)",
  FR: "France (FR)",
  // add more as needed
};

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
  page: Page,
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
  const LIST_PREVIEW = IDS.list.detailedPreview;
  const PA_PREVIEW = IDS.postalAddress.list.preview;
  // check preview fields
  await expect(page.getByTestId(LIST_PREVIEW)).toBeVisible();

  if (expected.street !== undefined) {
    await expect(page.getByTestId(PA_PREVIEW.street)).toHaveText(
      expected.street!,
    );
  }

  if (expected.postal_code !== undefined) {
    await expect(page.getByTestId(PA_PREVIEW.postalCode)).toHaveText(
      expected.postal_code!,
    );
  }

  if (expected.locality !== undefined) {
    await expect(page.getByTestId(PA_PREVIEW.locality)).toHaveText(
      expected.locality!,
    );
  }

  if (expected.region !== undefined) {
    await expect(page.getByTestId(PA_PREVIEW.region)).toHaveText(
      expected.region!,
    );
  }

  if (expected.country !== undefined) {
    const expectedText = resolveExpectedPreviewText(
      "country",
      expected.country,
    );
    await expect(page.getByTestId(PA_PREVIEW.country)).toHaveText(expectedText);
  }
}
