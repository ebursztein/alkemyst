# Analysis model

The native turn model accepts explicit compiled card abilities. It does not parse
rules text or assume that an unresolved interaction works. Catalog translation,
persisted jobs and the REST/MCP adapters are separate application operations.

`Trial` takes physical card indices partitioned into a retained opening hand and
an ordered library. `BattlefieldRunner` draws London hands from an immutable deck,
keeps a versioned seeded RNG across bounded batches, and records per-turn samples.
A worker can check cancellation between advances. Batch sizes do not change replay
results. Reports distinguish requested and completed trials and include Wilson
95% intervals, retained opening size, mulligans and forced keeps.

## Supported explicit effects

- One land per turn, untap at turn start, normal play/draw opening draw rules.
- One mana per available source; colored, generic, explicit colorless and
  two-color hybrid payment use landlord's maximum bipartite matching.
- Tapped, check-type, fast, slow and life-payment land entry conditions. Basic
  land types are separate from the colors a source produces. Life starts at 20;
  the policy pays life only when it would remain above zero.
- Artifact-, Equipment- and creature-spell-only sources. Spell-only mana cannot
  pay equip abilities. Resources spent on a tutor cannot also cast its target.
- Permanent Equipment and creatures, haste, normal mana-only equip abilities,
  Equipment searches to hand or battlefield with optional printed-mana-value
  limits, and shuffle after a search. Search effects explicitly distinguish spell
  resolution from entering the battlefield; fetched permanents fire only their
  modeled entry effects. Creature tutors retain their body even without a target.
- Legendary duplicate avoidance uses logical card identity; repeated copies do
  not inflate battlefield counts. Observed extra legendary copies remain visible
  as bricks. The card compiler must assign identities by modeled named object,
  independently of printing identifiers.

## Policy and interpretation

The explicit `priority-greedy-v1` policy chooses a land enabling the highest
priority affordable spell, then casts by priority, normalized cast cost, and
identity. It prefers restricted sources when paying. Tutors prefer higher-priority,
cheaper Equipment without knowingly fetching another legendary duplicate. After
casting, it attaches affordable Equipment to a creature already eligible to attack
when available. It does not search all legal play sequences or claim optimal play.

An equip ability can be used the turn Equipment enters; a creature cannot attack
until a later turn unless it has haste. Several Equipment may attach to one
creature. Attack-ready counts Equipment attached to eligible creatures; it does
not measure damage, successful attacks, evasion, or combat outcomes.

The three goal thresholds—Equipment in play, attached Equipment, and Equipment on
attack-ready creatures—are evaluated jointly from each complete trial, never by
multiplying marginal probabilities. Per-turn events are evaluated at the end of
the modeled main phase. A required draw from an empty library sets `decked`; that
trial does not satisfy a live battlefield goal for that or later turns.

Printed mana value, a chosen cast cost and an equip ability cost are distinct
inputs. A battlefield tutor bypasses casting its target but still applies its
printed-value search limit. Exact tiny scenarios, joint multivariate draws, and
free-Equipment draw cases anchor the seeded Monte Carlo tests.

## Boundaries

The current explicit model has no opponent, combat resolution, removal, blockers,
trigger stack, non-mana payments, free mulligan, or Arena smoothing. Equipment
creatures/reconfigure, alternate spell modes, ramp/Treasure, multi-mana sources,
other triggered tutoring, cost reductions, auto-attachment, and other interactions need
specific modeled effects. Inconsistent abilities fail validation. The application
compiler must present unsupported interactions and assumptions before running a
scenario; drawing a card alone is never treated as casting it.
