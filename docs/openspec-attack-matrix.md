# TypeWeaver OpenSpec — Attack Matrix

## Status
Draft

## Purpose
Define the hostile-condition matrix TypeWeaver should use to evaluate how fonts perform outside pristine viewing conditions.

This document covers the environmental and adversarial attack side of the system. It complements the scoring-model spec by defining the conditions under which scores are produced.

## Summary
TypeWeaver should test fonts under a structured family of degradations that approximate real-world failure modes.

The attack matrix should support:
- deterministic benchmark presets
- progressive severity levels
- scenario-specific bundles
- adaptive adversarial attacks in later phases

## Design principles
- **Reality-linked:** every attack should map to a real degradation mode or meaningful adversarial stress
- **Layered:** start with deterministic presets, then add adaptive attacks
- **Comparable:** the same font should be re-runnable under the same matrix
- **Composable:** scenario bundles should be built from reusable condition primitives
- **Optimization-aware:** attacks should help identify which mutations to try next
- **Explainable:** users should understand what each attack means

## Attack taxonomy
TypeWeaver should model two broad attack families.

## Family 1 — Environmental attacks
These change the viewing conditions without changing the font itself.

Examples:
- blur
- contrast loss
- downsampling
- raster artifacts
- compression artifacts
- glare / bloom
- motion blur
- camera capture distortion
- print / ink spread
- display gamma or aliasing weirdness

## Family 2 — Font-targeted adversarial attacks
These are designed to reveal weak points in glyph recognition and robustness.

Examples:
- attacks that make confusion pairs more similar in practice
- attacks that stress apertures, counters, and joins
- attacks that collapse thin strokes or spacing at small sizes
- attacks that expose failure in compressed or noisy renderings

## Core condition primitives
The first useful matrix should be built from a small set of reusable primitives.

## Primitive A — Blur
Purpose:
- simulate focus loss, motion softness, cheap optics, and distance softness

Severity ladder:
- blur_1: very mild
- blur_2: mild
- blur_3: medium
- blur_4: strong
- blur_5: severe

Key failure signals:
- counter closure
- stroke merging
- confusion-pair collapse
- line-level smearing

## Primitive B — Contrast loss
Purpose:
- simulate poor text/background separation or washed-out rendering

Severity ladder:
- contrast_1: slightly reduced
- contrast_2: moderate loss
- contrast_3: strong loss
- contrast_4: near-edge readability

Key failure signals:
- thin strokes disappearing
- weak interior detail
- poor figure-ground separation

## Primitive C — Small-size rendering
Purpose:
- simulate constrained UI text, dense dashboards, and compact displays

Candidate sizes:
- 16px
- 14px
- 12px
- 10px
- 8px where appropriate

Key failure signals:
- lost apertures
- fused shapes
- ambiguity between numerals and letters
- spacing collapse

## Primitive D — Raster degradation
Purpose:
- simulate low-quality rasterization, poor anti-aliasing, and pixel-grid stress

Severity ladder:
- raster_1: mild
- raster_2: moderate
- raster_3: harsh

Key failure signals:
- jagged stems
- collapsed bowls/counters
- loss of diagonals or subtle structure

## Primitive E — Compression / encoding artifacts
Purpose:
- simulate image-based workflows, screenshots, messaging apps, and camera pipelines

Severity ladder:
- jpeg_1: mild
- jpeg_2: moderate
- jpeg_3: strong
- jpeg_4: severe

Key failure signals:
- ringing around strokes
- edge halos
- texture noise interfering with recognition

## Primitive F — Glare / bloom / halo
Purpose:
- simulate bright displays, optical flare, or overexposed capture

Severity ladder:
- glare_1
- glare_2
- glare_3

Key failure signals:
- edge washing
- detail softening
- thin negative space disappearing

## Primitive G — Motion blur
Purpose:
- simulate moving viewers, moving cameras, or unstable capture

Severity ladder:
- motion_1
- motion_2
- motion_3

Key failure signals:
- directional smearing
- horizontal character ambiguity
- dense text becoming unreadable faster than isolated glyphs

## Primitive H — Camera capture / rephotography
Purpose:
- simulate the real camera path for OCR, scanning, or reading from a screen/sign

Sub-effects may include:
- perspective warp
- uneven lighting
- sensor noise
- sharpening artifacts
- compression
- moiré

Severity ladder:
- camera_1: gentle capture degradation
- camera_2: realistic handheld degradation
- camera_3: difficult capture

Key failure signals:
- OCR confusion
- character-edge instability
- local distortion causing pair collapse

## Primitive I — Print / ink spread
Purpose:
- simulate printed media, labels, thermal printing, or cheap physical output

Severity ladder:
- print_1
- print_2
- print_3

Key failure signals:
- counters filling in
- joins thickening
- fine detail disappearing

## Primitive J — Noise overlays
Purpose:
- simulate dirty channels, noisy displays, rough physical surfaces, or low-quality captures

Severity ladder:
- noise_1
- noise_2
- noise_3

