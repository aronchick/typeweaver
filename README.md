# TypeWeaver

TypeWeaver is a typography robustness lab and constrained font-creation platform.

This repository contains the **Phase 1 MVP** only:
- local font ingestion
- license normalization and policy status handling
- fixed Latin corpus rendering
- benchmark scoring for two profiles
- JSON report card output

Out of scope in this phase:
- font generation
- OCR scoring
- URL auditing
- website UI
- cloud/human-study workflows

## Workspace layout
- `crates/typeweaver-core`: shared types, corpus, profile and report models
- `crates/typeweaver-registry`: ingestion, license normalization, registry persistence
- `crates/typeweaver-render`: fixed Latin corpus rendering helpers
- `crates/typeweaver-bench`: profile benchmark runner and report builder
- `crates/typeweaver-cli`: CLI entrypoint (`ingest`, `bench`)

## Quickstart

Run all tests:

```bash
cargo test
```

Ingest local fonts and write `registry.json`:

```bash
cargo run -- ingest fixtures/fonts --registry-root fixtures/sample-output
```

Run a benchmark profile for one font id:

```bash
cargo run -- bench font-de79ad5b74ad40f2 --profile web_light_default --registry-root fixtures/sample-output
```

This produces:
- `fixtures/sample-output/registry.json`
- `fixtures/sample-output/reports/font-de79ad5b74ad40f2-web_light_default.json`

## Phase 1 corpus
The fixed Latin corpus includes:
- uppercase `A-Z`
- lowercase `a-z`
- digits `0-9`
- punctuation line
- confusion pairs: `O/0`, `I/l/1`, `S/5`, `B/8`, `rn/m`, `cl/d`

## Benchmark profiles
Implemented profiles (exactly):
- `web_light_default`
- `mobile_dark_low_contrast`

## Fixture notes
`fixtures/fonts` contains sample local files with sidecar `.license` files to demonstrate:
- approved assets: MIT, CC0
- rejected assets: OFL, mixed license
- quarantined asset: unknown/no license
