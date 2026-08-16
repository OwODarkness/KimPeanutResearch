---
name: kimpeanut-frontend
description: Maintain the KimPeanut Research Vue frontend with its clean, modern, researcher-focused visual language and modular component architecture. Use when designing, reviewing, or updating frontend pages, Vue components, frontend styling, or UI documentation in this repository.
---

# KimPeanut Frontend

Use this skill for all Vue and frontend work in KimPeanut Research.

## Before changing the UI

1. Read `AGENTS.md` and the relevant project document.
2. Read `docs/front/README.md` and the page-specific frontend reference when one exists.
3. Inspect the existing feature components before introducing new structure.
4. Keep research data, model output, and agent interpretation visually and conceptually distinct.

## Design direction

- Make the interface clean, calm, modern, and optimized for sustained literature review.
- Prefer clear hierarchy, generous whitespace, restrained color, compact metadata, and readable typography.
- Design for scanning: paper title, authors, venue, tags, notes, provenance, and state should be easy to locate.
- Use a navigation/context/content hierarchy when a page benefits from it; do not add panels merely for decoration.
- Treat source and provenance as first-class UI information.
- Keep controls visually quiet until they represent an important action.

## Vue structure

- Keep `App.vue` as a composition root, not a large feature implementation.
- Put feature code under `src/features/<feature>/`.
- Split meaningful regions into focused components; pass display state with typed props.
- Keep temporary fixtures and domain-facing types out of templates.
- Keep Tauri IPC and persistence behind services/composables; do not access SQLite or research files from Vue.
- Do not add event behavior to a static mock unless the user requests it.

## Completion workflow

1. Implement the smallest coherent frontend slice.
2. Keep the relevant `docs/front/` reference synchronized while coding: update it whenever the page structure, visual language, component boundary, or interaction policy changes.
3. Before considering the task complete, verify that the implementation and its frontend reference describe the same current behavior.
4. Run `npm run build` and `git diff --check`.
5. Report which components, docs, and checks changed.

Detailed current conventions live in `docs/front/README.md` and its linked references. Read only the relevant page reference for the task to keep context efficient.



<!-- No bundled resources; project docs are referenced directly above. -->

### scripts/
Executable code (Python/Bash/etc.) that can be run directly to perform specific operations.

**Examples from other skills:**
- PDF skill: `fill_fillable_fields.py`, `extract_form_field_info.py` - utilities for PDF manipulation
- DOCX skill: `document.py`, `utilities.py` - Python modules for document processing

**Appropriate for:** Python scripts, shell scripts, or any executable code that performs automation, data processing, or specific operations.

**Note:** Scripts may be executed without loading into context, but can still be read by Codex for patching or environment adjustments.

### references/
Documentation and reference material intended to be loaded into context to inform Codex's process and thinking.

**Examples from other skills:**
- Product management: `communication.md`, `context_building.md` - detailed workflow guides
- BigQuery: API reference documentation and query examples
- Finance: Schema documentation, company policies

**Appropriate for:** In-depth documentation, API references, database schemas, comprehensive guides, or any detailed information that Codex should reference while working.

### assets/
Files not intended to be loaded into context, but rather used within the output Codex produces.

**Examples from other skills:**
- Brand styling: PowerPoint template files (.pptx), logo files
- Frontend builder: HTML/React boilerplate project directories
- Typography: Font files (.ttf, .woff2)

**Appropriate for:** Templates, boilerplate code, document templates, images, icons, fonts, or any files meant to be copied or used in the final output.

---

**Not every skill requires all three types of resources.**
