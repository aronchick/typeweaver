# TypeWeaver OpenSpec — Implementation Roadmap

## Status
Draft

## Purpose
Turn the current TypeWeaver product/spec set into an execution sequence with concrete milestones.

This roadmap is intentionally product-first rather than code-first. It answers:
- what should be built next
- in what order
- why that order is the highest leverage
- what each phase must prove before the next one begins

## Source inputs
This roadmap is derived from:
- `docs/openspec-product.md`
- `docs/openspec-scoring-model.md`
- `docs/openspec-attack-matrix.md`
- `docs/openspec-demo-flow.md`
- `docs/canonical-plan.md`
- the current repo state (core, API, UI, OCR, metrics, deployability already present)

## Strategic framing
TypeWeaver already has a meaningful technical base.

What it does not yet have is the product loop that makes the system feel inevitable:
1. show a font under hostile conditions
2. score the failure clearly
3. generate stronger variants
4. let the user compare tradeoffs
5. export the winner

The roadmap should therefore prioritize product leverage in this order:
- make evaluation undeniable
- make repair compelling
- make generation expressive
- make optimization automatic

## North star
A user can upload a font or describe a vibe, test candidates under realistic hostile conditions, generate stronger variants, compare tradeoffs across robustness / vibe / weight, and export the best result.

## Phase 0 — Consolidate the spec baseline

### Goal
Make the repo self-describing so future implementation work has a stable product target.

### Scope
- keep `README.md` aligned with the product vision
- keep `docs/canonical-plan.md` as the reconstructed long-range plan
- land the OpenSpec docs as the planning source of truth
- identify any contradictions between repo behavior and specs

### Exit criteria
- repo contains a coherent doc stack
- builder can understand the intended product without external chat context
- next implementation phases can reference named docs instead of raw conversation history

### Status
Done enough to proceed

## Phase 1 — Scoring Model v1

### Goal
Define a concrete, implemented score model that is simple enough to ship but rich enough to guide both users and future mutation loops.

### Why first
Without a scoring model, the rest is mush. The attack matrix has nothing to aggregate, the UI has nothing to present, and mutation has no optimization signal.

### Scope
- choose the first top-line score shape
- choose the first subscore set
- define a stable score schema in code and JSON
- define target presets and first weighting strategy
- decide how worst-case and confusion-pair penalties work
- produce score explanations suitable for UI rendering

### Recommended v1 score contents
- overall score
- robustness score
- distinctiveness score
- cost score
- condition breakdown
- worst-case score
- top failure summary

### Key decisions required
- top-line scale (100 vs 1000 vs normalized percentile)
- weighting approach for presets
- whether machine-readability is in v1 or deferred
- how hard file size should penalize the total
- whether unsafe confusion pairs can hard-cap the score

### Exit criteria
- score JSON contract exists and is stable
- at least one preset weighting scheme is implemented
- score outputs are understandable without reading source code
- existing benchmark outputs can be upgraded to the new schema

## Phase 2 — Attack Matrix v1

### Goal
Implement the first meaningful hostile-condition matrix that powers the permutation explorer.

### Why second
Once the score model exists, the next highest-value move is to generate the evidence that feeds it.

### Scope
- implement deterministic condition primitives
- implement severity ladders
- implement a first compact scenario bundle system
- add confusion-pair stress probes
- map condition outputs into the score schema

### Recommended v1 matrix
- baseline
- blur: 3 levels
- low contrast: 3 levels
- small size: 3 levels
- raster degradation: 2 levels
- compression: 2 levels
- confusion-pair probes under baseline, blur, and low contrast

### Key decisions required
- exact severity definitions
- which scenario presets ship first
- which combined attacks become first-class
- compute budget per run

### Exit criteria
- one font can be evaluated across the full v1 matrix reproducibly
- condition outputs feed the score system cleanly
- worst-case condition and collapse threshold can be identified
- the matrix is strong enough to visibly differentiate fonts

## Phase 3 — Permutation Explorer UI

### Goal
Build the first product experience that shows TypeWeaver’s value without explanation-heavy handholding.

### Why third
The scoring model and attack matrix become compelling only when users can see the same font succeed and fail across conditions.

### Scope
- scenario selector
- condition grid / permutation view
- side-by-side baseline vs degraded comparison
- score dashboard
- failure hotspot summaries
- worst-case condition surfacing
- curated first-run examples

### Key UX behaviors
- show failure early
- keep the score breakdown visible
- let users toggle text samples / confusion sets / sizes
- make the worst-case condition obvious

### Exit criteria
- user can upload or select a font
- user can choose a scenario
- user can see multiple hostile-condition previews
- score changes and failure hotspots are visible and understandable
- the UI is compelling enough for a live demo

## Phase 4 — Repair / Variant Generation v1

### Goal
Let users request stronger variants from an existing font.

### Why fourth
This is the first moment the product does something active and valuable beyond diagnosis.

### Scope
- define the first mutation/repair action model
- generate multiple candidate variants
- score candidates under the same condition matrix
- rank variants against the original
- explain what changed in each variant

