---
title: Setup
description: Target platforms, first-run metadata, local data and preview readiness.
---

## Preview status

There are no published preview installers yet. The first targets are **macOS 13+
on Apple Silicon** and **Windows 11 x64**. Follow [download status](/download/)
and the exact release notes when a build is published. Stable distribution requires
trusted signing and macOS notarization.

Developers can [build a reviewed revision](/docs/guides/development/).

## First run

The native app starts an authenticated service on loopback and discovers the
platform's Arena logs. Start Arena if Settings says **Waiting for Arena logs**.
The app can import retained history and follow new log writes. It cannot recover
history that Arena has already discarded.

Use **Settings → Update card metadata** to populate the card library from
Scryfall. The download needs internet; an already imported snapshot remains usable
offline. A failed refresh retains the previous active snapshot. Local search covers
card names, rules text and types; the API also supports color, format, mana-value,
legendary and Arena-availability filters.

Import a deck using Arena text in **Decks**, or let capture record a submitted
Arena deck when that information is present. Local edits create a new version.
Export Arena text to transfer an edited list back into the game.

## Closing and quitting

Closing the window hides it and leaves capture and MCP available through the tray
or macOS menu bar. **Show Alkemyst** opens it again. **Pause Arena tracking** stops
capture without stopping the library or MCP service. **Quit Alkemyst** stops the
local services and exits.

Launch at login is opt-in in Settings. It opens in the background when enabled.
Settings reports live capture status and distinguishes paused, waiting, following,
and unavailable states.

## Local data

The database lives in the operating system's application-data directory for
`cards.alkemyst.desktop`, not in the Arena installation. Arena log files are read
without being rewritten. Treat your database as private: it may contain player or
match identifiers even when the interface displays only summaries.

The backup/restore interface is still being built. Do not replace or copy a live
SQLite database as a substitute for a validated backup. Development tests use
isolated databases and sanitized fixtures, never your live data.

## Offline use

Existing decks, history, notes, practice sessions and saved simulations are local.
New card metadata and uncached images require connectivity. Native simulation
runs do not call a cloud model. External agents may need their own connection.
