---
title: Alkemyst documentation
description: Your local Arena library, experiments, and agent connections.
---

Alkemyst is an open-source, local-first deck laboratory for Magic: The Gathering
Arena. It combines captured games, immutable deck versions, local card search,
seeded Rust simulations, and a manual practice table.

The app is in development. **Preview installers are not published yet.**
Check [download status](/download/) and the notes for any future release before installing.

## Start with your own library

1. Read [setup](/docs/setup/) for supported targets, card metadata and local data.
2. Learn what [Arena tracking](/docs/guides/tracking/) can and cannot capture.
3. Review the [analysis assumptions](/docs/guides/analysis/) before interpreting a result.
4. Connect your own agent using [MCP](/docs/guides/mcp/), or [contribute](/docs/guides/development/).

## What stays local

Decks, captured history, notes, practice sessions, and saved simulation runs use the
app's local SQLite database. Alkemyst has no accounts or cloud deck synchronization.

Public card metadata must be downloaded before it is available offline. Card image
URLs are cached as metadata; image files still load from Scryfall when displayed.
An external agent or a future optional AI provider has its own data-handling terms.
Connecting an agent does not make that agent local.

## Current boundaries

The first target is 60-card Arena constructed, including Bo1 and Bo3 captures when
the logs provide enough information. The practice table is manual, and the
simulation engine supports a bounded set of card interactions. Neither is a full
Magic rules engine. In-app AI chat, backup/restore UI and installer distribution
remain tracked work.