Key failure signals:
- edge confusion
- fine-detail disruption
- increased recognition variance

## Condition-combination attacks
Single primitives are useful, but real-world failures often come from combinations.

Important combined attacks include:
- blur + low contrast
- small size + low contrast
- small size + raster degradation
- camera capture + compression
- glare + low contrast
- print spread + small size

These combinations should become first-class scenario presets.

## Scenario bundles
Users will think in environments, not primitives. TypeWeaver should group primitives into scenario-specific bundles.

## Scenario: Mobile UI
Likely bundle:
- small-size rendering
- low contrast
- raster degradation
- occasional compression

## Scenario: Dense dashboard
Likely bundle:
- small-size rendering
- low contrast
- confusion-pair stress
- medium raster pressure

## Scenario: Signage / wayfinding
Likely bundle:
- blur
- distance softness
- glare
- camera capture
- motion blur

## Scenario: OCR / document capture
Likely bundle:
- camera capture
- compression
- low contrast
- slight blur
- perspective warp

## Scenario: Field / defense environment
Likely bundle:
- glare
- blur
- low contrast
- motion blur
- camera capture
- variable lighting

## Scenario: Printed labels / industrial output
Likely bundle:
- print spread
- low contrast
- small-size rendering
- noise

## Confusion-pair stress attacks
Beyond environmental attacks, TypeWeaver should run explicit confusion-pair probes.

Examples:
- render known problematic pairs in isolation and in context
- stress them under blur and low contrast
- detect the severity threshold at which they become unsafe
- rank pair-specific failure risk

These are essential because many fonts appear fine globally but fail badly on a small set of high-risk characters.

## Degradation ladders
Each primitive should support a clear severity progression.

Why:
- allows stability scoring
- reveals collapse thresholds
- supports area-under-curve metrics
- helps the optimization loop target the right failure region

A good matrix does not just say "passed blur" or "failed blur." It shows when and how the font falls apart.

## Deterministic vs adaptive attacks

## Phase 1 of attack system: deterministic presets
Start with fixed, reproducible severity ladders and scenario bundles.

Why:
- easier comparison
- easier debugging
- easier trust and product explanation
- strong enough for the first useful explorer

## Phase 2 of attack system: adaptive adversarial attacks
Later, TypeWeaver should intelligently search for a font’s weak points.

Examples:
- adjust blur/contrast combinations to maximize confusion risk
- identify the smallest size where a pair becomes unsafe
- target a font’s narrow counters, thin joins, or ambiguous stems
- search for the cheapest environmental degradation that causes collapse

This is where the system becomes a real adversarial optimizer.

## Recommended first attack matrix
The first implementation should be compact but strong.

Recommended initial matrix:
- baseline
- blur: 3 levels
- low contrast: 3 levels
- small-size: 3 levels
- raster degradation: 2 levels
- compression: 2 levels
- confusion-pair probes under baseline, blur, and low contrast

This is enough to create a persuasive permutation explorer and a first robust score breakdown.

## Metrics the matrix should support
The attack matrix should support producing:
- per-condition scores
- worst-case score
- collapse threshold
- mean hostile-condition score
- area-under-degradation curve
- confusion-pair safety thresholds
- scenario bundle scores

## Mutation feedback hooks
The attack matrix should not just report failure. It should suggest what kind of mutation might help.

Examples:
- blur failures suggest widening apertures or increasing interior clarity
- low-contrast failures suggest stronger stroke presence and figure-ground separation
- small-size failures suggest spacing or detail simplification
- camera/OCR failures suggest stronger pair separation and cleaner silhouette structure

## UX expectations
The user-facing system should show attacks in understandable language, not only internal names.

For example:
- "Low contrast (moderate)"
- "Blur (strong)"
- "Camera capture (realistic handheld)"
- "Small size (10px)"

Users should be able to:
- inspect a single condition
- compare conditions side by side
- view a scenario bundle result
- identify the worst-case condition quickly

## Open questions
1. Which primitive families matter most for the first compelling demo?
2. How many severity levels should each primitive have initially?
3. Which combined attacks should be treated as first-class rather than computed ad hoc?
4. Should worst-case discovery be deterministic search or learned/adaptive search?
5. How realistic do camera and print simulations need to be in the first release?
6. Should scenario bundles come before custom condition builders?
7. Which attacks are essential for OCR-focused use cases?
8. Which attacks most strongly differentiate good fonts from mediocre ones?
9. How much compute budget can the matrix assume per comparison run?
10. How should the matrix expose actionable hints back to the mutation system?

## Recommendation
Start with a disciplined deterministic matrix built from:
- blur
- low contrast
- small size
- raster degradation
- compression
- confusion-pair probes

Then add:
- scenario bundles
- camera / glare / print realism
- adaptive adversarial search

That progression keeps the first product understandable while preserving the long-term adversarial vision.

## Related documents
- `docs/openspec-product.md`
- `docs/openspec-scoring-model.md`
- `docs/openspec-demo-flow.md`
- `docs/canonical-plan.md`
