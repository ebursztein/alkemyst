# Catalog compiler

`Analysis::profile(version_id)` resolves an immutable version and its local card
metadata with one SQL statement. SQL owns reference joins, printed curve bins,
ordering and type/quantity totals. The engine cache tracks deck, printing, alias,
and catalog revisions; REST, SDK and MCP use the same operation.

`Analysis::prepare` keeps that exact resolved snapshot with the compiled physical
cards. A simulation job must persist this snapshot, compiler/model/policy versions,
parameters and seed, rather than re-resolving metadata while a job is running.

## Model support

A reviewed registry currently covers the sanitized Boros benchmark, plus generic
basic lands and simple creatures with explicitly recognized combat keywords.
Unreviewed abilities, layouts, casting symbols, missing metadata, and changes to
reviewed rules block simulation. A missing printed cost is never treated as zero.
Profiles and printed curves remain available even when simulation is blocked.

Registry entries bind Oracle identity to a SHA-256 fingerprint of name, printed
cost/value, types, rules text, colors, keywords and the corresponding face fields.
Canonical objects use serde_json's sorted keys; printed value is normalized as
f64. Printing IDs, artwork, prices and unrelated metadata are excluded. The
reviewed entries are checked by the real-card fixture compilation tests; errata
require reviewing the model and updating its fingerprint together.

Do not copy a name into a preset and assume its abilities are unchanged. Add or
extend typed engine effects first with separate failing tests, then add a reviewed
profile. Keep individual source/test files under 300 physical lines. Original
compiler code is Apache-2.0; the maintained landlord primitives retain MIT notices.

## Interpreting the profile

- Curves use printed front-side mana value and exclude lands. Unknown values have
  an explicit null bin. A chosen casting cost, Adventure cost and equip cost are
  distinct fields.
- Source counts are potential colors once a source is available/untapped.
  Unrestricted land colors are separate from board-dependent/spending-restricted
  colors and nonland mana abilities. They are not probabilities of usable mana
  on a turn; the turn model evaluates entry, conditions and restrictions.
- Missing sideboard cards remain visible and do not block a main-deck run.
  Sideboarding and companion use are outside this 60-card main-deck model.
- The current scope is precombat main phases without attacks, damage or opponent
  interaction. Card-specific exclusions remain visible; running that bounded
  model requires the caller to select those exclusions explicitly. The profile
  never claims optimal play or a complete Magic rules engine.
