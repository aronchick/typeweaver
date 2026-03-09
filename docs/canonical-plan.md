# TypeWeaver Canonical Plan

This document is the working source of truth for what TypeWeaver is supposed to become.

It began as a reconstruction of the original MVP brief and the later hardening / deployment passes. It now also captures the broader intended product direction:
- evaluate fonts under real-world degradation
- let users generate or "vibe together" candidate fonts
- score those fonts for robustness
- mutate and attack them adversarially
- iteratively search for fonts that are both robust and lightweight

## Core idea

TypeWeaver is not just a font preview tool.

It is a system for testing, generating, mutating, and optimizing fonts against reality.

The end state is a loop like this:
1. start from an existing font or a generated candidate
2. render and evaluate it under hostile conditions
3. identify failure modes
4. mutate or regenerate the font
5. rescore it
6. repeat until the system finds the best tradeoff between robustness, aesthetics, and file weight

## User-facing promise

A user should be able to:
- upload or ingest a font
- see permutations of that font under blur, contrast loss, and other real-world conditions
- get a score rather than just a visual impression
- generate new fonts from a desired vibe or set of constraints
- compare generated variants against existing ones
- tune fonts toward specific scenarios
- download the resulting font for their own use

This should work for both:
- **evaluation-first users**: "How bad does this font get under hostile conditions?"
- **generation-first users**: "Make me something with this vibe, but robust in the real world."

## Product pillars

## Pillar 1 — Real-world font evaluation

### Goal
Show how a font behaves outside pristine design mocks.

### Required condition families
The exact matrix can evolve, but the system should support conditions such as:
- low contrast
- blur
- small-size rendering
- noisy rasterization
- compression artifacts
- hostile display environments
- mutation scenarios that approximate damage or degradation

### Output expectations
For any condition set, the system should provide:
- visual previews
- structured measurements
- confusion-pair analysis
- a score or score breakdown
- enough traceability to understand *why* a font failed

## Pillar 2 — Generative font exploration

### Goal
Let users move from intent to candidate fonts quickly.

### User interaction model
A user should be able to express a direction such as:
- a vibe
- a target use case
- a robustness goal
- a set of constraints

And TypeWeaver should be able to:
- generate candidate fonts or font variants
- surface multiple permutations
- compare them against both aesthetic and robustness criteria

### Important principle
Generated fonts should not be judged only by taste.
They should be passed through the same evaluation machinery as imported fonts.

## Pillar 3 — Adversarial robustness loop

### Goal
Use failure as the optimization signal.

The system should eventually:
- attack a font with increasingly relevant degradation scenarios
- inspect the observed failures
- adapt the mutation strategy based on those failures
- produce new variants
- re-evaluate them automatically

This is not just benchmarking. It is an optimization loop.

### Long-term optimization target
Search for fonts that are:
- maximally robust in target conditions
- still true enough to the intended vibe
- lightweight enough to be practical to ship and use

## Pillar 4 — Downloadable outcomes

### Goal
The output should be usable, not just interesting.

Users should ultimately be able to:
- select a winning font variant
- export or download it
- understand what tradeoffs were made
- reuse it in actual products and environments

## Reconstructed history

## Phase 1 — Core MVP

### Original goal
Prove the deterministic local evaluation loop.

### Original in-scope work
- Rust workspace and crate scaffolding
- shared types
- asset registry
- license normalization
- local ingest CLI
- fixed Latin corpus rendering
- benchmark runner
- two benchmark profiles
- JSON report card output
- sample fixtures
- unit + integration tests

### Original explicit constraints
- Rust implementation
- prefer MIT / Apache-2.0 dependencies
- Latin script only
- deterministic output
- file-backed storage over heavyweight infra
- no font generation yet
- no OCR in the original MVP scope
- no URL auditing
- no website UI in the original MVP scope

### Original policy rules
Approved licenses:
- Public Domain
- CC0
- MIT
- Apache-2.0

Rejected or quarantined for the initial phase:
- OFL
- mixed / ambiguous licensing
- unknown / missing licenses

### Original required benchmark profiles
- `web_light_default`
- `mobile_dark_low_contrast`

### Original acceptance criteria
- `cargo test` passes
- `cargo run -- ingest <dir>` indexes candidate fonts and marks approval status
- `cargo run -- bench <font-id> --profile web_light_default` emits a JSON report
- sample fixtures included
- README includes setup and usage

## Phase 1.5 — Hardening pass

### Goal
Harden correctness, determinism, and CLI usability without expanding scope.

### Focus areas recovered from the hardening prompt

#### CLI ergonomics
- clear help text
- consistent commands and flags
- predictable output paths
- better errors for missing font IDs, bad profiles, and missing directories

#### Registry correctness
- tighter license normalization
- explicit approval / quarantine / rejection behavior
- deterministic registry JSON read/write
- duplicate avoidance using stable identifiers

#### Benchmark/profile correctness
- exact profile names verified
- explicit profile parsing/selection
- deterministic benchmark output

#### Report contract
- stable JSON schema
- clean field names
- sample fixture report

