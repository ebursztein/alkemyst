# Maintained landlord engine

Upstream: https://github.com/mtgoncurve/landlord
Revision: `18e995af042e61a947c9347d1aefd5a2ec93f9e9` (library 2025.4.1).
Original copyright: 2020 Justin Shrake. The original MIT license is in LICENSE.
The maintained derivative crate is MIT licensed; no upstream source is exempt
from formatting, Clippy, file limits, separate tests, or coverage.

## Adoption map

- `src/bipartite.rs` adopts `lib/src/bipartite.rs`: the same augmenting-path
  matching algorithm, with slices replacing Vec arguments and the unused
  recursion argument removed.
- `src/mana.rs` adapts the `lib/src/hand.rs` auto-tap adjacency construction and
  matching into a reusable payment operation. Each supplied source produces one
  unit, and payment returns the physical source indices consumed.
- `src/london.rs` adapts `lib/src/mulligan/london.rs`'s draw-seven, bounded redraw,
  retain, and bottom workflow. The caller supplies physical cards and policy.
- `src/simulation.rs` modernizes `lib/src/simulation.rs`'s repeated hand sampling
  into a seeded streaming histogram, without retaining every simulated hand.
- Tests port the Adeliz and Clarion mana-assignment regressions and elementary
  draw/mulligan cases using explicit injected card data rather than card macros.

## Changes and boundaries

There is no embedded card database, global card lookup, network access, browser
runtime, or speculative Arena hand smoothing. Alkemyst resolves its local catalog
and compiles model inputs outside this crate. Deprecated upstream web bindings,
set/legality tables, parser macros, database serialization, and website-specific
entry points are intentionally omitted. Native Rust needs no WASM runtime.

The replay protocol uses pinned PCG64-MCG and rejection-sampled Fisher-Yates, with
an explicit version string. London samples a full library and places bottomed
cards after every undrawn card. Upstream's shortened-deck swaps could expose
bottomed cards too early. Retention protects required identities, then the policy's
minimum lands, then explicit priorities with randomized ties while respecting the
maximum lands where possible. A forced keep is
reported when the floor cannot meet the requested constraints. No free mulligan
is assumed. On small decks, seven and the keep floor cap at the actual deck size.

Mana costs distinguish generic from explicit colorless. Two-color hybrid pips
use source alternatives without double-counting a land. X, phyrexian, snow, and
two-brid symbols return unsupported errors; the caller must model their choices
explicitly. One-unit sources do not infer conditional land abilities, ramp, or
multiple mana from rules text. The later turn simulator must provide available
sources and pay costs in sequence; drawing cards is not evidence they were cast.

Exact hypergeometric and joint multivariate calculations provide elementary
references for the Monte Carlo implementation. Joint categories are disjoint,
cover the whole deck, and express simultaneous count bounds. These are draw
probabilities, not battlefield or attack probabilities. All maintained engine
code has an independent 90% line-coverage floor, reported with upstream-derived
code included. No unmodified upstream simulation modules are shipped separately.
