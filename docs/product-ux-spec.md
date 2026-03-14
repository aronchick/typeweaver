# TypeWeaver product + UX simplification spec

## Goal
Make TypeWeaver understandable in under 10 seconds, reduce onboarding friction, and separate the product story from the working tool.

Core framing:

> Pick a font. Break it on purpose. See if it survives.

## Primary problems with the current experience
- Home page and tool are merged into one long screen.
- The first thing users see is upload friction.
- The page explains too many mechanics before the problem is clear.
- Contrast is elegant in places but too soft for a product about legibility.
- The default path assumes people already have a font file ready.

## New information architecture
- `/` — Home
- `/tool.html` — Tool / playground
- `/scenarios.html` — Scenario explainer page
- `/examples.html` — Example comparisons and before/after artifacts

Only Home and Tool are required for the first implementation pass.

## Homepage structure
### 1. Hero
Headline:
**Test fonts where typography actually gets hard.**

Subhead:
Choose a font, run it through realistic degradation, and see what still reads when the conditions get ugly.

Primary CTA:
- Try the tool

Secondary CTA:
- See common failure modes

### 2. What teams do today
Keep this concrete and slightly naive:
- Pick a font from a mockup or brand deck
- Review a few clean strings
- Approve it
- Ship it

### 3. Why that breaks
Show real-world pressure:
- small text
- low contrast
- dark UI
- scan noise
- blur
- compression
- OCR capture
- dense layouts
- lookalike glyph collisions

### 4. What TypeWeaver does
- starts from real fonts
- applies realistic stress conditions
- compares candidates side by side
- shows where clarity collapses
- helps teams choose for resilience, not just taste

### 5. Why use it
Use cases:
- OCR and document capture
- operational dashboards
- forms and enterprise UI
- labels and packaging
- field environments / low-quality capture

### 6. Bottom CTA
**Stop choosing fonts only at their best.**

CTA:
- Open the tool

## Tool page structure
### Goal
Feel like a playground, not a workflow engine.

### First-run path
1. Pick a font source
   - Google Fonts
   - Upload your own font (advanced)
2. Pick a mode
   - Compare two fonts
   - Stress-test one font
3. Pick a scenario
   - Everyday web
   - Dark / low contrast mobile
   - OCR / scan pressure
   - Dense data UI
4. Run

### Default state
The page should not be empty.

Preload:
- one or two common starter fonts
- a default scenario
- a sample string that includes high-confusion pairs

Recommended starter set:
- Inter
- IBM Plex Sans
- Source Serif 4
- Roboto Mono
- Archivo

## Google Fonts onboarding
The tool should make Google Fonts the easiest path.

### Short-term implementation
- Add a curated starter-font selector in the tool UI.
- Make it visually primary.
- Keep custom upload in a lower section labeled advanced.
- If direct ingest from Google Fonts is not ready yet, expose the curated list as the intended first path and wire ingestion next.

### Preferred implementation
- user selects a starter font
- frontend fetches the corresponding Google Fonts stylesheet / binary assets
- tool ingests the resulting files into the existing registry
- selected starter font appears in the same local font library as uploaded fonts

## Accessibility + contrast requirements
This product should exceed the minimum vibe threshold for readability.

### Minimum targets
- WCAG AA normal text: 4.5:1
- Large text: 3:1
- Small labels and helper text should still aim for 4.5:1 in practice

### Visual rules
- darken body copy and helper text
- avoid low-contrast micro-labels
- increase card/background separation
- use sans for UI and body text
- reserve serif only for large display moments
- strengthen interactive states and borders

## Recommended palette direction
- text / ink: very dark neutral
- background: warm light neutral is fine
- muted text: still dark enough to read comfortably
- accent colors: keep warm coral + teal, but do not use them for low-contrast text

## Graphics to build
Use explanatory product graphics, not generic marketing art.

### Set 1: clean vs real-world
- pristine specimen
- degraded specimen
- same words, visibly different outcomes

### Set 2: failure ladder
- 5 stages of degradation for the same font
- show where a given font collapses

### Set 3: confusion board
- O / 0
- I / l / 1
- rn / m
- S / 5
- B / 8

### Set 4: use-case cards
- OCR
- mobile dark UI
- dense dashboard
- labels / packaging

## Implementation order
### Phase 1 — immediate
- split current page into Home + Tool
- rewrite Home as narrative page
- add nav between pages
- improve contrast tokens and text colors
- relabel upload as advanced / secondary

### Phase 2 — onboarding
- add curated Google Fonts starter selection
- add default sample text and scenario
- improve tool empty states

### Phase 3 — richer explanation
- add scenarios page
- add examples page
- add explanatory graphics

### Phase 4 — polish
- formal contrast audit
- focus state audit
- keyboard and mobile pass

## Success criteria
- A first-time visitor understands the problem without touching the tool.
- A first-time user can reach a meaningful comparison without having a local font file.
- The tool feels simpler, clearer, and more trustworthy.
- Readability of the site itself matches the product’s promise.
