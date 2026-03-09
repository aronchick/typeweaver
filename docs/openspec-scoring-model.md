# TypeWeaver OpenSpec — Scoring Model

## Status
Draft

## Purpose
Define how TypeWeaver should score fonts so robustness is measurable, explainable, and useful for optimization.

This document is not the final formula. It defines the intended shape of the scoring system, the dimensions that matter, and the tradeoff model needed for future implementation.

## Summary
TypeWeaver should not rely on one opaque score.

It should expose:
- a top-line score for fast ranking
- a structured breakdown for diagnosis
- target-specific weighting presets
- enough detail to drive mutation and adversarial optimization loops

## Scoring principles
- **Explainable:** users should see why a score changed
- **Composable:** top-line score is built from visible subscores
- **Target-aware:** different environments weight different failures differently
- **Comparable:** scores should make side-by-side comparisons easy
- **Optimization-friendly:** score dimensions should help the system mutate fonts intelligently
- **Penalty-aware:** file size and complexity should count, not just raw robustness

## High-level model
Each font evaluation should produce:

1. **Overall score**
   - one headline number for ranking and quick scan

2. **Dimension scores**
   - interpretable subscores for robustness characteristics

3. **Condition scores**
   - performance under specific hostile conditions

4. **Failure diagnostics**
   - confusion hotspots, collapse thresholds, worst-case scenarios

5. **Tradeoff metrics**
   - file size, complexity, vibe fit, and other costs

## Proposed scoring layers

## Layer 1 — Core legibility score
Measures how well the font performs under baseline readable conditions.

Candidate components:
- glyph clarity
- spacing clarity
- recognizability at standard sizes
- paragraph / line reading comfort where relevant

This is the "is the font fundamentally readable at all?" layer.

## Layer 2 — Robustness score
Measures how well the font survives hostile conditions.

Candidate hostile-condition families:
- blur
- low contrast
- small-size rendering
- raster degradation
- compression artifacts
- glare / bloom / halo effects
- camera-capture distortions
- motion blur
- print / ink spread where relevant

This is the main score family for the product’s core wedge.

## Layer 3 — Distinctiveness score
Measures how clearly the font separates lookalike glyphs.

Candidate confusion sets:
- `O / 0`
- `I / l / 1`
- `S / 5`
- `B / 8`
- `rn / m`
- `cl / d`
- custom target-specific pairs

This matters disproportionately in hostile environments.

## Layer 4 — Stability score
Measures how gracefully the font degrades instead of catastrophically collapsing.

Signals might include:
- score drop slope as conditions worsen
- first-collapse threshold
- variance across condition families
- resilience near the edge of failure

A font that stays decent across many conditions may be preferable to one that is excellent in ideal conditions but falls apart quickly.

## Layer 5 — Machine-readability score
Optional or weighted differently depending on use case.

Signals might include:
- OCR recognition stability
- character-level machine confusion rate
- robustness under camera / scan distortions

This matters more for OCR-heavy or machine-read environments.

## Layer 6 — Intent-fit score
Measures how well a font matches the user’s requested vibe or design direction.

Possible signals:
- similarity to references
- fit to chosen aesthetic sliders
- preservation of source-family character during repair
- evaluator or model-based vibe agreement

This should not override legibility, but it should matter when comparing candidates.

## Layer 7 — Cost score
Measures deployment cost and practical weight.

Candidate components:
- file size
- glyph set size if relevant
- complexity or rendering overhead proxies
- web delivery cost

The product vision explicitly values robust but lightweight outputs, so cost cannot be ignored.

## Proposed score breakdown
A scored font should likely produce a structure like:

- overall_score
- legibility_score
- robustness_score
- distinctiveness_score
- stability_score
- machine_score
- intent_fit_score
- cost_score
- condition_breakdown
- confusion_breakdown
- failure_summary
- optimization_notes

## Overall score composition
The overall score should be a weighted composition of visible dimensions.

Illustrative structure:

```text
overall_score =
  w_legibility * legibility_score +
  w_robustness * robustness_score +
  w_distinctiveness * distinctiveness_score +
  w_stability * stability_score +
  w_machine * machine_score +
  w_intent * intent_fit_score -
  w_cost * cost_penalty
```

The exact weights should vary by preset.

## Target presets
Different product contexts should emphasize different dimensions.

## Preset: Mobile UI
Priorities:
- small-size resilience
- low-contrast performance
- line-height and spacing clarity
- moderate weight penalty

## Preset: Dashboard UI
Priorities:
- small-size readability
- confusion-pair safety
- low-contrast resilience
- dense-information clarity

