---
title: Analysis assumptions
description: Exact draw probabilities, Monte Carlo experiments, tutors, equipment and model boundaries.
---

## Ask for a measurable event

“Enough Equipment” is not one probability. Distinguish Equipment in play,
Equipment attached, and Equipment attached to an attack-ready creature. A joint
goal asks whether all selected thresholds hold by the specified turn.

The Lab runs the native Rust engine with a frozen deck snapshot, parameters,
seed, model version and policy version. Saved results include trial denominators
and Wilson 95% confidence intervals. More trials reduce sampling uncertainty;
they do not fix an incorrect or incomplete model.

## Exact calculations and simulations

Exact draw calculations use finite-deck probabilities. The joint category
calculation requires disjoint categories; overlapping categories cannot simply
be multiplied as independent events.

Monte Carlo experiments use deterministic seeded shuffles. The London mulligan
model draws a new full hand and bottoms physical cards according to its stated
policy. A replay requires the same seed, snapshot, parameters and model versions.

## Costs, tutors and turns

The equipment model spends mana on tutors before deploying searched cards. Tutor
costs, their available targets and destinations, land choices and supported draw
or selection effects affect when a battlefield goal can be reached. It uses a
named greedy policy, not an optimal search over every possible line.

Printed mana value, alternate or Adventure costs, equipment deployment and equip
costs are separate concepts. Curves do not prove a card is castable. Conditional
sources must be evaluated for the spell and turn; ramp or Treasure assumptions
must be stated explicitly rather than treated as extra lands.

## Legendary duplicates

Physical copies remain in the deck and can be drawn together. Supported legendary
permanents do not inflate the battlefield count: redundant copies can be bricks.
A tutor cannot produce a missing copy that is no longer in its supported search
zone. Sideboard cards do not quietly become main-deck copies.

## Unsupported interactions

The current equipment experiment is a bounded precombat goldfish. It does not
model an opponent, combat outcomes, every triggered ability or every Magic rule.
The compiler checks supported card profiles and rules hashes; unsupported decks
are blocked instead of being assigned an invented result.

Read the Lab's model boundaries and card-interaction notes before running. A
simulation reports the stated event under those assumptions, not the strength of
a deck in a metagame. Alkemyst does not encode speculative Arena opening-hand
smoothing or a universal land-count recommendation as fact.

## Observations are separate evidence

Captured performance comes from observed games with visible sample counts and
missing-data indicators. Simulation probabilities are model estimates. Keep both
alongside the deck version, play/draw setting and experiment parameters when
comparing a proposed change.
