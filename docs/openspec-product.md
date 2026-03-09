# TypeWeaver OpenSpec Product Document

## Status
Draft

## Document purpose
This document captures the intended product shape for TypeWeaver as discussed after the initial MVP, hardening, service, and observability phases.

It is deliberately broader than the original MVP brief. The MVP proved that deterministic ingestion, rendering, and benchmarking were feasible. This document defines the larger product the system is supposed to become.

This is an OpenSpec-style product doc, not an implementation ticket. It is meant to align product intent, scoring direction, UX priorities, and unanswered design questions before the next major build phase.

## One-line summary
TypeWeaver is a system for evaluating, generating, mutating, and optimizing fonts so they remain legible and useful under real-world degradation.

## Core thesis
Most font tools optimize for how a typeface looks in ideal mockups.

TypeWeaver should optimize for how a typeface survives reality.

That means users should be able to:
- preview fonts under hostile conditions
- understand where and why they fail
- generate or mutate better candidates
- score tradeoffs across robustness, aesthetics, and weight
- export the strongest result for actual use

## Product vision
TypeWeaver is a typography robustness lab, generative font workbench, and adversarial optimization system.

The product has three tightly connected loops:

1. **Evaluation loop**
   - inspect a font under realistic conditions
   - score its behavior
   - surface weaknesses

2. **Generation loop**
   - create candidate fonts from vibe, constraints, or base references
   - compare them against each other and against imported fonts

3. **Optimization loop**
   - attack fonts with increasingly relevant degradation or mutation scenarios
   - learn from failure modes
   - generate or mutate stronger variants
   - iterate until the system finds better tradeoffs

## User promise
A user should be able to say one of two things:

### Evaluation-first promise
"Show me whether this font still works in the real world."

### Generation-first promise
"Help me create a font with this vibe that is also robust in the real world."

TypeWeaver should support both journeys in a shared system.

## Target users
Likely early users include:
- product designers
- design systems teams
- accessibility teams
- industrial / embedded UI teams
- signage and field-ops designers
- branding teams with legibility constraints
- OCR / computer-vision adjacent teams
- researchers studying glyph robustness or recognizability

## Product pillars

## Pillar 1 — Real-world permutation explorer

### Goal
Let users see how fonts behave across hostile conditions instead of only pristine previews.

### Core behavior
The user selects or uploads a font and TypeWeaver renders it across a condition matrix.

Examples of conditions include:
- low contrast
- blur
- small-size rendering
- downsampling artifacts
- compression noise
- poor rasterization
- hostile or degraded display conditions
- camera-capture or rephotography distortions
- mutation scenarios

### Output
The system should show:
- side-by-side visual permutations
- score changes by condition
- confusion-pair failure hotspots
- readable explanation of why the score changed
- a summary of best-case, typical-case, and worst-case behavior

### Why this matters
This is the fastest path to proving product value. Even before generation or closed-loop mutation is mature, the permutation explorer gives users an immediate answer to a real question: "Will this font hold up?"

## Pillar 2 — Structured robustness scoring

### Goal
Replace vague visual judgment with a score system that is interpretable and tunable.

### Principles
The score should not be a single mysterious number. It should be a composed model with visible sub-scores.

### Candidate score dimensions
The exact schema remains open, but likely dimensions include:
- readability under ideal conditions
- readability under low contrast
- readability under blur
- small-size resilience
- confusion-pair distinctiveness
- OCR / recognizer stability
- mutation resilience
- byte size / font weight cost
- vibe / intent fit

### Top-line scoring model
The system may still provide a headline score, but it should be derived from a visible weighted breakdown.

### Score profile presets
Different users may want different target presets, for example:
- mobile UI
- dashboard UI
- signage
- industrial labels
- accessibility-focused reading
- field / defense conditions
- OCR-first environments

## Pillar 3 — Generative font exploration

### Goal
Let users create or steer candidate fonts from intent.

### Core behavior
A user provides one or more of:
- a vibe description
- a base font
- target use conditions
- explicit constraints
- aesthetic references

TypeWeaver then generates or proposes candidate fonts or mutations.

### Example user intents
- "Make this feel like transit signage but less sterile"
- "Give me something utilitarian, warm, and strong under blur"
- "Preserve the feel of this font but separate I, l, and 1 much more aggressively"
- "Make me a compact UI font that survives low contrast at small sizes"

### Input modes under consideration
- natural-language prompt
- sliders / toggles
- reference fonts or moodboards
- parameterized constraints
- combinations of the above

### Key principle
Generated fonts should be evaluated by the same robustness machinery as imported fonts. No special pleading for generated outputs.

