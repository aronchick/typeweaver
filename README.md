# TypeWeaver

TypeWeaver is a typography robustness lab and constrained font-creation platform.

Phase 1 focuses on:
- ingesting approved fonts
- validating provenance and license class
- rendering a fixed Latin corpus
- applying a small benchmark matrix
- generating JSON report cards

Phase 1 explicitly excludes:
- font generation
- OCR scoring
- URL auditing
- website UI
- human evaluation workflows

## Docs
See docs/ for RFCs and implementation guidance.

## Planned workspace
- crates/typeweaver-core
- crates/typeweaver-registry
- crates/typeweaver-render
- crates/typeweaver-bench
- crates/typeweaver-cli
