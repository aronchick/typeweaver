# TypeWeaver OpenSpec — Demo Flow

## Status
Draft

## Purpose
Define the interactive product and demo flow that best communicates TypeWeaver’s value.

This document focuses on what a user should experience, what the primary wedges are, and how the product should present scoring, mutation, and optimization without feeling like an academic lab tool.

## Summary
The first compelling TypeWeaver experience should not begin with full font generation.

It should begin with a clear, visceral workflow:
- upload or select a font
- watch it face reality
- see where it fails
- generate improved variants
- compare tradeoffs
- export the strongest result

That is the shortest path to a demo that feels real and differentiated.

## Primary demo thesis
The demo should make one truth obvious:

**A font that looks fine in a static mockup may fail badly in the real world.**

TypeWeaver wins when it shows:
1. the failure
2. the score impact
3. a stronger alternative
4. the tradeoff clearly enough that the user trusts the result

## Recommended primary wedge
The best near-term wedge is:

**"Test your font in reality, then repair it."**

This is sharper than leading with "generate a font from scratch" because:
- users immediately understand the problem
- failure is visually legible
- the score has context
- the optimization loop feels earned
- it naturally leads into generation and mutation

## Product modes
The system should eventually support three user-facing modes.

## Mode 1 — Evaluate
Goal:
- inspect an existing font under hostile conditions

User action:
- upload a font or select a known font
- choose a target scenario or preset
- inspect condition permutations and scores

Primary outputs:
- score breakdown
- worst-case condition
- confusion-pair hotspots
- scenario bundle results

## Mode 2 — Repair / Mutate
Goal:
- produce stronger variants of an existing font

User action:
- choose optimization goals
- generate repaired or mutated variants
- compare results against the original

Primary outputs:
- ranked variants
- explanations of changes
- robustness / vibe / weight tradeoffs

## Mode 3 — Generate from vibe
Goal:
- create a candidate font from intent rather than a starting artifact

User action:
- provide prompt, references, sliders, or constraints
- generate candidate fonts
- score them under target conditions

Primary outputs:
- multiple candidates
- intent-fit plus robustness comparison
- frontier of tradeoffs

## Recommended flagship demo flow
This should be the canonical product story shown to others.

### Step 1 — Start with a font the user likes
The user uploads a font or picks one from a curated set.

The system immediately shows:
- name / provenance
- baseline preview
- target scenarios available

### Step 2 — Choose a target environment
The user selects a preset such as:
- dashboard UI
- mobile UI
- signage
- field conditions
- OCR capture

This keeps the demo grounded in use rather than abstractions.

### Step 3 — Show the reality wall
The system renders the font across several hostile conditions at once.

The user sees:
- clean baseline
- mild degradation
- medium degradation
- worst-case example
- score shifts for each condition

This should feel immediate and slightly brutal. The point is to surface the hidden weakness quickly.

### Step 4 — Highlight failure hotspots
TypeWeaver should call out:
- top failing confusion pairs
- collapse threshold
- conditions causing the largest drop
- whether the font degrades gracefully or catastrophically

The UI should explain these simply, for example:
- "This font collapses under blur sooner than expected"
- "I/l/1 ambiguity becomes unsafe at 10px low contrast"
- "Most of the score drop comes from narrow counters"

### Step 5 — Offer repair
Once failure is visible, the product should offer a clear next action:

- **Repair this font**
- **Generate stronger variants**
- **Optimize for this scenario**

This is the emotional transition from diagnosis to value creation.

### Step 6 — Show variants as a frontier, not just a winner
The system should generate several candidate repairs, each with visible tradeoffs.

For each variant, show:
- overall score
- robustness score
- worst-case score
- weight / size cost
- vibe retention or drift
- a short explanation of what changed

Examples:
- "Variant A improves blur resilience but grows file size"
- "Variant B preserves the original feel better but remains weaker in OCR"
- "Variant C is the strongest overall but drifts furthest from the original"

### Step 7 — Compare side by side
This is the moment that sells the product.

Users should be able to compare:
- original vs repaired
- multiple repaired variants
- the same text under the same hostile condition
- the score breakdown beside the visual preview

The comparison should make tradeoffs feel concrete rather than theoretical.

### Step 8 — Export the winner
After comparison, the user should be able to:
- download the selected font variant
- download a score report
- keep the comparison artifact or specimen

This makes the product outcome practical.

## First-run UX recommendation
A first-time user should not land on a blank lab interface.

