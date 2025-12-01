// end2end/global-setup.ts

// setup for navigation of postal address list
import { chromium } from "@playwright/test";
import {
  expectSavesDisabled,
  fillFields,
  clickSave,
  waitForPostalAddressListUrl,
} from "./helpers/postal_address";
export default async () => {
  const NEW_URL = "http://localhost:3000/postal-address/new_pa";
  const browser = await chromium.launch();
  const page = await browser.newPage();
  await page.goto(NEW_URL);
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
    await page.goto(NEW_URL);
  }
  await page.close();
  await browser.close();
};