## Pillar 4 — Adversarial mutation and optimization

### Goal
Use failure as the signal that drives improvement.

### Core behavior
The system should eventually:
- attack a candidate font with adversarial degradation conditions
- detect where performance collapses
- mutate the font or generate new variants informed by the failure
- rescore the candidates
- keep iterating until improvement stalls or a threshold is reached

### Two kinds of mutation

#### Environmental mutation
Mutate the condition under which the font is tested:
- stronger blur
- worse contrast
- harsher raster damage
- camera / display distortions
- scenario-specific perturbations

#### Font mutation
Mutate the artifact itself:
- glyph shape changes
- spacing changes
- aperture changes
- stroke contrast changes
- confusion-pair separation
- width or weight tuning
- file size / complexity tuning

### Long-term objective
Search for fonts that are:
- robust in the intended use environment
- aligned with the intended vibe
- lightweight enough to ship and use practically

## Pillar 5 — Downloadable, usable outputs

### Goal
The result should be more than a report.

Users should ultimately be able to export:
- a chosen font variant
- a packaged score report
- comparison artifacts or specimens
- provenance and transformation history

### Candidate output formats
- TTF
- OTF
- variable font
- webfont bundle
- report package with score breakdown and conditions

## Product boundaries

### In scope for the larger product
- deterministic font ingestion
- controlled evaluation under hostile conditions
- structured robustness scoring
- generation from vibe or constraints
- mutation and adversarial improvement loops
- downloadable optimized outputs

### Explicit non-goals for now
- trying to become a full general-purpose design suite
- becoming a broad multi-tenant SaaS platform before the core loop is good
- supporting every script/language early
- optimizing for decorative novelty over robustness

## Product principles
- **Reality first.** Optimize for actual conditions, not pristine mockups.
- **Determinism where possible.** Inputs, conditions, and score outputs should be reproducible.
- **Explainability over magic.** Users should understand why a score changed.
- **Shared evaluation path.** Imported and generated fonts go through the same scoring system.
- **Optimization is explicit.** Tradeoffs between robustness, vibe, and weight should be visible.
- **Usable output.** The best result should be exportable and practical to deploy.

## Jobs to be done

### Job 1
When I am choosing a font for a constrained environment, I want to see how it behaves under hostile conditions so I can avoid fonts that collapse in practice.

### Job 2
When I have a font I like but it fails in key conditions, I want the system to suggest stronger variants so I can preserve intent while improving robustness.

### Job 3
When I have only an aesthetic direction, I want to generate candidates that fit that vibe and survive the target environment so I can move from taste to deployable artifact faster.

### Job 4
When I need to justify a typography choice, I want measurable score breakdowns and evidence so I can explain the decision to others.

## Representative user journey

### Flow A — Evaluate an existing font
1. User uploads or selects a font
2. User chooses a target environment or preset
3. TypeWeaver renders permutations across a condition matrix
4. TypeWeaver shows scores and failure hotspots
5. User compares this font against alternatives
6. User optionally asks for repair / optimization suggestions

### Flow B — Generate a robust font from intent
1. User provides vibe, references, and constraints
2. TypeWeaver generates candidate fonts or variants
3. TypeWeaver evaluates them under the chosen conditions
4. TypeWeaver ranks candidates by score breakdown
5. User refines constraints or picks one for further mutation
6. User exports the preferred result

### Flow C — Closed-loop adversarial optimization
1. User starts from a font or generated candidate
2. TypeWeaver runs attacks across degradation scenarios
3. TypeWeaver finds weak points
4. TypeWeaver mutates the font or proposes variants
5. TypeWeaver reruns scoring
6. User reviews a frontier of tradeoffs
7. User downloads the winning version

## Proposed system architecture direction
This section describes the product-level system model rather than exact implementation details.

### Layer A — Intake and registry
- ingest fonts
- normalize metadata and licensing
- track provenance and variants

### Layer B — Rendering and condition engine
- render deterministic corpora
- apply condition presets and perturbations
- support extensible hostile-condition families

### Layer C — Scoring and analysis
- compute structured sub-scores
- detect confusion-pair failures
- analyze breakdowns and worst-case scenarios

### Layer D — Generative and mutation engine
- generate candidates from vibe / constraints
- mutate existing fonts or parameter sets
- adapt mutation strategy using scoring feedback

### Layer E — UX and export layer
- permutation explorer
- side-by-side comparison UI
- optimization controls
- export/download flow

## Likely sequencing
The likely smartest development order is:

