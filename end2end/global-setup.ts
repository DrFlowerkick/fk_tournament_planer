// end2end/global-setup.ts

// setup for navigation of postal address list
import { chromium, Page } from "@playwright/test";
import {
  expectSavesDisabled,
  fillFields,
  clickSave,
  waitForPostalAddressListUrl,
} from "./helpers/postal_address";
import {
  openHomePage,
  selectSportPluginByName,
  goToNewTournament,
} from "./helpers/home";
import { selectors } from "./helpers/selectors";

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

  // 1. Select Sport
  await openHomePage(page);
  await selectSportPluginByName(page, PLUGINS.GENERIC);

  for (const t of SEED_TOURNAMENTS) {
    await goToNewTournament(page);

    // Fill form
    await FORM.inputs.name.fill(t.name);
    await FORM.inputs.entrants.fill(t.entrants);

    await FORM.actions.save.click();

    // Wait for completion (URL update) to be ready for the next one
    await page.waitForURL(/tournament_id=/, { timeout: 10000 });

    // Back to dashboard for next round
    await openHomePage(page);
    await selectSportPluginByName(page, PLUGINS.GENERIC);
  }
}

export default async () => {
  const browser = await chromium.launch();

  // Create a browser context with baseURL to allow relative navigation in helpers
  const context = await browser.newContext({
    baseURL: "http://localhost:3000",
  });
  const page = await context.newPage();

  // 1. Seed Postal Addresses
  const NEW_PA_URL = "/postal-address/new_pa";
  await page.goto(NEW_PA_URL);
  const names = ["Alpha", "Beta", "Gamma"];
  for (const name of names) {
    await expectSavesDisabled(page);
    await fillFields(page, {
      name: `E2E Nav ${name}`,
      street: "Teststr. 1",
      postal_code: "12345",
      locality: "Teststadt",
      region: "",
      country: "DE",
    });
    await clickSave(page);
    await waitForPostalAddressListUrl(page);
    await page.goto(NEW_PA_URL);
  }

  // 2. Seed Tournaments
  // We use try-catch so a failure here doesn't kill the whole test run,
  // e.g. if the page is not yet reachable.
  try {
    console.log("🌱 Seeding Tournaments...");
    await seedTournaments(page);
    console.log("✅ Seeding Complete");
  } catch (e) {
    console.error("❌ Seeding failed:", e);
    // Optional: throw e; if Seeding is strictly necessary
  }

  await page.close();
  await browser.close();
};
