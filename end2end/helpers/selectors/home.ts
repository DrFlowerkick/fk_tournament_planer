import type { Page } from "@playwright/test";
import { getListSelectors } from "./common";

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
  tournamentsList: {
    root: "tournaments-list-root",
    filters: {
      statusSelect: "filter-tournament-state-select",
      includeAdhoc: "filter-include-adhoc-toggle",
    },
    emptyState: "tournaments-list-empty",
  },
  editTournament: {
    root: "tournament-editor-root",
    title: "tournament-editor-title",
    form: "tournament-editor-form",
    inputs: {
      name: "input-tournament-name",
      entrants: "input-tournament-entrants",
      mode: "select-tournament-mode",
      num_rounds_swiss: "input-tournament-swiss-num_rounds",
    },
    links: {
      configureSingleStage: "action-btn-configure-stage-single-stage",
      configurePoolStage: "action-btn-configure-stage-pool-stage",
      configureFinalStage: "action-btn-configure-stage-final-stage",
      configureSwissSystem: "action-btn-configure-stage-swiss-system",
    },
    actions: {
      close: "action-btn-close-edit-base",
    },
  },
  editStage: {
    root: "stage-editor-root",
    title: "stage-editor-title",
    form: "stage-editor-form",
    inputs: {
      numGroups: "input-stage-num-groups",
    },
    groupActionBtn: (index: number) => `action-btn-configure-group-${index}`,
  },
  editGroup: {
    root: "group-editor-root",
    title: "group-editor-title",
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
    tournamentsList: {
      ...getListSelectors(page),
      root: page.getByTestId(ids.tournamentsList.root),
      filters: {
        status: page.getByTestId(
          ids.tournamentsList.filters.statusSelect,
        ),
        adhocToggle: page.getByTestId(
          ids.tournamentsList.filters.includeAdhoc,
        ),
      },
      emptyState: page.getByTestId(ids.tournamentsList.emptyState),
    },
    editTournament: {
      root: page.getByTestId(ids.editTournament.root),
      title: page.getByTestId(ids.editTournament.title),
      form: page.getByTestId(ids.editTournament.form),
      inputs: {
        name: page.getByTestId(ids.editTournament.inputs.name),
        entrants: page.getByTestId(
          ids.editTournament.inputs.entrants,
        ),
        mode: page.getByTestId(ids.editTournament.inputs.mode),
        num_rounds_swiss: page.getByTestId(
          ids.editTournament.inputs.num_rounds_swiss,
        ),
      },
      links: {
        configureSingleStage: page.getByTestId(
          ids.editTournament.links.configureSingleStage,
        ),
        configurePoolStage: page.getByTestId(
          ids.editTournament.links.configurePoolStage,
        ),
        configureFinalStage: page.getByTestId(
          ids.editTournament.links.configureFinalStage,
        ),
        configureSwissSystem: page.getByTestId(
          ids.editTournament.links.configureSwissSystem,
        ),
      },
      actions: {
        close: page.getByTestId(ids.editTournament.actions.close),
      },
    },
    editStage: {
      root: page.getByTestId(ids.editStage.root),
      title: page.getByTestId(ids.editStage.title),
      form: page.getByTestId(ids.editStage.form),
      inputs: {
        numGroups: page.getByTestId(ids.editStage.inputs.numGroups),
      },
      groupActionBtn: (index: number) =>
        page.getByTestId(ids.editStage.groupActionBtn(index)),
    },
    editGroup: {
      root: page.getByTestId(ids.editGroup.root),
      title: page.getByTestId(ids.editGroup.title),
    },
  };
}