### Phase A — Make evaluation undeniable
Build the permutation explorer and a first robust scoring breakdown.

### Phase B — Add guided repair
Allow users to create improved variants from an existing font.

### Phase C — Add vibe-driven generation
Let users create candidates from prompts, references, and constraints.

### Phase D — Add adversarial optimization loop
Automate the attack-mutate-score-repeat cycle.

### Phase E — Finish export and production workflows
Deliver downloadable outputs and polished reporting/provenance.

## MVP vs product distinction
The existing repo already covers much of the deterministic evaluation foundation.

What is still needed for the full product is not "more of the same CLI work." It is the higher-level system that turns those primitives into:
- a condition explorer
- a structured scoring model
- a generation surface
- a mutation engine
- an optimization loop
- a usable export workflow

## Open product questions
These questions are intentionally preserved here because they define the next design pass.

## 1. Scoring questions
1. Should TypeWeaver expose a single top-line score, a weighted breakdown, or both?
2. Should the system optimize for human readability, machine readability, or both?
3. When tradeoffs appear, which should dominate: human legibility, OCR stability, vibe fidelity, or file weight?
4. Should there be target-specific score presets for use cases like dashboards, signage, embedded screens, defense/field conditions, or accessibility?
5. Is robustness primarily about confusion-pair disambiguation, or also paragraph-scale reading comfort and recognition speed?

## 2. Attack and condition questions
6. Which degradation modes matter most in the first release: blur, low contrast, small size, compression, raster artifacts, glare, motion blur, camera capture, printing defects, or others?
7. Should the first condition matrix be deterministic presets, adaptive adversarial attacks, or both?
8. Should TypeWeaver automatically discover the worst-case condition for a font?
9. How should environment mutation and font mutation interact in the loop?
10. Do we want scenario-specific attack suites such as field signage, low-end mobile, or OCR capture?

## 3. Generation and vibe questions
11. What does "vibe together a font" mean operationally: plain-language prompting, sliders, reference-font blending, parametric controls, or all of the above?
12. Should generated outputs be fully novel, guided mutations of a base font, blends of references, or repair passes on an imported font?
13. Which aesthetic axes matter most early: warmth, utility, futurism, signage feel, bureaucratic clarity, friendliness, severity, compactness, or something else?
14. Should vibe fit be an explicit scored dimension or just a generation input?
15. How much brand fidelity matters relative to robustness?

## 4. Optimization-loop questions
16. What exactly is the optimization target: maximum robustness under a vibe constraint, maximum vibe under a robustness floor, best robustness-per-kilobyte, or a Pareto frontier?
17. Should the system return one winner, top-N candidates, or an explorable frontier of tradeoffs?
18. How aggressive should mutation be: subtle repair, moderate reshaping, or near-total redesign if the score improves?
19. Should every optimization step be explainable in human terms?
20. What should stop the loop: no meaningful gain, compute budget, score threshold, weight threshold, or user intervention?

## 5. Output and artifact questions
21. What should users be able to download first: TTF, OTF, variable font, webfont bundle, report package, or all of them?
22. How much provenance should be exported alongside the font: parent font, mutations applied, score history, condition breakdown, and license metadata?
23. What legal constraints apply when mutating and exporting derivative fonts based on uploaded inputs?
24. Should outputs be optimized for named targets like low-contrast dashboard UI, camera OCR, field labels, or distant signage?

## 6. Product-shape questions
25. Is TypeWeaver primarily a research lab, pro design tool, API/service, demoable product, or automated optimizer?
26. Who is the true first beachhead user?
27. Which matters most next: a compelling interactive demo, a rigorous scoring engine, a generation workflow, or the closed-loop optimizer?
28. What is the cleanest wedge: "test your font in reality" or "generate a more robust font"?
29. What is the flagship demo story?
30. How long should the system remain Latin-first before multilingual robustness becomes important?

## Current recommendation
The sharpest near-term product sequence appears to be:

1. **Permutation explorer first**
   - upload/select font
   - simulate hostile conditions
   - show a structured robustness score

2. **Variant generator second**
   - produce repaired or mutated versions
   - compare them side by side

3. **Adversarial optimizer third**
   - automate the search for the best robustness / vibe / weight frontier

This sequence gives TypeWeaver a usable wedge early while still preserving the larger vision.

## Success criteria for the next planning phase
This document will have done its job if it leads to three concrete follow-on specs:
- a scoring-model spec
- an attack / condition-matrix spec
- an interaction / demo-flow spec

## Related documents
- `README.md`
- `docs/canonical-plan.md`
- earlier MVP/hardening task docs in `docs/`
