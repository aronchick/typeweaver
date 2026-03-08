# Task: Implement TypeWeaver Phase 1 MVP

## Context
TypeWeaver is a typography robustness lab and constrained font-creation platform.
Phase 1 focuses on:
- ingesting approved fonts
- rendering a fixed Latin corpus
- applying a small benchmark matrix
- outputting report cards as JSON

## Scope
Build a Rust workspace with crates for:
- shared types
- asset registry
- renderer
- benchmark runner
- CLI

## Constraints
- Use Rust
- Prefer MIT or Apache-2.0 dependencies
- Do not implement generation
- Do not implement OCR
- Do not implement URL auditing
- Do not implement website UI
- Latin script only
- Approved licenses for ingestion: Public Domain, CC0, MIT, Apache-2.0
- Reject or quarantine OFL and unknown licenses for now

## Deliverables
1. Rust workspace and crate scaffolding
2. Registry schema and license normalization
3. CLI command to ingest local font files into registry
4. Renderer for uppercase/lowercase/digits/confusion corpus
5. Benchmark profiles:
   - web_light_default
   - mobile_dark_low_contrast
6. JSON report card schema and sample output
7. Unit tests for license normalization, profile parsing, and report serialization

## Acceptance criteria
- cargo test passes
- cargo run -- ingest <dir> indexes candidate fonts and marks approval status
- cargo run -- bench <font-id> --profile web_light_default emits a report card JSON file
- sample fixtures included
- README includes setup and usage

## Implementation notes
- Use ttf-parser for font metadata/parsing
- Use swash for shaping/raster if useful
- Keep output deterministic
- Prefer JSON files over databases in Phase 1 unless a tiny embedded store is clearly simpler
- Make the CLI and file formats easy for later web/API integration
