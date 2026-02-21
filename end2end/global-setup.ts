// end2end/global-setup.ts

// setup for navigation of postal address list
import { chromium, Page, expect } from "@playwright/test";
import {
  selectors,
  openHomePage,
  selectSportPluginByName,
  goToNewTournament,
  fillFields,
  closeForm,
  waitForPostalAddressListUrl,
  fillAndBlur,
  waitForAppHydration,
  clickNewPostalAddress,
} from "./helpers";

const PLUGINS = {
  GENERIC: "Generic Sport",
};

// Constants for seed data, reusable in tests
// (Ideally moved to a shared constants file, but hardcoded here is fine)
const SEED_TOURNAMENTS = [
  { name: "Seed Draft Tournament", entrants: "16" },
  { name: "Seed Big Tournament", entrants: "64" },
  { name: "Seed Pro Tournament", entrants: "32" },
];

async function seedTournaments(page: Page) {
  const FORM = selectors(page).home.dashboard.editTournament;

  // 1. Select Sport (ensure we are in the context)
  await openHomePage(page);
  await selectSportPluginByName(page, PLUGINS.GENERIC);

  for (const t of SEED_TOURNAMENTS) {
    await goToNewTournament(page);

    // Fill form
    console.log(`🌱 Seeding Tournament: ${t.name}`);
    await expect(FORM.inputs.name).toHaveAttribute("aria-invalid", "true");
    await expect(FORM.inputs.entrants).toHaveAttribute("aria-invalid", "true");

    // Use the helper here
    await fillAndBlur(FORM.inputs.name, t.name);
    await fillAndBlur(FORM.inputs.entrants, t.entrants);

    await expect(FORM.inputs.name).toHaveValue(t.name);
    await expect(FORM.inputs.entrants).toHaveValue(t.entrants);
    await expect(FORM.inputs.name).toHaveAttribute("aria-invalid", "false");
    await expect(FORM.inputs.entrants).toHaveAttribute("aria-invalid", "false");

    // Save
    await expect(FORM.actions.save).toBeEnabled();
    await FORM.actions.save.click();

    // Wait for completion (URL update) to be ready for the next one
    await page.waitForURL(/tournament_id=/, { timeout: 10000 });
  }
}

async function seedPostalAddresses(page: Page) {
  const NEW_PA_URL = "/postal-address";
  await page.goto(NEW_PA_URL);
  await waitForAppHydration(page);

  const names = ["Alpha", "Beta", "Gamma"];

  for (const name of names) {
    console.log(`🌱 Seeding Postal Address: ${name}`);
    // Click "New" button to open the form
    await clickNewPostalAddress(page);

    // Fill form fields (using helper for consistency)
    await fillFields(page, {
      name: `E2E Nav ${name}`,
      street: "Teststr. 1",
      postal_code: "12345",
      locality: "Teststadt",
      region: "",
      country: "DE",
    });
    await closeForm(page);
    await waitForPostalAddressListUrl(page, true);
  }
}

export default async () => {
  const browser = await chromium.launch();

  // Create a browser context with baseURL to allow relative navigation in helpers
  const context = await browser.newContext({
    baseURL: "http://localhost:3000",
  });
  const page = await context.newPage();

  try {
    console.log("🌱 Seeding Postal Addresses...");
    await seedPostalAddresses(page);
    console.log("✅ Postal Addresses Seeded");

    console.log("🌱 Seeding Tournaments...");
    await seedTournaments(page);
    console.log("✅ Tournaments Seeded");
  } catch (e) {
    console.error("❌ Seeding failed:", e);
    // Throwing error to stop the test run, as seeding is mandatory
    throw e;
  } finally {
    await page.close();
    await browser.close();
  }
};