The initial experience should offer:
- a curated set of example fonts
- a one-click scenario such as "Test under hostile conditions"
- a guided story mode for the first run

The first experience should teach the value prop without requiring the user to understand the whole model.

## Demo surfaces
The UI likely needs several distinct surfaces.

## Surface A — Font picker / input
Capabilities:
- upload font
- choose sample fonts
- view provenance / license status
- select source baseline

## Surface B — Scenario selector
Capabilities:
- choose a preset
- inspect what conditions the preset includes
- optionally customize weights or condition families later

## Surface C — Permutation explorer
Capabilities:
- view the same text across conditions
- inspect glyph-level and line-level previews
- toggle sample text, confusion sets, and sizes
- view score overlays

## Surface D — Score dashboard
Capabilities:
- overall score
- subscore breakdown
- worst-case condition
- collapse threshold
- confusion-pair hotspots
- cost / weight metrics

## Surface E — Variant gallery
Capabilities:
- compare repaired or generated variants
- sort by score, worst-case, size, or vibe retention
- inspect change summaries

## Surface F — Export panel
Capabilities:
- choose output artifact
- download font
- download report/specimen/provenance

## Demo storytelling principles
- **Show the failure early.** Do not bury the problem.
- **Make the score legible.** Numbers need visual context.
- **Keep mutation explainable.** Users should understand what changed.
- **Use side-by-side comparison aggressively.** This is the product’s strongest visual move.
- **Ground everything in a target scenario.** "Good" only makes sense relative to use.
- **End with a usable artifact.** Export is part of the story.

## Recommended first scripted demo
This is the likely best investor / collaborator / internal demo flow.

1. Start with a visually attractive but fragile font
2. Select a realistic target like low-contrast dashboard or field signage
3. Show the permutation explorer and watch the score fall
4. Highlight specific failure points like `I/l/1` or counter collapse under blur
5. Ask TypeWeaver to repair/optimize the font
6. Show 3 candidate variants with clear tradeoffs
7. Compare original vs best variant under the same hostile conditions
8. Export the strongest result

That tells the whole product story in a few minutes.

## Generation-first flow
A second strong demo can lead from vibe instead of repair.

### Flow
1. User enters a prompt or chooses aesthetic sliders
2. User specifies target environment
3. TypeWeaver generates several candidates
4. Candidates are immediately scored under the chosen matrix
5. User explores the tradeoff frontier
6. User optionally asks for another optimization pass
7. User exports a chosen font

This is exciting, but it should probably come after the evaluation-and-repair story is solid.

## Adversarial optimizer flow
A more advanced mode should expose the loop itself.

### Flow
1. User selects a font and optimization target
2. TypeWeaver displays active attack families
3. The system iterates through attack → score → mutate → rescore
4. The UI shows progress across generations
5. The user reviews the resulting frontier
6. The user exports the chosen endpoint

This mode is likely better for advanced users and demos once the system is mature.

## What should feel magical
The magic moment is not merely "AI generated a font."

The real magic moment is:
- a font fails in a visible, credible way
- the system explains why
- the system produces stronger options
- the improvement is visible under the exact same hostile condition

That feels earned.

## What should remain boring and trustworthy
- score reproducibility
- condition naming
- export behavior
- provenance and licensing
- comparison mechanics

The system needs some boring trustworthiness underneath the flashy demo.

## UX questions still open
1. How much control should users have over the attack matrix in the first UI?
2. Should the product default to presets only, or allow custom condition builders early?
3. Should mutation explanations be generated text, structured diffs, or both?
4. How many candidate variants should be shown at once?
5. Should the comparison view prioritize full text, single words, glyph grids, or all three?
6. How should vibe retention be shown visually?
7. Should users tune the optimization target directly with sliders?
8. When should export appear in the flow: always visible or only after comparison?
9. How guided should the first-run demo be?
10. Should the product lead with uploaded fonts, curated examples, or both?

## Recommendation
The first polished experience should be:
- scenario-first
- comparison-heavy
- score-visible
- failure-forward
- repair-oriented

In short:
- **Test the font**
- **Expose the weakness**
- **Offer stronger variants**
- **Show the tradeoff**
- **Export the winner**

That is the cleanest demo arc and the strongest wedge.

## Related documents
- `docs/openspec-product.md`
- `docs/openspec-scoring-model.md`
- `docs/openspec-attack-matrix.md`
- `docs/canonical-plan.md`
