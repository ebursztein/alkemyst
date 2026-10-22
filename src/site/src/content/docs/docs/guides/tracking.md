---
title: Arena tracking
description: What is captured, what may be missing, and how game history stays faithful.
---

## Capture what the logs actually contain

Alkemyst follows Arena's current log and imports retained previous-session data.
It handles partial writes, restarts, duplicate imports and rotation. Available deck
submissions, game events, results, mulligans, play/draw and land-drop observations
can appear in the local history.

Missing fields stay missing. A partial capture is not a complete game, and an
unknown result never becomes a loss. A lack of observed land drops is not proof
that the player missed every land drop.

Arena controls the contents and retention of its logs. Detailed event availability
may depend on Arena's logging settings and client behavior. Enable detailed logs
in Arena if needed, then verify the captured timeline rather than assuming that
all previous games can be reconstructed.

## Matches, games and versions

A match can contain multiple games. Bo1/Bo3 labels are only assigned when the
capture supplies an explicit supported queue classification. Captured deck
versions remain attached to their history. A local edit or agent edit creates a
new version rather than changing the list associated with an old game.

## Read the Games view

Game summaries distinguish known results from unknown ones. Observed performance
uses sample counts and a decisive-game denominator where appropriate. Turn
summaries and timelines show capture gaps; unassigned events are not silently
invented into a turn.

Use these observations to form a question, then test that question under the
[simulation model](/docs/guides/analysis/). A simulated opening does not establish
an empirical match win rate.

## Troubleshooting

- **Waiting for Arena logs:** launch Arena and check that its logging is enabled.
- **Paused:** resume from Settings or the tray.
- **Needs attention:** check that the log directory and local data are accessible,
  then pause and resume to retry. Share only sanitized diagnostics.
- **Unknown card IDs:** refresh metadata. Some Arena identities require aliases
  from Arena's local card database; metadata availability and ID resolution differ.

For a bug report, describe the visible symptom and the app revision. Do not post
raw logs, local databases, MCP credentials or private player identifiers publicly.
