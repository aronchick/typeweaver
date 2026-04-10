# TypeWeaver

TypeWeaver is a typography robustness lab, font design workbench, and adversarial font optimization system.

The goal is not just to preview fonts in ideal conditions. The goal is to help people:
- see how fonts behave under real-world degradation
- compare permutations under blur, contrast loss, and other hostile viewing conditions
- generate or "vibe together" new fonts from desired qualities
- score those fonts for robustness
- tune them toward the strongest tradeoff between legibility, robustness, and weight
- ultimately download and use the resulting fonts themselves

In plain English: TypeWeaver should help someone ask, *"Will this hold up in the real world, and if not, can we evolve it into something that does?"*

## Product vision

TypeWeaver is meant to do three connected jobs:

### 1. Evaluate fonts under real-world conditions
A user should be able to take an existing font and inspect it under a battery of conditions that approximate reality:
- low contrast
- blur
- compression / raster artifacts
- small sizes
- hostile rendering environments
- mutation scenarios and adversarial degradation

The system should not just show the font. It should score it.

### 2. Generate and tune fonts from intent
A user should be able to describe a direction, feeling, or vibe for a font, and TypeWeaver should help generate candidate fonts that match that intent.

Those candidates should then be scored using the same robustness pipeline rather than judged only aesthetically.

The point is not "pretty generation" in isolation. The point is generation that can survive contact with reality.

### 3. Close the loop with adversarial optimization
TypeWeaver should eventually be able to:
- attack a font with increasingly relevant degradation or mutation scenarios
- observe where it fails
- tweak the font or generate new variants
- rescore the results
- iterate until it finds the best balance of:
  - robustness
  - visual intent / vibe
  - file weight / practical deployability

This is the real long-term loop: generate, attack, score, mutate, repeat.

## What exists today

The repository already contains a substantial base for that vision:
- deterministic Rust workspace
- registry and license normalization
- local ingest pipeline
- fixed corpus rendering
- benchmark scoring
- JSON report output
- API crate
- embedded web UI
- OCR crate
- OpenTelemetry + Prometheus observability
- deploy artifacts for running as a service

So this is no longer just a sketch. It is an implemented early platform.

## Current repo structure

- `crates/typeweaver-core`: shared types, corpus, profile and report models
- `crates/typeweaver-registry`: ingestion, license normalization, registry persistence
- `crates/typeweaver-render`: fixed corpus rendering helpers
- `crates/typeweaver-bench`: profile benchmark runner and report builder
- `crates/typeweaver-cli`: CLI entrypoint and `serve` command
- `crates/typeweaver-api`: Axum API + service layer
- `frontend/`: static site served by Caddy; talks to the API over same-origin `/api/*`
- `crates/typeweaver-ocr`: OCR scoring / related analysis work

## Canonical plan

The repo roadmap now lives in:

- `docs/canonical-plan.md`

That document is the source of truth for:
- original MVP intent
- hardening pass goals
- service / UI / deployment evolution
- the broader product direction toward generation + adversarial robustness loops

## Product model

TypeWeaver should be understood as a pipeline with four layers:

### A. Font intake
- ingest existing fonts
- normalize metadata and licensing
- track provenance and variants

### B. Robustness evaluation
- render deterministic corpora
- run degradation and mutation scenarios
- score outcomes across profiles and contexts

### C. Generative design
- let users specify desired vibes, constraints, or target environments
- generate candidate fonts or parameterized mutations
- compare candidates using the same benchmark system

### D. Optimization loop
- use adversarial attacks to find weak points
- mutate or regenerate the font
- rerun scoring
- search for the best robustness / aesthetics / size tradeoff

## What the scoring system should eventually care about

The score should not be a single vague number. It should become a structured view of performance under conditions like:
- readability at multiple sizes
- resilience under blur
- resilience under low contrast
- confusion-pair failure rates (`O/0`, `I/l/1`, `rn/m`, etc.)
- OCR or recognizer stability where relevant
- glyph distinctiveness under mutation
- file size / font weight cost
- fit to target vibe / design intent

## Why this matters

Most font workflows optimize for how something looks in ideal mocks.

TypeWeaver is supposed to optimize for how something survives reality.

A font on a marketing page, dashboard, embedded device, bad mobile screen, or noisy low-contrast environment has to withstand conditions that standard type exploration tools mostly ignore.

That is the wedge.

## Quickstart

Run commands from the workspace root (`/opt/typeweaver` or your local clone).

Run quality checks:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Ingest local fonts:

```bash
cargo run -- ingest fixtures/fonts
```

List ingested assets:

```bash
cargo run -- list
```

List available benchmark profiles:

```bash
cargo run -- profiles
```

Run one benchmark profile for a font id:

```bash
cargo run -- bench font-de79ad5b74ad40f2 --profile web_light_default
```

Run the local service:

```bash
cargo run -- serve
```

Web UI routes:

- `/` serves the marketing homepage.
- `/tool` serves the actual playground.
- The tool defaults to curated Google Fonts starters and syncs the selected starter into the existing registry before generating a JSON report.
- Upload is still available inside `/tool`, with an optional declared-license sidecar generated from the UI.

## Output layout

By default, commands write under `.typeweaver`:

- `.typeweaver/registry/registry.json`
- `.typeweaver/reports/<font-id>/<profile>/report.json`
- `.typeweaver/reports/<font-id>/<profile>/preview.txt`

You can change the root with `--registry-root <dir>` on `ingest`, `list`, and `bench`.

## Current benchmark profile examples

Implemented profile names include:
- `web_light_default`
- `mobile_dark_low_contrast`

These are early examples, not the final condition matrix.

## Immediate next frontier

The natural next step is to turn the current benchmark/reporting foundation into a richer mutation-and-optimization loop:
- more degradation profiles
- more interpretable robustness scores
- candidate generation workflows
- explicit font mutation primitives
- adversarial attack orchestration
- download-ready optimized outputs

If TypeWeaver succeeds, it should let someone move from:
- "I like this font"

to:
- "I know how this font fails"
- "I can see stronger variants"
- "I can generate better candidates"
- "I can ship the most robust one"
