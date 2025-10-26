import { test, expect, Page } from "@playwright/test";
import { T } from "../helpers/selectors";
import {
  openNewForm,
  typeThenBlur,
  expectFieldValidity,
  expectSavesDisabled,
  expectSavesEnabled,
  fillAllRequiredValid,
} from "../helpers/form";

const NEW_ROUTE = "/postal-address/new";

test.describe("Per-field normalization → validation + gated save", () => {
  test("Initial state: empty form marks required fields invalid and disables save buttons", async ({
    page,
  }) => {
    await openNewForm(page);

    // All required fields should be invalid at start (empty form).
    await expectFieldValidity(page, T.form.inputName, "", /*invalid*/ true);
    await expectFieldValidity(page, T.form.inputStreet, "", /*invalid*/ true);
    await expectFieldValidity(
      page,
      T.form.inputPostalCode,
      "",
      /*invalid*/ true
    );
    await expectFieldValidity(page, T.form.inputLocality, "", /*invalid*/ true);
    await expectFieldValidity(page, T.form.inputCountry, "", /*invalid*/ true);

    // Save actions must be gated (disabled).
    await expectSavesDisabled(page);
  });

  test("Name: trim & collapse spaces on blur; then validate field", async ({
    page,
  }) => {
    await openNewForm(page);

    // focus → type → blur -> normalize -> validate
    await typeThenBlur(
      page,
      T.form.inputName,
      "   Müller   GmbH   ",
      T.form.inputStreet
    );

    // expect normalized value & validation state (assume name becomes valid after normalization)
    await expectFieldValidity(
      page,
      T.form.inputName,
      "Müller GmbH",
      /*invalid*/ false
    );

    // as long as other required fields are empty/invalid, saving must remain disabled
    await expectSavesDisabled(page);
  });

  test("Street: trim & collapse spaces on blur; then validate field", async ({
    page,
  }) => {
    await openNewForm(page);

    // focus → type → blur -> normalize -> validate
    await typeThenBlur(
      page,
      T.form.inputStreet,
      "   Main      \n      Street         42  ",
      T.form.inputLocality
    );
    await expectFieldValidity(
      page,
      T.form.inputStreet,
      "Main Street 42",
      /*invalid*/ false
    );

    await expectSavesDisabled(page);
  });

  test("Locality: trim & collapse spaces on blur; then validate field", async ({
    page,
  }) => {
    await openNewForm(page);

    // focus → type → blur -> normalize -> validate
    await typeThenBlur(
      page,
      T.form.inputLocality,
      "   Berlin   Mitte  ",
      T.form.inputCountry
    );
    await expectFieldValidity(
      page,
      T.form.inputLocality,
      "Berlin Mitte",
      /*invalid*/ false
    );

    await expectSavesDisabled(page);
  });

  test("Country: uppercase on blur; then validate field", async ({ page }) => {
    await openNewForm(page);

    // blur path
    await typeThenBlur(page, T.form.inputCountry, "de", T.form.inputStreet);
    await expectFieldValidity(
      page,
      T.form.inputCountry,
      "DE",
      /*invalid*/ false
    );

    await expectSavesDisabled(page);
  });

  test("Postal code (DE-specific rule): strip spaces/non-digits; validate length=5; gate while invalid", async ({
    page,
  }) => {
    await openNewForm(page);

    /**
     * NOTE (DE-specific):
     * The following assertions reflect a *German* postal code rule
     * (exactly 5 digits after normalization). This is NOT a generic
     * rule for all countries. Treat it as an example/default, and
     * adapt to country-specific rules once available.
     */

    // set DE
    await typeThenBlur(page, T.form.inputCountry, "DE", T.form.inputStreet);

    // Example 1: "   10115    " -> "10115" (valid for DE)
    await typeThenBlur(
      page,
      T.form.inputPostalCode,
      "   10115    ",
      T.form.inputStreet
    );
    await expectFieldValidity(
      page,
      T.form.inputPostalCode,
      "10115",
      /*valid*/ false
    );

    // Example 2: "  1234   " -> "1234" (invalid after normalization for DE: length != 5)
    await typeThenBlur(
      page,
      T.form.inputPostalCode,
      "  1234   ",
      T.form.inputStreet
    );
    await expectFieldValidity(
      page,
      T.form.inputPostalCode,
      "1234",
      /*invalid*/ true
    );

    // Example 3: "  1234A   " -> "1234A" (invalid after normalization for DE: must be 5 digits)
    await typeThenBlur(
      page,
      T.form.inputPostalCode,
      "  1234A   ",
      T.form.inputStreet
    );
    await expectFieldValidity(
      page,
      T.form.inputPostalCode,
      "1234A",
      /*invalid*/ true
    );

    // While invalid, save must be gated
    await expectSavesDisabled(page);
  });

  test("Postal code (DE-specific rule): set invalid postal_code before country -> first valid, after setting country invalid", async ({
    page,
  }) => {
    await openNewForm(page);

    /**
     * NOTE (DE-specific):
     * The following assertions reflect a *German* postal code rule
     * (exactly 5 digits after normalization). This is NOT a generic
     * rule for all countries. Treat it as an example/default, and
     * adapt to country-specific rules once available.
     */

    // Example 1: "   10115    " -> "10115" (valid for DE)
    await typeThenBlur(
      page,
      T.form.inputPostalCode,
      "   1011    ",
      T.form.inputStreet
    );
    await expectFieldValidity(
      page,
      T.form.inputPostalCode,
      "1011",
      /*first valid*/ false
    );

    // set DE
    await typeThenBlur(page, T.form.inputCountry, "DE", T.form.inputStreet);

    await expectFieldValidity(
      page,
      T.form.inputPostalCode,
      "1011",
      /*now invalid*/ true
    );

    // While invalid, save must be gated
    await expectSavesDisabled(page);
  });

  test("Entering valid input for all required fields enables save buttons", async ({
    page,
  }) => {
    await openNewForm(page);

    // Initially: empty form → invalid → save disabled
    await expectSavesDisabled(page);

    // fill all fields with valid values
    const name = `E2E Valid Test Address`;
    await fillAllRequiredValid(page, name);

    // expect all fields are valid (values taken from ..helpers/form.ts)
    await expectFieldValidity(page, T.form.inputName, name, false);
    await expectFieldValidity(
      page,
      T.form.inputStreet,
      "Beispielstr. 1",
      false
    );
    await expectFieldValidity(page, T.form.inputCountry, "DE", false);
    await expectFieldValidity(page, T.form.inputPostalCode, "10115", false);
    await expectFieldValidity(
      page,
      T.form.inputLocality,
      "Berlin Mitte",
      false
    );

    // All required fields are now valid → save buttons should be enabled
    await expectSavesEnabled(page);
  });
});
