import { test, expect } from "@playwright/test";

test("Smoke: Select Plugin -> Search Config -> New -> Cancel", async ({ page }) => {
  // 1. Open Sport Config List
  //    If no sport_id is in query, it should show the Plugin Selection
  await page.goto("/sport-config");
  await expect(page.getByText("Select a Sport")).toBeVisible();

  // 2. Select a Sport Plugin (e.g., "Generic Sport")
  //    This should update the URL to include ?sport_id=...
  await page.getByRole('button', { name: 'Generic Sport' }).click();
  const url = new URL(page.url());
  expect(url.searchParams.has('sport_id')).toBeTruthy();

  // 3. Verify Search/List View is now active for configs
  await expect(page.getByPlaceholder("Search Configuration")).toBeVisible();

  // 4. Navigate to New Form
  await page.getByRole('link', { name: 'New' }).click();

  // 5. Verify Sport-Specific Form is rendered
  //    (e.g., check for a field specific to Generic Sport)
  //    For now, we just check if the form is visible.
  await expect(page.getByRole('button', { name: 'Save' })).toBeVisible();

  // 6. Cancel back to search context
  await page.getByRole('button', { name: 'Cancel' }).click();
  
  // 7. Verify we are back at the list and sport_id is preserved
  await expect(page.getByPlaceholder("Search Configuration")).toBeVisible();
  expect(page.url()).toContain("sport_id=");
});
