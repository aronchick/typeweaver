# TypeWeaver project instructions

## Mission
TypeWeaver is a typography robustness lab and constrained font-creation platform.

Phase 1 only:
- ingest approved fonts
- validate license metadata
- render a fixed Latin corpus
- apply a small benchmark matrix
- emit JSON report cards

Do not implement:
- font generation
- OCR scoring
- URL auditing
- website UI
- human study workflows
- cloud infrastructure

## Technical constraints
- Language: Rust
- Prefer MIT or Apache-2.0 dependencies
- Keep modules small, explicit, and deterministic
- Prefer simple CLI entrypoints before APIs
- Avoid speculative abstractions
- Write tests for parsing, profile loading, and report output

## License policy
Approved licenses for ingestion in Phase 1:
- Public Domain
- CC0
- MIT
- Apache-2.0

Reject or quarantine:
- OFL
- GPL variants
- unknown licenses
- ambiguous provenance
- mixed-license packs

## Expected workspace shape
Create a Rust workspace with these crates:
- crates/typeweaver-core
- crates/typeweaver-registry
- crates/typeweaver-render
- crates/typeweaver-bench
- crates/typeweaver-cli

## Phase 1 benchmark profiles
Implement exactly these initial profiles:
- web_light_default
- mobile_dark_low_contrast

## Corpus scope
Latin only.
Include:
- uppercase A-Z
- lowercase a-z
- digits 0-9
- punctuation
- confusion pairs: O/0, I/l/1, S/5, B/8, rn/m, cl/d

## Deliverables
- Rust workspace scaffolding
- registry schema and license normalization
- local font ingestion command
- renderer for fixed corpus
- benchmark runner for the two profiles
- JSON report card output
- tests
- README usage examples

## Acceptance criteria
- cargo test passes
- cargo run -- ingest <dir> indexes fonts and records approval status
- cargo run -- bench <font-id> --profile web_light_default emits a JSON report
- sample fixtures and sample output are included
- README documents setup and usage

## Working style
- Start with the workspace and shared types
- Then implement registry
- Then ingestion CLI
- Then renderer
- Then benchmark logic
- Then report card JSON
- Update README as features land
- Run tests after each milestone
