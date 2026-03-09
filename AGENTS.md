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

<!-- BEGIN BEADS INTEGRATION -->
## Issue Tracking with bd (beads)

**IMPORTANT**: This project uses **bd (beads)** for ALL issue tracking. Do NOT use markdown TODOs, task lists, or other tracking methods.

### Why bd?

- Dependency-aware: Track blockers and relationships between issues
- Git-friendly: Dolt-powered version control with native sync
- Agent-optimized: JSON output, ready work detection, discovered-from links
- Prevents duplicate tracking systems and confusion

### Quick Start

**Check for ready work:**

```bash
bd ready --json
```

**Create new issues:**

```bash
bd create "Issue title" --description="Detailed context" -t bug|feature|task -p 0-4 --json
bd create "Issue title" --description="What this issue is about" -p 1 --deps discovered-from:bd-123 --json
```

**Claim and update:**

```bash
bd update <id> --claim --json
bd update bd-42 --priority 1 --json
```

**Complete work:**

```bash
bd close bd-42 --reason "Completed" --json
```

### Issue Types

- `bug` - Something broken
- `feature` - New functionality
- `task` - Work item (tests, docs, refactoring)
- `epic` - Large feature with subtasks
- `chore` - Maintenance (dependencies, tooling)

### Priorities

- `0` - Critical (security, data loss, broken builds)
- `1` - High (major features, important bugs)
- `2` - Medium (default, nice-to-have)
- `3` - Low (polish, optimization)
- `4` - Backlog (future ideas)

### Workflow for AI Agents

1. **Check ready work**: `bd ready` shows unblocked issues
2. **Claim your task atomically**: `bd update <id> --claim`
3. **Work on it**: Implement, test, document
4. **Discover new work?** Create linked issue:
   - `bd create "Found bug" --description="Details about what was found" -p 1 --deps discovered-from:<parent-id>`
5. **Complete**: `bd close <id> --reason "Done"`

### Auto-Sync

bd automatically syncs via Dolt:

- Each write auto-commits to Dolt history
- Use `bd dolt push`/`bd dolt pull` for remote sync
- No manual export/import needed!

### Important Rules

- ✅ Use bd for ALL task tracking
- ✅ Always use `--json` flag for programmatic use
- ✅ Link discovered work with `discovered-from` dependencies
- ✅ Check `bd ready` before asking "what should I work on?"
- ❌ Do NOT create markdown TODO lists
- ❌ Do NOT use external issue trackers
- ❌ Do NOT duplicate tracking systems

For more details, see README.md and docs/QUICKSTART.md.

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

<!-- END BEADS INTEGRATION -->
