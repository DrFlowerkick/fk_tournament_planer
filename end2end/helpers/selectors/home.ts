import type { Page } from "@playwright/test";

export const HOME_IDS = {
  hero: {
    root: "home-hero",
    title: "home-hero-title",
    description: "home-hero-desc",
  },
  sportSelection: {
    grid: "sport-selection-grid",
    // Helper to generate ID based on Name (stripping whitespace)
    btnSelectPluginByName: (name: string) =>
      `btn-select-sport-${name.replace(/\s/g, "")}`,
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

      // New selector method based on the clean name logic
      pluginButtonByName: (name: string) =>
        page.getByTestId(ids.sportSelection.btnSelectPluginByName(name)),
    },
  };
}
