import { test, expect, Page, Locator } from "@playwright/test";
import {
  openSportConfigurationList,
  clickEditSportConfig,
  extractQueryParamFromUrl,
  waitForAppHydration,
  waitForNavigationRowSelectionByName,
  searchAndOpenByNameOnCurrentPage,
  selectors,
} from "../../helpers";
import exp from "constants";

export interface SportConfigTestAdapter {
  sportName: string;
  generateData: () => any;
  fillSpecificFields: (page: Page, data: any) => Promise<void>;
  assertSpecificFields: (row: Locator, data: any) => Promise<void>;
}

export function runSportConfigSharedTests(adapter: SportConfigTestAdapter) {
  test.describe(`Sport Config Shared Tests: ${adapter.sportName}`, () => {
    test("Create and Edit Flow", async ({ page }) => {
      const ts = Date.now();
      const initialName = `E2E ${adapter.sportName} ${ts} Flow`;
      const updatedName = `${initialName} Updated`;
      const initialData = adapter.generateData();
      const updatedData = adapter.generateData();

      const SC = selectors(page).sportConfig;

      await test.step("Navigate to New Sport Config", async () => {
        await openSportConfigurationList(page, adapter.sportName);

        // Wait for the "New" button to be enabled/visible if it depends on selection
        await expect(SC.list.btnNew).toBeVisible();
        await SC.list.btnNew.click();
        await expect(SC.form.root).toBeVisible();
      });

      await test.step("Fill and Save New Config", async () => {
        await SC.form.inputName.fill(initialName);
        await adapter.fillSpecificFields(page, initialData);

        await SC.form.btnSave.click();

        // Wait for save to complete
        await expect(SC.list.previewByName(initialName)).toBeVisible();
        // Expect edit inputs are not visible anymore
        await expect(SC.form.root).toBeHidden();

        await searchAndOpenByNameOnCurrentPage(page, initialName, "sport_config_id");

        // Verify preview
        await expect(SC.list.detailedPreview).toBeVisible();
        await adapter.assertSpecificFields(
          SC.list.detailedPreview,
          initialData,
        );
      });

      await test.step("Edit Config", async () => {
        await expect(SC.list.btnEdit).toBeVisible();
        await clickEditSportConfig(page);

        // Update fields
        await SC.form.inputName.fill(updatedName);
        await adapter.fillSpecificFields(page, updatedData);

        await SC.form.btnSave.click();
        await expect(SC.list.previewByName(updatedName)).toBeVisible();

        // Verify preview
        await expect(SC.list.detailedPreview).toBeVisible();
        await adapter.assertSpecificFields(
          SC.list.detailedPreview,
          updatedData,
        );
      });
    });

    test("Live Update (Preview-only UI)", async ({ browser }) => {
      const ts = Date.now();
      const initialName = `E2E ${adapter.sportName} ${ts} Live`;
      const updatedName = `${initialName} Updated`;
      const initialData = adapter.generateData();
      const updatedData = adapter.generateData();

      const ctxA = await browser.newContext();
      const ctxB = await browser.newContext();
      const pageA = await ctxA.newPage();
      const pageB = await ctxB.newPage();

      const SC_A = selectors(pageA).sportConfig;
      const SC_B = selectors(pageB).sportConfig;

      try {
        // --- User A creates the config ---
        await test.step("User A creates config", async () => {
          await openSportConfigurationList(pageA, adapter.sportName);

          await expect(SC_A.list.btnNew).toBeVisible();
          await SC_A.list.btnNew.click();

          await SC_A.form.inputName.fill(initialName);
          await adapter.fillSpecificFields(pageA, initialData);

          await SC_A.form.btnSave.click();
          // Wait for save to complete
          await expect(SC_A.list.previewByName(initialName)).toBeVisible();
          // Expect edit inputs are not visible anymore
          await expect(SC_A.form.root).toBeHidden();
        });

        // Get IDs
        const urlA = pageA.url();
        const sportId = extractQueryParamFromUrl(urlA, "sport_id");
        const configId = extractQueryParamFromUrl(urlA, "sport_config_id");

        // --- User B edits the config ---
        await test.step("User B edits config", async () => {
          await pageB.goto(
            `/sport-configurations/edit?sport_id=${sportId}&sport_config_id=${configId}`,
          );

          // Ensure explicit navigation waits for WASM hydration before interacting
          await waitForAppHydration(pageB);

          // Wait for form to be ready
          await expect(SC_B.form.root).toBeVisible();

          // Update
          await SC_B.form.inputName.fill(updatedName);
          await adapter.fillSpecificFields(pageB, updatedData);

          await SC_B.form.btnSave.click();
          // Wait for save to complete
          await expect(SC_B.list.previewByName(updatedName)).toBeVisible();
        });

        // --- User A sees updates live ---
        await test.step("User A sees updates live", async () => {
          await expect(SC_A.list.entryName(configId!)).toHaveText(updatedName);
          await adapter.assertSpecificFields(
            SC_A.list.detailedPreview,
            updatedData,
          );
        });
      } finally {
        await ctxA.close();
        await ctxB.close();
      }
    });
  });
}
