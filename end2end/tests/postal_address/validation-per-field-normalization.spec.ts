import { test, expect, Page } from "@playwright/test";
import {
  openNewForm,
  expectSavesDisabled,
  expectSavesEnabled,
  fillAllRequiredValid,
  fillAndBlur,
  selectThenBlur,
  expectFieldValidity,
  selectors,
} from "../../helpers";

const NEW_ROUTE = "/postal-address/new";

test.describe("Per-field normalization → validation + gated save", () => {
  test("Initial state: empty form marks required fields invalid and disables save buttons", async ({
    page,
  }) => {
    const PA = selectors(page).postalAddress;

    // -------------------- Arrange & Act --------------------
    await openNewForm(page);

    // All required fields should be invalid at start (empty form).
    await expectFieldValidity(PA.form.inputName, "", /*invalid*/ true);
    await expectFieldValidity(PA.form.inputStreet, "", /*invalid*/ true);
    await expectFieldValidity(PA.form.inputPostalCode, "", /*invalid*/ true);
    await expectFieldValidity(PA.form.inputLocality, "", /*invalid*/ true);
    await expectFieldValidity(PA.form.inputCountry, "", /*invalid*/ true);

    // Save actions must be gated (disabled).
    await expectSavesDisabled(page);
  });

  test("Name: trim & collapse spaces on blur; then validate field", async ({
    page,
  }) => {
    const PA = selectors(page).postalAddress;

    // -------------------- Arrange & Act --------------------
    await openNewForm(page);

    // as long as other required fields are empty/invalid, saving must remain disabled
    await expectSavesDisabled(page);

    // focus → type → blur -> normalize -> validate
    await fillAndBlur(PA.form.inputName, "   Müller   GmbH   ");

    // expect normalized value & validation state (assume name becomes valid after normalization)
    await expectFieldValidity(
      PA.form.inputName,
      "Müller GmbH",
      /*invalid*/ false,
    );

    // as long as other required fields are empty/invalid, saving must remain disabled
    await expectSavesDisabled(page);
  });

  test("Street: trim & collapse spaces on blur; then validate field", async ({
    page,
  }) => {
    const PA = selectors(page).postalAddress;

    // -------------------- Arrange & Act --------------------
    await openNewForm(page);

    // as long as other required fields are empty/invalid, saving must remain disabled
    await expectSavesDisabled(page);

    // focus → type → blur -> normalize -> validate
    await fillAndBlur(
      PA.form.inputStreet,
      "   Main      \n      Street         42  ",
    );
    await expectFieldValidity(
      PA.form.inputStreet,
      "Main Street 42",
      /*invalid*/ false,
    );

    // as long as other required fields are empty/invalid, saving must remain disabled
    await expectSavesDisabled(page);
  });

  test("Locality: trim & collapse spaces on blur; then validate field", async ({
    page,
  }) => {
    const PA = selectors(page).postalAddress;

    // -------------------- Arrange & Act --------------------
    await openNewForm(page);

    // as long as other required fields are empty/invalid, saving must remain disabled
    await expectSavesDisabled(page);

    // focus → type → blur -> normalize -> validate
    await fillAndBlur(PA.form.inputLocality, "   Berlin   Mitte  ");
    await expectFieldValidity(
      PA.form.inputLocality,
      "Berlin Mitte",
      /*invalid*/ false,
    );

    // as long as other required fields are empty/invalid, saving must remain disabled
    await expectSavesDisabled(page);
  });

  test("Country: uppercase on blur; then validate field", async ({ page }) => {
    const PA = selectors(page).postalAddress;

    // -------------------- Arrange & Act --------------------
    await openNewForm(page);

    // as long as other required fields are empty/invalid, saving must remain disabled
    await expectSavesDisabled(page);

    // blur path
    // Note: "uppercase on blur" is no longer relevant for a select field
    // as the values are predefined ISO codes.
    await selectThenBlur(PA.form.inputCountry, "DE");
    await expectFieldValidity(PA.form.inputCountry, "DE", /*invalid*/ false);

    // as long as other required fields are empty/invalid, saving must remain disabled
    await expectSavesDisabled(page);
  });

  test("Postal code (DE-specific rule): strip spaces/non-digits; validate length=5; gate while invalid", async ({
    page,
  }) => {
    const PA = selectors(page).postalAddress;

    // -------------------- Arrange & Act --------------------
    await openNewForm(page);

    // as long as other required fields are empty/invalid, saving must remain disabled
    await expectSavesDisabled(page);

    /**
     * NOTE (DE-specific):
     * The following assertions reflect a *German* postal code rule
     * (exactly 5 digits after normalization). This is NOT a generic
     * rule for all countries. Treat it as an example/default, and
     * adapt to country-specific rules once available.
     */

    // set DE
    await selectThenBlur(PA.form.inputCountry, "DE");

    // Example 1: "   10115    " -> "10115" (valid for DE)
    await fillAndBlur(PA.form.inputPostalCode, "   10115    ");
    await expectFieldValidity(
      PA.form.inputPostalCode,
      "10115",
      /*valid*/ false,
    );

    // Example 2: "  1234   " -> "1234" (invalid after normalization for DE: length != 5)
    await fillAndBlur(PA.form.inputPostalCode, "  1234   ");
    await expectFieldValidity(
      PA.form.inputPostalCode,
      "1234",
      /*invalid*/ true,
    );

    // Example 3: "  1234A   " -> "1234A" (invalid after normalization for DE: must be 5 digits)
    await fillAndBlur(PA.form.inputPostalCode, "  1234A   ");
    await expectFieldValidity(
      PA.form.inputPostalCode,
      "1234A",
      /*invalid*/ true,
    );

    // as long as other required fields are empty/invalid, saving must remain disabled
    await expectSavesDisabled(page);
  });

  test("Postal code (DE-specific rule): set invalid postal_code before country -> first valid, after setting country invalid", async ({
    page,
  }) => {
    const PA = selectors(page).postalAddress;

    // -------------------- Arrange & Act --------------------
    await openNewForm(page);

    // as long as other required fields are empty/invalid, saving must remain disabled
    await expectSavesDisabled(page);

    /**
     * NOTE (DE-specific):
     * The following assertions reflect a *German* postal code rule
     * (exactly 5 digits after normalization). This is NOT a generic
     * rule for all countries. Treat it as an example/default, and
     * adapt to country-specific rules once available.
     */

    // Example 1: "   10115    " -> "10115" (valid for DE)
    await fillAndBlur(PA.form.inputPostalCode, "   1011    ");
    await expectFieldValidity(
      PA.form.inputPostalCode,
      "1011",
      /*first valid*/ false,
    );

    // set DE
    await selectThenBlur(PA.form.inputCountry, "DE");

    await expectFieldValidity(
      PA.form.inputPostalCode,
      "1011",
      /*now invalid*/ true,
    );

    // as long as other required fields are empty/invalid, saving must remain disabled
    await expectSavesDisabled(page);
  });

  test("Entering valid input for all required fields enables save buttons", async ({
    page,
  }) => {
    const PA = selectors(page).postalAddress;

    // -------------------- Arrange & Act --------------------
    await openNewForm(page);

    // Initially: empty form → invalid → save disabled
    await expectSavesDisabled(page);

    // fill all fields with valid values
    const name = `E2E Valid Test Address`;
    await fillAllRequiredValid(page, name);

    // expect all fields are valid (values taken from ..helpers/form.ts)
    await expectFieldValidity(PA.form.inputName, name, false);
    await expectFieldValidity(PA.form.inputStreet, "Beispielstr. 1", false);
    await expectFieldValidity(PA.form.inputCountry, "DE", false);
    await expectFieldValidity(PA.form.inputPostalCode, "10115", false);
    await expectFieldValidity(PA.form.inputLocality, "Berlin Mitte", false);

    // All required fields are now valid → save buttons should be enabled
    await expectSavesEnabled(page);
  });
});
