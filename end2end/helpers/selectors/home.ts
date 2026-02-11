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
    tournamentsList: {
      root: "tournaments-list-root",
      filters: {
        statusSelect: "select-filter-tournament-state",
        includeAdhoc: "filter-include-adhoc-toggle",
        limitSelect: "filter-limit-select",
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
        configureSingleStage: "link-configure-single-stage",
        configurePoolStage: "link-configure-pool-stage",
        configureSwissSystem: "link-configure-swiss-system",
      },
      actions: {
        save: "btn-tournament-save",
        cancel: "btn-tournament-cancel",
      },
    },
    editStage: {
      root: "stage-editor-root",
      title: "stage-editor-title",
      form: "stage-editor-form",
      inputs: {
        numGroups: "input-stage-num-groups",
      },
      groupLink: (index: number) => `link-configure-group-${index}`,
    },
    editGroup: {
      root: "group-editor-root",
      title: "group-editor-title",
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
      tournamentsList: {
        ...getListSelectors(page),
        root: page.getByTestId(ids.dashboard.tournamentsList.root),
        filters: {
          status: page.getByTestId(
            ids.dashboard.tournamentsList.filters.statusSelect,
          ),
          adhocToggle: page.getByTestId(
            ids.dashboard.tournamentsList.filters.includeAdhoc,
          ),
          limit: page.getByTestId(
            ids.dashboard.tournamentsList.filters.limitSelect,
          ),
        },
        actions: {
          show: page.getByTestId("action-btn-show"),
          register: page.getByTestId("action-btn-register"),
          results: page.getByTestId("action-btn-results"),
        },
        emptyState: page.getByTestId(ids.dashboard.tournamentsList.emptyState),
      },
      editTournament: {
        root: page.getByTestId(ids.dashboard.editTournament.root),
        title: page.getByTestId(ids.dashboard.editTournament.title),
        form: page.getByTestId(ids.dashboard.editTournament.form),
        inputs: {
          name: page.getByTestId(ids.dashboard.editTournament.inputs.name),
          entrants: page.getByTestId(
            ids.dashboard.editTournament.inputs.entrants,
          ),
          mode: page.getByTestId(ids.dashboard.editTournament.inputs.mode),
          num_rounds_swiss: page.getByTestId(
            ids.dashboard.editTournament.inputs.num_rounds_swiss,
          ),
        },
        links: {
          configureSingleStage: page.getByTestId(
            ids.dashboard.editTournament.links.configureSingleStage,
          ),
          configurePoolStage: page.getByTestId(
            ids.dashboard.editTournament.links.configurePoolStage,
          ),
          configureSwissSystem: page.getByTestId(
            ids.dashboard.editTournament.links.configureSwissSystem,
          ),
        },
        actions: {
          save: page.getByTestId(ids.dashboard.editTournament.actions.save),
          cancel: page.getByTestId(ids.dashboard.editTournament.actions.cancel),
        },
      },
      editStage: {
        root: page.getByTestId(ids.dashboard.editStage.root),
        title: page.getByTestId(ids.dashboard.editStage.title),
        form: page.getByTestId(ids.dashboard.editStage.form),
        inputs: {
          numGroups: page.getByTestId(ids.dashboard.editStage.inputs.numGroups),
        },
        groupLink: (index: number) =>
          page.getByTestId(ids.dashboard.editStage.groupLink(index)),
      },
      editGroup: {
        root: page.getByTestId(ids.dashboard.editGroup.root),
        title: page.getByTestId(ids.dashboard.editGroup.title),
      },
    },
  };
}