## Preset: Signage / Wayfinding
Priorities:
- distance readability
- blur tolerance
- distinctiveness
- fast recognition under stress

## Preset: OCR / Camera Capture
Priorities:
- machine-readability
- character separation
- resilience to camera distortion and compression

## Preset: Accessibility / High-Stress Reading
Priorities:
- legibility
- confusion-pair separation
- stability under low contrast
- conservative aesthetic tradeoffs

## Preset: Field / Defense Conditions
Priorities:
- hostile-environment robustness
- distance recognition
- camera / glare / blur tolerance
- strong collapse resistance

## Condition scoring
Every hostile condition should yield its own score band.

Example condition breakdown:
- baseline
- blur_low
- blur_med
- blur_high
- contrast_low
- contrast_very_low
- small_12px
- small_10px
- compression_med
- compression_high
- camera_capture
- motion_blur

The exact condition list belongs in the attack-matrix spec, but the scoring model needs to accept per-condition outputs and compose them.

## Worst-case and frontier metrics
In addition to means and weighted sums, TypeWeaver should capture:
- worst-case condition score
- median hostile-condition score
- collapse threshold
- robustness area-under-curve across condition severity
- best robustness-per-kilobyte ratio

These are especially valuable for optimization loops.

## Distinctiveness and confusion scoring
Confusion should be modeled explicitly, not hidden in general readability.

For each confusion set, TypeWeaver should estimate:
- baseline distinguishability
- distinguishability under hostile conditions
- threshold where a pair becomes unsafe
- contribution of that pair to the total penalty

Possible outputs:
- pair safety score
- confusion risk index
- top failing confusion sets

## Stability and degradation slope
A font should be rewarded for degrading gracefully.

Two fonts might share the same average score, but one may collapse abruptly once blur or contrast crosses a threshold. That should be penalized.

Useful stability metrics:
- score delta between adjacent severity levels
- collapse point
- max drop between steps
- variance across related attacks

## Cost / weight modeling
TypeWeaver should not blindly reward more complex or heavier fonts if the improvement is marginal.

Candidate cost metrics:
- raw file size
- compressed file size
- complexity proxies
- number of special alternates / features where relevant

Possible derived metrics:
- robustness per KB
- robustness gain per mutation step
- marginal score improvement vs added size

## Intent-fit scoring
This is the hardest subjective dimension and should be separated from hard robustness metrics.

Possible approaches:
- similarity to chosen references
- structured aesthetic sliders mapped to evaluator heuristics
- semantic prompt evaluation using a rubric
- human ranking input where available

Intent-fit should likely be a visible dimension, not a hidden adjustment.

## Score interpretation UX
The UI should answer these questions quickly:
- Which font is strongest overall?
- Which font is strongest under my target conditions?
- Where does each font fail?
- What did I sacrifice to gain robustness?
- Is this improvement worth the added size or reduced vibe fidelity?

## Optimization use
The scoring system is not just for display. It should guide mutation.

Examples:
- low distinctiveness score on `I / l / 1` suggests aperture or stroke differentiation changes
- collapse under blur suggests heavier separation and clearer counters
- poor robustness-per-KB suggests simplifying the design while preserving clarity
- weak intent-fit suggests mutation drifted too far from the requested vibe

## Recommended first implementation shape
A practical first scoring version should include:
- overall score
- robustness score
- distinctiveness score
- cost score
- condition breakdown
- worst-case score
- top failure summary

That is enough to support the first useful permutation explorer without overbuilding the subjective layers.

## Open questions
1. Should the top-line score be normalized to 100, 1000, or a percentile-style ranking?
2. Should human and machine readability be separate top-level axes or merged under robustness?
3. Which should dominate early presets: average hostile-condition score or worst-case score?
4. How much should file size penalize the total score?
5. Should intent-fit affect the top-line score or remain a side metric initially?
6. Do we want a hard fail floor for unsafe confusion pairs?
7. Should some conditions be mandatory for all presets and others optional?
8. Should users be able to customize weightings directly?
9. How should missing data be handled when some condition scores are unavailable?
10. What score outputs are required for mutation loops vs human-facing UI?

## Recommendation
Start with a visible, decomposition-friendly score system built around:
- hostile-condition robustness
- confusion-pair distinctiveness
- worst-case performance
- cost penalty

Then layer in:
- machine-readability
- vibe / intent-fit
- richer stability modeling

That path keeps the product sharp and gives the optimization loop something concrete to learn from.

## Related documents
- `docs/openspec-product.md`
- `docs/openspec-attack-matrix.md`
- `docs/openspec-demo-flow.md`
- `docs/canonical-plan.md`
