# Manual practice sandbox

Practice deals physical main-deck copies from an immutable deck version and frozen
catalog metadata. It does not require simulator support for the card's rules.
Sideboard, command and companion cards are not shuffled into the main library.
The seeded landlord shuffle is reproducible; it does not model Arena smoothing.

Before keeping, each London mulligan reshuffles the complete deck and deals up to
seven cards. Keep requires exactly one distinct hand card per mulligan, in the
order the player wants them on the bottom. Those cards follow the entire undrawn
library. On the draw, keeping starts turn one with a draw; on the play it does not.
A failed draw records `decked` and finishes practice without creating a match loss.

`NextTurn` untaps the battlefield, resets the tracked land drop and draws one card.
`PlayLand` verifies the selected face is a land and tracks one land play per turn.
`Play` manually moves a nonland card from hand to battlefield or graveyard. Mana
payment, enter-tapped conditions, abilities, attachments, combat and state-based
actions (including the legend rule) remain manual. `Tap` and explicit `Move`
overrides support this sandbox; Move can only select already visible cards and
cannot reveal a library card. These boundaries are returned in every view.

Steps are immutable and append with an expected revision. One SQL transaction
commits the idempotency receipt and next step, so competing actions cannot both
advance the same revision. Retrying a successful action returns its original
step, even after subsequent actions; clients should refetch the latest view after
action completion and avoid replacing a newer view with an older revision.
Session origin and each action's origin are retained independently.

The SQL engine owns saved inputs, states, read caching and invalidation. The public
view contains visible cards and a library count, not an ordered library. Inputs
expose the original deck roster without shuffle order. Reopening the database or
refreshing the catalog cannot rewrite an existing practice session. Versioned
shuffle and sandbox identifiers prevent silently applying changed semantics to an
old session. A session is bounded to thirty turns and one thousand actions.

REST, the generated SDK and explicitly allowlisted MCP tools share creation,
listing, latest-view, frozen-input and action operations. Practice is separate
from captured match history and from automated simulation policies/results.
