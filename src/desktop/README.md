# Desktop interface

React, TanStack Router/Query and the OpenAPI-generated SDK provide Decks, Games,
Lab, Practice and Settings. Application reads and writes use the local HTTP API;
Tauri's `connection_bootstrap` command supplies only a loopback address and scoped
credential. Credentials stay in memory and are revealed in Settings only on request.
Native menus, the tray and window lifecycle are implemented; installer readiness
is tracked separately.

Deck workflows include Arena import/export, duplication, append-only version
editing, local card search, notes, version browsing and quantity/curve comparison.
The Lab exposes supported seeded native simulations, frozen run assumptions,
progress, cancellation, replay metadata and confidence intervals with actual sample
counts. Model exclusions are visible; unsupported decks cannot be simulated.
Games show observed results, partial-capture flags, sampled turns and event timelines.
Practice is a saved manual sandbox with ordered London bottoming and visible-card
controls. It does not automate payments, abilities, combat or state-based actions.

The interface follows the system light/dark theme, respects reduced motion and
provides semantic controls, focus navigation and labeled chart tables. Untitled UI's
MIT button attribution is preserved; see `public/licenses/untitled-ui.txt` and `src/ui/README.md`.

## Verification

`pnpm verify`, `pnpm security` and `pnpm test:e2e` run from the repository root.
Vitest workflows use an isolated native backend with public card metadata; the
history fixture is sanitized. Browser tests replace only the native connection
bootstrap, use the actual API/SQL worker, and run keyboard, narrow-window, theme
and WCAG Axe checks. They never open the user's database or Arena logs.

`pnpm dev` starts the browser development surface at port 1420. Without a native
bootstrap it displays connection guidance; it never falls back to public or
unauthenticated data. `pnpm --filter @alkemyst/desktop build` produces the frontend
assets for the native shell. Browser fixtures are test-only infrastructure, not a
production connection mechanism.

## Native shell (in development)

`pnpm --filter @alkemyst/desktop tauri dev` runs the Tauri window with Vite.
The shell stores its SQLite database in the operating system's application-data
folder, discovers Arena logs, and starts an authenticated loopback service. Closing
the window hides it; the menu-bar/system-tray icon can reopen it or pause capture.
Explicit Quit stops capture, simulation jobs, MCP, HTTP and the database worker.
Launch at login is disabled until explicitly enabled in Settings.

After building the frontend, `pnpm --filter @alkemyst/desktop native:smoke` exercises
a real native window and tray against an isolated temporary database: close the
window, ingest a sanitized game, initialize MCP, then Quit and verify HTTP stopped.
The smoke feature is test infrastructure and is excluded from release builds.

Coverage includes the native build script and real-window smoke test. Native
packaging and Windows GUI verification remain in progress. The new dependency lockfile currently fails OSV on Linux-only Tauri
transitives; remediation is tracked in Sprinty S05-008. No preview installer is
published, and stable signing/notarization remains a release prerequisite.
