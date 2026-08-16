# Architecture

## System shape

```text
┌────────────────────────────────────────────┐
│ Vue 3 + TypeScript UI                       │
└───────────────────┬────────────────────────┘
                    │ Tauri IPC
┌───────────────────▼────────────────────────┐
│ Rust application core                       │
│  Library · Paper service · Search            │
│  Analysis · Provenance · Tools · Policy      │
└───────────┬──────────────────────┬─────────┘
            │                      │
┌───────────▼──────────┐  ┌────────▼─────────┐
│ SQLite + FTS5        │  │ Local files       │
│ research records     │  │ PDFs / text/cache │
└──────────────────────┘  └──────────────────┘
            │
┌───────────▼────────────────────────────────┐
│ Optional adapters: OpenAlex, Crossref,      │
│ Semantic Scholar, arXiv, Unpaywall, LLMs    │
└────────────────────────────────────────────┘
```

The UI presents state and invokes application commands. The Rust core owns all
domain decisions and side effects. SQLite and local research files are the
durable state. External services are replaceable adapters.

## Core domain

The initial library should support these first-class records:

- `Paper`, `Author`, `Venue`, `Tag`, `Collection`, and `Citation`
- `PaperFile` and extracted `Section`
- `Method`, `Dataset`, `Experiment`, `Metric`, and `Result`
- `Claim`, `Evidence`, `Interpretation`, and `Hypothesis`
- `Repository`, `ResearchTask`, `ResearchRun`, and `AuditEvent` (later phases)

Relationships should initially be normal SQLite foreign keys and join tables.
For example, a paper can cite papers, propose methods, use datasets, report
experiments, and support claims. A separate graph database is not needed to
express or query these relationships early on.

## Ingestion and analysis pipeline

```text
PDF / DOI / arXiv / URL
       ↓
Import and identify
       ↓
Metadata retrieval and source storage
       ↓
Parse into text and sections
       ↓
Index with SQLite FTS5
       ↓
Structured analysis
       ↓
Evidence-backed library records
```

Every derived record should retain its source paper and the best available
location reference. Generated output also records the parser or model version
that produced it.

### Deterministic paper-field extraction

The paper extraction tool is a pure Rust capability that accepts parsed,
page-oriented text from an interchangeable PDF/OCR adapter. The adapter must
preserve reading blocks or layout regions for multi-column documents; flattened
page text can interleave independent columns and is therefore not reliable
input. A request selects the desired fields (`Metadata`, `Abstract`, or explicit
`ContributionStatements`) in one pass over that representation; it does not
read files, invoke a model, or persist records. Each returned value includes
page and byte-range provenance plus parser and extractor identifiers. Import
workflows can request a standard field set, while a user action can request one
field only. Explicit contribution text is extracted as source statements, never
silently rewritten into a generated summary.

DOI discovery is part of local parsing: parser adapters retain DOI candidates
from PDF/XMP metadata, visible front matter, and embedded DOI links. Optional
Crossref, OpenAlex, or publisher lookup is a separate network-capable metadata
enrichment service, with provider provenance and no silent overwrites of local
or user-corrected facts.

## Agent boundary

The future agent is a client of the core, not a privileged owner of it:

```text
Agent runtime → tool registry → policy check → Rust service → persistent state
                                      ↓
                                  audit event
```

This prevents prompt injection in a paper or webpage from becoming a direct
instruction to the application. Tools stay narrow: `search_papers`,
`import_paper`, `extract_method`, or `create_claim` are preferable to arbitrary
shell, database, or filesystem access.

## Persistence rule

Research data, model context, and generated interpretation are different kinds
of state. The database should preserve sources and evidence; an LLM can reason
over that state but does not replace it.
