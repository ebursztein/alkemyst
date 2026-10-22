import {
  createBrowserHistory,
  createRootRoute,
  createRoute,
  createRouter,
  Navigate,
} from "@tanstack/react-router";
import { Shell } from "./shell.js";
import { DecksPage } from "../pages/decks.js";
import { DeckDetailPage } from "../pages/deck-detail.js";
import { LabIndex, LabPage } from "../pages/lab.js";
import { GamesPage } from "../pages/games.js";
import { GameDetailPage } from "../pages/game-detail.js";
import { PracticePage } from "../pages/practice.js";
import { PracticeTablePage } from "../pages/practice-table.js";
import { SettingsPage } from "../pages/settings.js";
import { Failure } from "../ui/states.js";
const root = createRootRoute({
  component: Shell,
  notFoundComponent: () => (
    <Failure error={{ message: "This page is not in your local workspace." }} />
  ),
});
const index = createRoute({
  getParentRoute: () => root,
  path: "/",
  component: () => <Navigate to="/decks" />,
});
const decks = createRoute({ getParentRoute: () => root, path: "/decks", component: DecksPage });
const detail = createRoute({
  getParentRoute: () => root,
  path: "/decks/$versionId",
  component: DeckDetailPage,
});
const lab = createRoute({ getParentRoute: () => root, path: "/lab", component: LabIndex });
const experiment = createRoute({
  getParentRoute: () => root,
  path: "/lab/$versionId",
  component: LabPage,
});
const games = createRoute({ getParentRoute: () => root, path: "/games", component: GamesPage });
const game = createRoute({
  getParentRoute: () => root,
  path: "/games/$gameId",
  component: GameDetailPage,
});
const practice = createRoute({
  getParentRoute: () => root,
  path: "/practice",
  component: PracticePage,
});
const table = createRoute({
  getParentRoute: () => root,
  path: "/practice/$sessionId",
  component: PracticeTablePage,
});
const settings = createRoute({
  getParentRoute: () => root,
  path: "/settings",
  component: SettingsPage,
});
const tree = root.addChildren([
  index,
  decks,
  detail,
  lab,
  experiment,
  games,
  game,
  practice,
  table,
  settings,
]);
export function createDesktopRouter(history = createBrowserHistory()) {
  return createRouter({
    routeTree: tree,
    history,
    defaultPreload: "intent",
    scrollRestoration: true,
  });
}
declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof createDesktopRouter>;
  }
}
