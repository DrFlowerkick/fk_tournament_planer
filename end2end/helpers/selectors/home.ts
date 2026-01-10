import type { Page } from "@playwright/test";

export const HOME_IDS = {
  hero: {
    root: "home-hero",
    title: "home-hero-title",
    description: "home-hero-desc",
  },
  sportSelection: {
    grid: "sport-selection-grid",
    // Helper function to generate ID based on plugin ID
    btnSelectPlugin: (pluginId: string) => `btn-select-sport-${pluginId}`,
  },
} as const;

export function getHomeSelectors(page: Page) {
  const ids = HOME_IDS;
  return {
    hero: {
      root: page.getByTestId(ids.hero.root),
      title: page.getByTestId(ids.hero.title),
      description: page.getByTestId(ids.hero.description),
    },
    sportSelection: {
      grid: page.getByTestId(ids.sportSelection.grid),
      // Select specific ID if known
      pluginButton: (pluginId: string) =>
        page.getByTestId(ids.sportSelection.btnSelectPlugin(pluginId)),
      // Select by visible name (more robust for uncertain UUIDs)
      pluginButtonByName: (name: string) =>
        page
          .getByTestId(ids.sportSelection.grid)
          .getByRole("button", { name: name }),
    },
  };
}