### Likely initial focus
Start with guided repair rather than unconstrained font invention.

Examples:
- strengthen ambiguous pairs
- widen apertures
- improve small-size behavior
- trade small aesthetic drift for large robustness gains

### Exit criteria
- starting from one font, the system can produce several candidate repairs
- variants can be scored and compared side by side
- the user can understand why a candidate improved or regressed

## Phase 5 — Comparison Frontier + Export v1

### Goal
Make the outcome practical and decision-oriented.

### Why fifth
Without comparison and export, TypeWeaver risks becoming a lab toy instead of a product.

### Scope
- compare original vs variants
- sort/rank by score, worst-case, size, vibe retention
- show tradeoff summaries
- export chosen font variant
- export score report / specimen / provenance bundle

### Exit criteria
- user can select a winning candidate
- user can download it
- user can preserve the evidence behind that choice

## Phase 6 — Vibe-Driven Generation

### Goal
Let users create candidate fonts from intent instead of only repairing an existing font.

### Why sixth
The generation story is exciting, but the evaluation-and-repair loop should be trustworthy first.

### Scope
- define vibe input model
- support prompts, constraints, and references
- create candidate generation workflows
- evaluate generated candidates with the same score system
- surface robustness / vibe / weight tradeoffs

### Key questions to answer here
- how much of vibe is prompt-driven vs slider-driven
- whether outputs are novel, blended, or guided mutations
- how intent-fit is scored

### Exit criteria
- user can provide intent and get candidates
- candidates are evaluated under target conditions
- user can compare them meaningfully, not just aesthetically

## Phase 7 — Adversarial Optimization Loop

### Goal
Automate the attack → mutate → rescore → rank cycle.

### Why seventh
This is the deepest differentiator, but it needs the earlier layers to be credible first.

### Scope
- adaptive attack selection
- mutation policy informed by failure modes
- generation-over-generation search
- stopping rules and budget control
- frontier visualization
- progress/history view

### Optimization targets under consideration
- maximize robustness under vibe constraints
- maximize vibe fit above a robustness floor
- maximize robustness-per-kilobyte
- produce a Pareto frontier instead of a single winner

### Exit criteria
- the system can iterate across multiple generations automatically
- optimization progress is visible and interpretable
- users can choose from a tradeoff frontier, not just a single opaque best result

## Phase 8 — Scenario Packs and Advanced Targets

### Goal
Turn the core loop into a family of target-specific products.

### Scope
- dashboard UI preset
- mobile UI preset
- signage preset
- OCR/camera preset
- field/defense preset
- print/label preset
- future multilingual or script-specific expansions

### Exit criteria
- presets are meaningful and distinct
- product can be demoed credibly in different vertical contexts
- target-specific score weighting and attack bundles feel intentional

## Cross-cutting workstreams

## Workstream A — Data contracts
Needs to progress alongside all phases.

Includes:
- score JSON schema
- condition result schema
- variant/provenance schema
- export bundle schema

## Workstream B — Explanation layer
Needs to progress alongside scoring, UI, and mutation.

Includes:
- failure summaries
- mutation explanations
- tradeoff explanations
- scenario descriptions

## Workstream C — Provenance and legal clarity
Important before export becomes a central feature.

Includes:
- base-font provenance tracking
- derivative metadata
- license safety rules
- export restrictions where needed

## Workstream D — Performance and compute budgeting
Important once the attack matrix and optimization loop expand.

Includes:
- evaluation time targets
- caching strategy
- budget caps per run
- UI expectations for async work

## Workstream E — Demo readiness
Should be maintained continuously.

Includes:
- curated sample fonts
- curated target scenarios
- default comparison texts
- narrative demo flows

## Suggested milestone sequence

### Milestone M1
Score schema and score preset decision

### Milestone M2
Deterministic attack matrix working end-to-end

### Milestone M3
Permutation explorer demoable in browser

### Milestone M4
Repair variants generated and ranked against original

### Milestone M5
Comparison frontier + export available

### Milestone M6
Vibe-driven generation working

### Milestone M7
Closed-loop adversarial optimizer working

## What not to do yet
- do not lead with unconstrained font generation before scoring is trustworthy
- do not overbuild a broad SaaS shell before the core product loop is compelling
- do not hide the score logic in a black box
- do not treat export as an afterthought if the goal is practical use

## Immediate next actions
The most rational next execution moves are:
1. convert this roadmap into tracked implementation issues
2. create a score-model decision document for v1
3. decide the initial attack matrix exact primitives and severity ladders
4. define the first browser permutation explorer interaction contract

## Recommendation
The best execution order remains:
1. scoring model
2. attack matrix
3. permutation explorer
4. repair variants
5. comparison + export
6. vibe generation
7. adversarial optimization loop

That order gives TypeWeaver a believable wedge early while preserving the ambitious end state.

## Related documents
- `docs/openspec-product.md`
- `docs/openspec-scoring-model.md`
- `docs/openspec-attack-matrix.md`
- `docs/openspec-demo-flow.md`
- `docs/canonical-plan.md`
