# TypeWeaver Canonical Plan

This document reconstructs the working plan that drove the initial TypeWeaver build and the later hardening / deployment pass.

It is derived from:
- the original Phase 1 task doc (`docs/codex-task-phase1.md`)
- the compacted OpenClaw session summary from 2026-03-08
- the hardening prompt used for the Codex pass on Hetzner
- the current repository state and commit history

## What TypeWeaver is

TypeWeaver is a typography robustness lab and constrained font-creation platform.

The near-term goal is not full font generation. The goal is to create a deterministic pipeline that can:
- ingest local font assets
- normalize and classify licenses
- render a fixed Latin evaluation corpus
- benchmark fonts against a small profile matrix
- emit stable JSON report cards
- surface the results through a local API and lightweight web UI

## Product framing

TypeWeaver should feel like a small, serious evaluation system rather than a toy demo.

That means:
- deterministic outputs
- predictable file layout
- explicit approval / quarantine / rejection policy
- stable machine-readable contracts for later UI/API work
- enough observability to operate it as a service

## Phase structure

## Phase 1 — Core MVP

### Goal
Prove the end-to-end evaluation loop locally.

### In scope
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

### Explicit constraints
- Rust implementation
- Prefer MIT / Apache-2.0 dependencies
- Latin script only
- deterministic output
- file-backed storage, not a heavyweight database
- no font generation yet
- no OCR in the original MVP scope
- no URL auditing
- no website UI in the original MVP scope

### Policy rules
Approved licenses for ingestion:
- Public Domain
- CC0
- MIT
- Apache-2.0

Rejected or quarantined for Phase 1:
- OFL
- mixed/ambiguous licensing
- unknown / missing licenses

### Required benchmark profiles
- `web_light_default`
- `mobile_dark_low_contrast`

### Required report contract
The report output must be stable enough for future API/UI consumers.

Minimum expectations:
- clean field names
- deterministic ordering / serialization
- predictable output path
- sample fixture report checked in

### Phase 1 acceptance criteria
- `cargo test` passes
- `cargo run -- ingest <dir>` indexes candidate fonts and marks approval status
- `cargo run -- bench <font-id> --profile web_light_default` emits a JSON report
- sample fixtures are included
- README explains setup and usage

## Phase 1.5 — Hardening pass

This was the first major Codex-driven improvement pass after the MVP existed.

### Goal
Harden correctness, determinism, and CLI usability without expanding product scope.

### Focus areas recovered from the hardening prompt

#### 1. CLI ergonomics
- make help text clear and accurate
- keep commands and flags consistent
- make output paths predictable
- improve errors for:
  - missing font IDs
  - bad profiles
  - missing directories

#### 2. Registry correctness
- tighten license normalization
- make approval / quarantine / rejection behavior explicit and tested
- keep registry JSON read/write stable and deterministic
- avoid duplicate ingestion using a stable identifier such as file hash

#### 3. Benchmark/profile correctness
- verify exact profile names
- keep profile parsing / selection explicit and tested
- make benchmark output deterministic

#### 4. Report card contract
- stabilize the JSON schema for later API/UI use
- keep field names clean and future-proof
- include at least one sample fixture report

#### 5. Test coverage
- add unit tests for normalization edge cases
- add end-to-end integration tests for ingest + bench
- keep `cargo test` green with zero warnings

#### 6. Repo hygiene
- ensure `target/` and `.typeweaver/` are ignored
- do not commit build artifacts

### Hardening acceptance criteria
- all prior Phase 1 behavior still works
- CLI output is clearer and more predictable
- report + registry output is deterministic
- duplicate ingestion is handled safely
- tests are materially stronger than the initial MVP

## Phase 2 — Service layer

This phase moved TypeWeaver from a local CLI-only tool to a locally hosted system.

### Goal
Expose the benchmark pipeline through a local service and make it usable from a browser.

### Scope recovered from memory + repo history
- add `typeweaver-api` crate
- add `serve` subcommand to the CLI
- provide REST endpoints for ingest / list / bench / status workflows
- embed a lightweight static web UI
- keep the file-backed workspace layout intact

### Operating principle
The API and UI sit on top of the same deterministic core rather than inventing a second execution path.

## Phase 3 — Deployability

### Goal
Make TypeWeaver easy to run on a box without hand-holding.

### Scope recovered from repo history
- Dockerfile
- systemd service file
- Caddy config
- deploy script
- GitHub Actions build + deploy workflow
- correct Hetzner path / port wiring

### Deployment assumptions from session history
- primary deployment target was Hetzner at `/opt/typeweaver`
- service intended to run persistently under systemd / tmux during bring-up
- Caddy fronts the service

## Phase 4 — Observability and production readiness

This is the phase memory referred to as already completed on 2026-03-08.

### Goal
Make the service inspectable and operable in production.

### Scope recovered from memory + commits
- OpenTelemetry tracing
- Prometheus metrics
- bearer-token auth
- richer health surface
- split liveness from deeper health
- add:
  - `/okz` for shallow liveness
  - `/healthz` for deeper health
  - `/varz` / metrics-style visibility where appropriate

### Production expectations
- service should expose health and metrics endpoints suitable for deployment checks
- telemetry should help explain request behavior and benchmark activity
- auth should be good enough for a non-publicly writable service

## Canonical architecture

The architecture implied by the original plan and the current repo is:

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
  - deterministic fixed-corpus rendering
- `typeweaver-bench`
  - benchmark execution
  - report assembly
- `typeweaver-cli`
  - ingest / list / profiles / bench / serve commands
- `typeweaver-api`
  - Axum service layer over the same core pipeline
- `typeweaver-ocr`
  - added later for OCR scoring experiments / extension

## Non-goals that should remain explicit

Unless deliberately reopened, these are still outside the original plan:
- generalized font generation
- broad web product scope beyond a thin operator UI
- cloud-first multi-tenant architecture
- non-Latin corpus expansion
- internet URL auditing / scraping workflows
- human-study workflows

## Current status against the reconstructed plan

### Completed
- Phase 1 MVP exists
- Hardening pass landed
- Service layer exists
- Web UI exists
- Deploy artifacts exist
- Observability / metrics / health endpoints exist

### Known shipped milestones visible in git history
- OCR crate added
- OTel + Prometheus observability added
- API crate added
- embedded vanilla JS web UI added
- CLI `serve` command added
- Docker + systemd + Caddy + deploy workflow added
- richer auth + health endpoints added

### Practical status summary
TypeWeaver is no longer just a plan. It is an implemented local service with CLI, API, UI, deploy assets, and observability. The remaining work is more about product direction, polish, and whatever comes after the MVP/hardening/service/deploy sequence.

## Suggested next-step roadmap

If work resumes, the most sensible order is:

1. **Stabilize contracts**
   - freeze report schema
   - freeze registry schema
   - document API surface clearly

2. **Decide what OCR means in-product**
   - keep as diagnostic scoring only, or
   - turn it into a first-class benchmark dimension

3. **Tighten operations**
   - verify auth model
   - verify deployment health checks
   - add smoke tests against the live service

4. **Choose the next product boundary**
   - stay an operator/internal tool, or
   - become a hosted/public-facing evaluation product

## Original Phase 1 task, preserved in distilled form

The original build brief was effectively:
- build a Rust workspace for TypeWeaver
- ingest approved local fonts
- normalize license metadata
- render a fixed Latin corpus
- benchmark two profiles
- emit JSON report cards
- keep outputs deterministic
- add tests and sample fixtures
- avoid expanding into generation, OCR, URL auditing, or a full web product too early

That remains the best concise statement of the project’s intent.
