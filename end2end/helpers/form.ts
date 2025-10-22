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
  await expect(page.getByTestId(T.search.input)).toBeVisible();
  await expect(page.getByTestId(T.search.btnNew)).toBeVisible();
  await expect(page.getByTestId(T.search.btnModify)).toBeVisible();
  await expect(page.getByTestId(T.search.btnModify)).toBeDisabled;
}

/**
 * Open the "New Postal Address" form directly.
 */
export async function openNewForm(page: Page) {
  // Navigate to "new" route and assert the form exists
  await page.goto(ROUTES.newAddress);
  await expect(page.getByTestId(T.form.root)).toBeVisible();
  await expect(page.getByTestId(T.form.btnSave)).toBeVisible();
  await expect(page.getByTestId(T.form.btnSaveAsNew)).toBeHidden;
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
export async function fillAll(
  page: Page,
  name: string,
  street: string,
  postal_code: string,
  locality: string,
  region: string,
  country: string
) {
  // Name
  await typeThenBlur(page, T.form.inputName, name, T.form.inputStreet);

  // Street
  await typeThenBlur(page, T.form.inputStreet, street, T.form.inputCountry);

  // Country
  await typeThenBlur(
    page,
    T.form.inputCountry,
    country,
    T.form.inputPostalCode
  );

  // Postal code
  await typeThenBlur(
    page,
    T.form.inputPostalCode,
    postal_code,
    T.form.inputLocality
  );

  // Locality
  await typeThenBlur(page, T.form.inputLocality, locality, T.form.inputRegion);

  // region
  await typeThenBlur(page, T.form.inputRegion, region, T.form.inputName);
}

/**
 * Fill all required fields with valid normalized values (DE-specific ZIP example).
 */
export async function fillAllRequiredValid(page: Page, name: string) {
  await fillAll(
    page,
    name,
    "Beispielstr. 1",
    "10115",
    "Berlin Mitte",
    "",
    "DE"
  );
}

/**
 * Save and expect we leave the form or see a success state (adjust to your app). ToDo: What happens, if save is not successful?
 */
export async function clickSave(page: Page) {
  await expectSavesEnabled(page);
  await page.getByTestId(T.form.btnSave).click();
}
/**
 * Save and expect we leave the form or see a success state (adjust to your app). ToDo: What happens, if save is not successful?
 */
export async function clickSaveModify(page: Page) {
  await expect(page.getByTestId(T.form.btnSaveAsNew)).toBeVisible();
  await expectSavesEnabled(page);
  await page.getByTestId(T.form.btnSaveAsNew).click();
}

/**
 * Ensure we are on the list page with the search input visible.
 */
export async function ensureListVisible(page: Page) {
  const input = page.getByTestId(T.search.input);

  // Try a short, polite wait — maybe we are already on the list.
  try {
    await expect(input).toBeVisible();
    return;
  } catch {
    // Not on list (or not yet rendered) → navigate explicitly.
    await openPostalAddressList(page);
  }
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
 * Enter modify mode from a detail page (if you have a dedicated edit button).
 */
export async function openModifyForm(page: Page) {
  await expect(page.getByTestId(T.search.btnModify)).toBeVisible();
  await page.getByTestId(T.search.btnModify).click();
  // Assert the form is shown again
  await expect(page.getByTestId(T.form.root)).toBeVisible();
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
