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
- `crates/typeweaver-cli`: CLI entrypoint (`ingest`, `list`, `profiles`, `bench`)

## Quickstart

Run commands from the workspace root (`/opt/typeweaver`).

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

Expected output shape:
```text
ingested=5 approved=2 rejected=2 quarantined=1 registry=.typeweaver/registry/registry.json
```

List ingested assets:

```bash
cargo run -- list
```

Expected first line:
```text
font_id    family    style    license    status
```

List available benchmark profiles:

```bash
cargo run -- profiles
```

Expected rows include:
```text
web_light_default
mobile_dark_low_contrast
```

Run one benchmark profile for a font id:

```bash
cargo run -- bench font-de79ad5b74ad40f2 --profile web_light_default
```

Expected output shape:
```text
report=.typeweaver/reports/font-de79ad5b74ad40f2/web_light_default/report.json preview=.typeweaver/reports/font-de79ad5b74ad40f2/web_light_default/preview.txt
```

### Output layout
By default, commands write under `.typeweaver`:

- `.typeweaver/registry/registry.json`
- `.typeweaver/reports/<font-id>/<profile>/report.json`
- `.typeweaver/reports/<font-id>/<profile>/preview.txt`

You can change the root with `--registry-root <dir>` on `ingest`, `list`, and `bench`.

Sample generated artifacts are included under `fixtures/sample-output/` and match the fixture benchmark example above (`font-de79ad5b74ad40f2` + `web_light_default`).

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
