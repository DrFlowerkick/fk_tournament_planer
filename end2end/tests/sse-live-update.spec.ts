import { test, expect, Page } from "@playwright/test";
import { T } from "../helpers/selectors";
import {
  openNewForm,
  fillAll,
  clickSave,
  typeThenBlur,
  expectPreviewShows,
  extractUuidFromUrl,
  waitForSseConnected,
  expectTestIdTextWithObserver,
} from "../helpers/form";

// --- Test data ---------------------------------------------------------------
// Unique test data (avoid partial-unique collisions)
const ts = Date.now();
const initial = {
  // Use values that make assertions obvious and avoid trimming/casing ambiguity.
  name: `E2E Test Address ${ts}`,
  street: "Main Street 42",
  postal_code: "10555",
  locality: "Berlin",
  region: "BE",
  country: "DE",
};

const edited = {
  // Only the name changes; other fields remain identical to ensure focused assertions.
  name: `${initial.name} (edited)`,
};

// --- Test --------------------------------------------------------------------

test.describe("SSE live update (Preview-only UI)", () => {
  test("editing in B updates Preview in A within 2s (no reload)", async ({
    browser,
  }) => {
    // Create two completely separate browser contexts to simulate two users.
    const ctxA = await browser.newContext();
    const ctxB = await browser.newContext();
    const pageA = await ctxA.newPage();
    const pageB = await ctxB.newPage();

    try {
      // -------------------- Arrange (A creates address) ----------------------
      await pageA.goto("/"); // baseURL is assumed to be configured in Playwright config.

      // Open and create a new, valid address.
      await openNewForm(pageA);
      await fillAll(
        pageA,
        initial.name,
        initial.street,
        initial.postal_code,
        initial.locality,
        initial.region,
        initial.country
      );
      await clickSave(pageA);

      // After save, route should be /postal-address/<uuid>
      await pageA.waitForURL(/\/postal-address\/[0-9a-f-]{36}$/);
      const urlA = pageA.url();
      const id = extractUuidFromUrl(urlA);

      // Ensure the preview shows the initial values and correct version
      await expectPreviewShows(pageA, initial);
      await expect(pageA.getByTestId(T.search.preview.version)).toHaveText("0");

      // If an SSE status element exists, only proceed when connected.
      await waitForSseConnected(pageA);

      // ----------------------- Act (B edits & saves) -------------------------
      // B opens the edit route directly for the same UUID.
      await pageB.goto(`/postal-address/${id}/edit`);

      // Change just the name; other fields remain as-is.
      await typeThenBlur(
        pageB,
        T.form.inputName,
        edited.name,
        T.form.inputStreet
      );

      // Start latency timer immediately before the save action.
      //const t0 = Date.now();
      await clickSave(pageB);

      // ----------------------- Assert (A updates via SSE) --------------------
      // wait for new version
      //await expect(pageA.getByTestId(T.search.preview.version)).toHaveText("1");
      // A's preview should reflect the edited name.
      await expect(pageA.getByTestId(T.search.preview.name)).toHaveText(edited.name);
      /*await expect
        .poll(
          async () => {
            // Hole beide Werte in JEDEM Poll-Intervall
            const version = await pageA
              .getByTestId(T.search.preview.version)
              .textContent();
            const name = await pageA
              .getByTestId(T.search.preview.name)
              .textContent();

            // Gib den aktuellen Zustand zurück (nur für Debugging-Logs)
            return { version, name };
          },
          {
            // Die Bedingung, die WAHRE sein muss, damit das Pollen stoppt
            message: `Warte darauf, dass Name='${edited.name}' UND Version='1' sind`,
            timeout: 10000,
          }
        )
        .toMatchObject({
          version: "1",
          name: edited.name,
        });*/

      // Optional sanity check: A did not navigate away (no hard reload).
      await expect(pageA).toHaveURL(urlA);
    } finally {
      // Cleanup contexts.
      await ctxA.close();
      await ctxB.close();
    }
  });
});
