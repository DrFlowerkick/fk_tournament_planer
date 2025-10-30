// Shared helpers for address form flows
import { expect, Page } from "@playwright/test";
import { T } from "./selectors";

export const ROUTES = {
  newAddress: "/postal-address/new",
  list: "/postal-address",
};

/**
 * Open the "Postal Address List".
 */
export async function openPostalAddressList(page: Page) {
  // Navigate to "list" route and assert elements exist
  await page.goto(ROUTES.list);
  await page.waitForLoadState('domcontentloaded');
  await expect(page.getByTestId(T.search.input)).toBeVisible();
  await expect(page.getByTestId(T.search.btnNew)).toBeVisible();
  await expect(page.getByTestId(T.search.btnModify)).toBeVisible();
  await expect(page.getByTestId(T.search.btnModify)).toBeDisabled();
}

/**
 * Wait for navigation to a postal address detail page (UUID URL).
 */
export async function waitForPostalAddressListUrl(page: Page) {
  await page.waitForURL(/\/postal-address\/[0-9a-f-]{36}$/);
  await page.waitForLoadState('domcontentloaded');
  await expect(page.getByTestId(T.search.input)).toBeVisible();
  await expect(page.getByTestId(T.search.btnNew)).toBeVisible();
  await expect(page.getByTestId(T.search.btnModify)).toBeVisible();
  await expect(page.getByTestId(T.search.btnModify)).toBeEnabled();
}

/**
 * Extracts the UUID from a /postal-address/<uuid> URL.
 */
export function extractUuidFromUrl(url: string): string {
  const m = url.match(/\/postal-address\/([0-9a-f-]{36})(?:$|\/)/i);
  if (!m) throw new Error(`No UUID found in URL: ${url}`);
  return m[1];
}

/**
 * Open the "New Postal Address" form directly.
 */
export async function openNewForm(page: Page) {
  // Navigate to "new" route and assert the form exists
  await page.goto(ROUTES.newAddress);
  await page.waitForLoadState('domcontentloaded');
  await expect(page.getByTestId(T.form.root)).toBeVisible();
  await expect(page.getByTestId(T.form.btnSave)).toBeVisible();
  await expect(page.getByTestId(T.form.btnSaveAsNew)).toBeHidden();
}

/**
 * Enter modify mode from a detail page (if you have a dedicated edit button).
 */
export async function openModifyForm(page: Page) {
  await expect(page.getByTestId(T.search.btnModify)).toBeVisible();
  await page.getByTestId(T.search.btnModify).click();
  // Assert the form is shown again
  await waitForPostalAddressEditUrl(page);
}

/**
 * Wait for navigation to a edit postal address page (UUID URL).
 */
export async function waitForPostalAddressEditUrl(page: Page) {
  await page.waitForURL(/\/postal-address\/[0-9a-f-]{36}\/edit$/);
  await page.waitForLoadState('domcontentloaded');
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
 * Type a value into a field, then blur by focusing another field.
 * Simulates: focus → type → blur -> normalize -> validate for that field.
 */
export async function typeThenBlur(
  page: Page,
  inputTid: string,
  value: string,
  blurToTid: string
) {
  await expect(page.getByTestId(inputTid)).toBeVisible();
  await page.getByTestId(inputTid).fill(value);
  await page.getByTestId(blurToTid).focus();
}

/**
 * Assert a field's normalized value and validation state using aria-invalid.
 */
export async function expectFieldValidity(
  page: Page,
  inputTid: string,
  expectedValue: string,
  isInvalid: boolean
) {
  const input = page.getByTestId(inputTid);
  await expect(input).toHaveValue(expectedValue);
  if (isInvalid) {
    await expect(input).toHaveAttribute("aria-invalid", "true");
  } else {
    const ariaInvalid = await input.getAttribute("aria-invalid");
    expect(ariaInvalid === null || ariaInvalid === "false").toBeTruthy();
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
    await typeThenBlur(
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
  await fillFields(
    page,
    {
      name,
      street: "Beispielstr. 1",
      postal_code: "10115",
      locality: "Berlin Mitte",
      region: "",
      country: "DE"
    }
  );
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

/**
 * Search on the *current page* and open the unique match.
 * - Does not navigate.
 * - Optionally clears the input before typing.
 * - If your dropdown uses aria-busy, we wait for it to be "false".
 */
export async function searchAndOpenByNameOnCurrentPage(
  page: Page,
  term: string,
  opts: {
    clearFirst?: boolean;
    expectUnique?: boolean;
    waitAriaBusy?: boolean;
  } = {}
) {
  const { clearFirst = true, expectUnique = true, waitAriaBusy = true } = opts;

  const input = page.getByTestId(T.search.input);
  const list = page.getByTestId(T.search.suggestList);
  const items = page.getByTestId(T.search.suggestItem);

  await expect(input).toBeVisible();

  // Clear input if requested
  if (clearFirst) {
    await input.fill("");
  }

  // Type the search term
  await input.fill(term);

  // Ensure list is present
  await expect(list).toBeAttached();

  // If your dropdown marks loading via aria-busy, wait until it's finished
  if (waitAriaBusy) {
    await expect(list).toHaveAttribute("aria-busy", "false");
  }

  // Filter rows by visible text
  const row = items.filter({ hasText: term });

  // Option A: enforce uniqueness (assert exactly one)
  if (expectUnique) {
    await expect(row.first()).toBeVisible();
    await expect(row).toHaveCount(1);
    await row.first().click();
    return;
  }

  // Option B: just take the first visible match
  await expect(row.first()).toBeVisible();
  await row.first().click();
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
    await expect(page.getByTestId(T.search.preview.country)).toHaveText(
      expected.country!
    );
  }
}
