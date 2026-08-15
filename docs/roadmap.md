# Development Roadmap

## Phase 1 — Local Research Library

Deliver an independently useful desktop application.

- Tauri + Vue + Rust project skeleton
- SQLite schema, migrations, and application-data directory
- Paper import and managed local PDF storage
- Metadata editing and optional DOI/arXiv lookup
- Tags, collections, citations, paper detail views
- SQLite FTS5 search

**Exit criterion:** a researcher can build, organize, and search a personal
library offline after import.

## Phase 2 — Structured Paper Analysis

- Parse PDFs into sections and searchable extracted text
- Screen papers through problem, contribution, and limitation fields
- Extract methods, datasets, experiments, metrics, and results
- Create claims and evidence records with source locations
- Compare papers through structured fields rather than prose-only summaries

**Exit criterion:** each analysis result is inspectable and traceable to a
paper location or explicitly marked as an interpretation.

## Phase 3 — Research Intelligence

- Topic and related-paper discovery
- Citation and concept relationship views
- Research questions, notes, and task state
- Better cross-paper comparison and gap discovery

**Exit criterion:** the library can help a user navigate connections and gaps
across a body of literature.

## Phase 4 — Controlled Agent Runtime

- Tool registry and typed tool inputs/outputs
- Agent state and planning loop
- Capability policy, approvals, provenance validation, audit log
- Human review for consequential actions

**Exit criterion:** the agent can perform a research workflow using only
auditable, policy-controlled tools.

## Phase 5 — Literature to Code to Experiment

- Repository inspection and code search
- Controlled code changes, builds, and test execution
- Experiment definitions, execution, and result storage
- Analysis that links results back to hypotheses and paper evidence

**Exit criterion:** a user can trace an experimental conclusion from code and
results through the hypothesis and supporting research evidence.
