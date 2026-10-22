import { watchSettingsNavigation } from "./native-navigation.js";
import { useEffect, useRef } from "react";
import { Link, Outlet, useNavigate, useRouterState } from "@tanstack/react-router";
import { useCatalogStatus } from "./catalog-sync.js";
const navigation = [
  { path: "/decks", label: "Decks", icon: "▤" },
  { path: "/games", label: "Games", icon: "◷" },
  { path: "/lab", label: "Lab", icon: "⚗" },
  { path: "/practice", label: "Practice", icon: "◇" },
  { path: "/settings", label: "Settings", icon: "⚙" },
] as const;
export function Shell() {
  const navigate = useNavigate();
  useEffect(
    () =>
      watchSettingsNavigation(() => {
        void navigate({ to: "/settings" });
      }),
    [navigate],
  );
  const path = useRouterState({ select: (state) => state.location.pathname });
  const previous = useRef(path);
  useEffect(() => {
    if (previous.current !== path) {
      document.getElementById("main-content")?.focus({ preventScroll: true });
      previous.current = path;
    }
  }, [path]);
  const catalog = useCatalogStatus();
  return (
    <div className="app-layout">
      <a className="skip-link" href="#main-content">
        Skip to content
      </a>
      <aside className="sidebar">
        <Link to="/decks" className="brand" aria-label="Alkemyst decks">
          <svg viewBox="0 0 32 36" aria-hidden="true">
            <path
              d="M11 3h10M13 3v10L4 28q-2 5 4 5h16q6 0 4-5l-9-15V3M9 23h14"
              fill="none"
              stroke="currentColor"
              strokeWidth="2.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
            <circle cx="15" cy="26" r="2" fill="currentColor" />
          </svg>
          <span>
            alkemyst<span className="brand-dot">.</span>
          </span>
        </Link>
        <p className="sidebar-label">YOUR WORKSPACE</p>
        <nav aria-label="Main navigation">
          {navigation.map((n) => (
            <Link
              key={n.path}
              to={n.path}
              activeProps={{ className: "active", "aria-current": "page" }}
            >
              <span aria-hidden="true">{n.icon}</span>
              {n.label}
            </Link>
          ))}
        </nav>
        <div className="sidebar-footer">
          <span className={`status-dot ${catalog.isError ? "disconnected" : ""}`} />
          <div>
            <strong>
              {catalog.isError
                ? "Service unavailable"
                : catalog.isPending
                  ? "Connecting…"
                  : "Local library"}
            </strong>
            <small>
              {catalog.data
                ? `${catalog.data.cards.toLocaleString()} card identities`
                : "On this device"}
            </small>
          </div>
        </div>
        <p className="sidebar-note">
          No cloud sync.
          <br />
          Your brews stay yours.
        </p>
      </aside>
      <main id="main-content" className="main-content" tabIndex={-1}>
        {catalog.data?.cards === 0 && path !== "/settings" && (
          <p className="notice" role="status">
            Download card metadata to enable local search and deck analysis.{" "}
            <Link to="/settings">Set up card library</Link>
          </p>
        )}
        <Outlet />
      </main>
    </div>
  );
}
