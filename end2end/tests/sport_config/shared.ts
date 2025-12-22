import { test, expect, Page } from "@playwright/test";
import { selectors } from "../../helpers/selectors";
import {
  openSportSelectionAndConfigList,
  clickNewToOpenEditForm,
  clickEditToOpenEditForm,
} from "../../helpers/sport_config";
import {
  searchAndOpenByNameOnCurrentPage,
  extractQueryParamFromUrl,
} from "../../helpers/utils";

export interface SportConfigTestAdapter {
  sportName: string;
  generateData: () => any;
  fillSpecificFields: (page: Page, data: any) => Promise<void>;
  assertSpecificFields: (page: Page, data: any) => Promise<void>;
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
        await openSportSelectionAndConfigList(page);

        // Select the sport plugin
        await SC.pluginSelector.input.click();
        await SC.pluginSelector.input.fill(adapter.sportName);

        // Wait for suggestions and click the first one
        await expect(SC.pluginSelector.items.first()).toBeVisible();
        await SC.pluginSelector.items.first().click();

        // Wait for the "New" button to be enabled/visible if it depends on selection
        await expect(SC.search.btnNew).toBeVisible();
        await SC.search.btnNew.click();
        await expect(SC.form.root).toBeVisible();
      });

      await test.step("Fill and Save New Config", async () => {
        await SC.form.inputName.fill(initialName);
        await adapter.fillSpecificFields(page, initialData);

        await SC.form.btnSave.click();

        // Wait for save to complete
        await expect(SC.search.dropdown.input).toHaveValue(initialName);
        // Expect edit inputs are not visible anymore
        await expect(SC.form.root).toBeHidden();
      });

      await test.step("Verify Created Config in List", async () => {
        await searchAndOpenByNameOnCurrentPage(
          SC.search.dropdown,
          initialName,
          {
            clearFirst: true,
            expectUnique: true,
          }
        );

        // Verify preview
        await expect(SC.search.preview).toBeVisible();
        await expect(SC.search.preview).toContainText(initialName);
        await adapter.assertSpecificFields(page, initialData);
      });

      await test.step("Edit Config", async () => {
        await clickEditToOpenEditForm(page);

        // Update fields
        await SC.form.inputName.fill(updatedName);
        await adapter.fillSpecificFields(page, updatedData);

        await SC.form.btnSave.click();
        await expect(SC.search.dropdown.input).toBeVisible();
      });

      await test.step("Verify Updated Config", async () => {
        await searchAndOpenByNameOnCurrentPage(
          SC.search.dropdown,
          updatedName,
          {
            clearFirst: true,
            expectUnique: true,
          }
        );

        await expect(SC.search.preview).toBeVisible();
        await expect(SC.search.preview).toContainText(updatedName);
        await adapter.assertSpecificFields(page, updatedData);
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
          await openSportSelectionAndConfigList(pageA);

          // Select the sport plugin
          await SC_A.pluginSelector.input.click();
          await SC_A.pluginSelector.input.fill(adapter.sportName);

          // Wait for suggestions and click the first one
          await expect(SC_A.pluginSelector.items.first()).toBeVisible();
          await SC_A.pluginSelector.items.first().click();

          await expect(SC_A.search.btnNew).toBeVisible();
          await SC_A.search.btnNew.click();

          await SC_A.form.inputName.fill(initialName);
          await adapter.fillSpecificFields(pageA, initialData);

          await SC_A.form.btnSave.click();
          // Wait for save to complete
          await expect(SC_A.search.dropdown.input).toHaveValue(initialName);
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
            `/sport/edit_sc?sport_id=${sportId}&sport_config_id=${configId}`
          );
          // Wait for form to be ready
          await expect(SC_B.form.root).toBeVisible();

          // Update
          await SC_B.form.inputName.fill(updatedName);
          await adapter.fillSpecificFields(pageB, updatedData);

          await SC_B.form.btnSave.click();
          // Wait for save to complete
          await expect(SC_B.search.dropdown.input).toBeVisible();
        });

        // --- User A sees updates live ---
        await test.step("User A sees updates live", async () => {
          await expect(SC_A.search.preview).toContainText(updatedName);
          await adapter.assertSpecificFields(pageA, updatedData);
        });
      } finally {
        await ctxA.close();
        await ctxB.close();
      }
    });
  });
}