#### Test coverage
- unit tests for normalization edge cases
- end-to-end integration tests for ingest + bench
- zero-warning green test runs

#### Repo hygiene
- ignore `target/` and `.typeweaver/`
- do not commit build artifacts

## Phase 2 — Service layer

### Goal
Move from CLI-only evaluation to a local service and browser-accessible product.

### Scope recovered from repo history
- `typeweaver-api` crate
- `serve` subcommand
- REST endpoints
- embedded static web UI
- shared deterministic core pipeline underneath

### Role in the larger vision
This is the layer that turns TypeWeaver from a dev tool into something a user can actually operate interactively.

## Phase 3 — Deployability

### Goal
Make TypeWeaver easy to run as a real service.

### Scope recovered from repo history
- Dockerfile
- systemd unit
- Caddy config
- deploy script
- GitHub Actions build + deploy workflow
- corrected Hetzner path / port wiring

### Deployment assumptions from session history
- primary target was Hetzner at `/opt/typeweaver`
- systemd + tmux were used during bring-up
- Caddy fronts the service

## Phase 4 — Observability and production readiness

### Goal
Make the system inspectable enough to support iteration and operations.

### Scope recovered from memory + commits
- OpenTelemetry tracing
- Prometheus metrics
- bearer-token auth
- richer health endpoints
- split liveness from deeper health
- `okz` / `healthz` / metrics-style operational visibility

### Why this matters
If TypeWeaver is going to run optimization loops, batch evaluations, and eventually generation/mutation pipelines, observability is not optional.

## Canonical system model

The architecture implied by the original plan and current repo is:

- `typeweaver-core`
  - shared types
  - corpus definitions
  - benchmark profile definitions
  - report models
- `typeweaver-registry`
  - ingest logic
  - license normalization
  - registry persistence
  - approval / quarantine / rejection policy
- `typeweaver-render`
  - deterministic rendering
- `typeweaver-bench`
  - benchmark execution
  - report assembly
- `typeweaver-cli`
  - ingest / list / profiles / bench / serve
- `typeweaver-api`
  - service layer over the same pipeline
- `typeweaver-ocr`
  - OCR scoring and related evaluation work

## What the scoring system should grow into

The final score probably should not be one opaque scalar.

It should likely become a structured scoring model spanning dimensions like:
- legibility under ideal conditions
- legibility under blur
- legibility under low contrast
- small-size resilience
- confusion-pair distinctiveness
- OCR / recognizer stability where appropriate
- mutation resilience
- byte size / weight cost
- fit to stated vibe or design intent

A single top-line score can still exist, but it should be composed from intelligible parts.

## What mutation means here

Mutation is central to the vision.

It can mean at least two things:

### 1. Environmental mutation
Change the viewing conditions:
- blur
- contrast collapse
- raster damage
- noise
- hostile rendering contexts

### 2. Font mutation
Change the artifact itself:
- glyph shape variants
- spacing / width / weight adjustments
- contrast adjustments inside the letterforms
- confusion-pair separation tuning
- parameter search across generated candidates

TypeWeaver should eventually learn from both kinds.

## Intended optimization loop

A likely long-term loop looks like this:

1. User provides:
   - an existing font, or
   - a vibe / direction / constraint set
2. TypeWeaver generates one or more candidates
3. TypeWeaver evaluates those candidates under a battery of hostile conditions
4. TypeWeaver identifies failure hotspots
5. TypeWeaver mutates or regenerates candidates informed by those failures
6. TypeWeaver reruns evaluation
7. TypeWeaver ranks candidates by robustness, weight, and intent-fit
8. User inspects and downloads the preferred outcome

## Current status against the real vision

### Already present
- deterministic evaluation core
- registry + ingest
- benchmark/reporting foundation
- service/UI/deploy layer
- OCR-related work
- observability and metrics

### Still missing relative to the long-term vision
- richer degradation condition matrix
- explicit user-facing permutation explorer
- formal scoring breakdown for hostile conditions
- vibe-driven generation UX and model
- font mutation engine
- adversarial attack orchestration
- automated optimization loop
- polished download/export workflow for generated outcomes

## Strategic next steps

If development resumes, the highest-leverage order is probably:

### 1. Define the scoring model
Lock down what "robust" means in measurable terms.

### 2. Build the condition matrix
Expand from a couple of profiles to a meaningful family of real-world degradations.

### 3. Build the permutation explorer
Let users visibly compare fonts under conditions before the generation stack gets much larger.

### 4. Define the generation interface
Decide how "vibes" and constraints map to candidate creation.

### 5. Build the mutation + adversarial loop
Make failures feed directly into the next candidate wave.

### 6. Finish the export story
Make the best result downloadable and practically usable.

## Concise statement of intent

TypeWeaver exists to help people create and choose fonts that survive reality.

That means:
- previewing fonts under hostile conditions
- scoring them meaningfully
- generating better candidates from intent
- attacking and mutating those candidates based on feedback
- iterating until the system finds something stronger
- letting the user take that result and use it in the world
