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
  // NEU: Dashboard Area
  dashboard: {
    root: "sport-dashboard",
    title: "sport-dashboard-title",
    description: "sport-dashboard-desc",
    nav: {
      tournaments: "link-nav-tournaments",
      planNew: "link-nav-plan-new",
      adhoc: "link-nav-adhoc",
      config: "link-nav-config",
      about: "link-nav-about",
    },
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
      pluginButtonByName: (name: string) =>
        page.getByTestId(ids.sportSelection.btnSelectPluginByName(name)),
    },
    dashboard: {
      root: page.getByTestId(ids.dashboard.root),
      title: page.getByTestId(ids.dashboard.title),
      description: page.getByTestId(ids.dashboard.description),
      nav: {
        tournaments: page.getByTestId(ids.dashboard.nav.tournaments),
        planNew: page.getByTestId(ids.dashboard.nav.planNew),
        adhoc: page.getByTestId(ids.dashboard.nav.adhoc),
        config: page.getByTestId(ids.dashboard.nav.config),
        about: page.getByTestId(ids.dashboard.nav.about),
      },
    },
  };
}
