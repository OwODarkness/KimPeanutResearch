# Paper Importer Design

## Scope

The library will eventually accept three import styles:

1. a local PDF;
2. a DOI; and
3. a paper title.

This document specifies only the **local-PDF import** path. DOI and title
imports are deferred until the local library, provenance model, and parser
adapter are dependable. They are not aliases for a local import: both need
network lookup, provider provenance, caching, and an explicit privacy and
approval policy.

```text
TODO: DOI import → resolve/enrich with a provider adapter, then optionally
      acquire a user-approved PDF.
TODO: title import → search a provider, require the user to choose a result,
      then optionally acquire a user-approved PDF.
```

The canonical application-data layout has one database, `data/library.db`.
The duplicated `library.db` line in the initial sketch is intentionally
collapsed here. Here `data/` means the backend-configured application-data
root, not a path hard-coded relative to the project or the Tauri configuration.

```text
data/
├── library.db
└── papers/
    ├── 8f5d2c.../
    │   ├── paper.pdf
    │   ├── metadata.json
    │   ├── parsed.json
    │   ├── figures/
    │   │   ├── figure_001.png
    │   │   └── figure_002.png
    │   └── cache/
    │       └── ...
    └── a19ce1.../
        ├── paper.pdf
        ├── metadata.json
        └── parsed.json
```

`<paper-id>` is an application-generated opaque ID (for example, a UUID
without separators), not a title, DOI, or path-derived identifier. A source
filename can change, and a DOI can be absent or later corrected; neither is a
safe storage key.

## Ownership and durable records

The Vue UI asks a Tauri command to import a selected path. The Rust library
service owns validation, copying, parsing, IDs, database changes, and recovery.
No UI code reads or writes `data/` directly.

`BackendConfig` is the Rust-only runtime configuration boundary for this root.
It contains `StorageConfig.data_dir` and `layout_version` (currently `1`) and
derives the database, `papers/`, staging, and per-paper paths. It is separate
from `tauri.conf.json`, which remains packaging/window configuration. A chosen
custom location is persisted in `backend-settings.json` under the platform
local-app-data directory, outside the movable library itself.

The current migration command accepts an empty, absolute target directory,
copies every managed library file there, persists the new configuration, and
then routes future imports to it. The previous location is retained as a
recovery copy; automatic deletion is deferred until the library has durable
import/database recovery states.

```text
Vue file-picker
  → Tauri import_local_pdf command
  → Rust import service
      → managed files under data/papers/<paper-id>/
      → parser adapter + PaperExtractionTool
      → SQLite repositories / FTS5 index
  → import result for the UI
```

The durable representations serve different purposes:

| Location | Responsibility |
| --- | --- |
| `paper.pdf` | Immutable managed copy of the user-selected source PDF. |
| `metadata.json` | Import receipt and source/extracted metadata, including provenance and user corrections when introduced. |
| `parsed.json` | Versioned parser output and deterministic extraction results used to rebuild derived database/index records. |
| `figures/` | Derived page figures/crops, each linked to its source page and producer version. It may be regenerated. |
| `cache/` | Disposable parser/render/download intermediates. Never the only copy of information needed to restore the library. |
| `library.db` | Queryable normalized library state, relationships, import status, audit records, and FTS5 index. |

The PDF remains the original local source. `metadata.json` and `parsed.json`
are derived artifacts, not replacements for it. SQLite must retain enough
provenance to point from a user-visible field or search result back to the
paper and page/location that produced it.

## Local PDF flow

```text
User selects a PDF
    ↓
Validate file and create an import receipt
    ↓
Hash source and check for an existing managed copy
    ↓
Create data/papers/.staging/<import-id>/ and copy source as paper.pdf
    ↓
Create Paper + PaperFile records in SQLite as `importing`
    ↓
Parse the managed copy into page-aware text/layout representation
    ↓
Run PaperExtractionTool for Metadata, Abstract, Contributions
    ↓
Write metadata.json and parsed.json; optionally extract figures
    ↓
Populate normalized records and FTS5 from parsed output
    ↓
Atomically promote staging directory to papers/<paper-id>/
    ↓
Mark import `ready`, or `needs_review` with non-fatal warnings
```

The import must always parse the managed copy, never the original selected path:
the user can move, replace, or delete that original at any time after choosing
it. File copying is a normal import action, not a destructive move.

### Current implemented slice

The current desktop flow is intentionally limited to managed source storage.
The library pane's **Import papers** action opens a native PDF-only picker and
calls the typed `import_local_pdf` Tauri command. That command adapts to the
Rust `LocalPdfImporter`, which receives injected `StorageConfig`, accepts one
regular `.pdf` file, creates `data/papers/<opaque-id>/`, and copies the source
to `paper.pdf` while returning its byte count and SHA-256. It does not expose
the original absolute path to the frontend.

No SQLite record, parser output, metadata file, staging/recovery workflow, or
visible paper-list update exists in this slice yet. Those concerns deliberately
remain outside `LocalPdfImporter`; the next importer stages can compose this
managed-copy receipt instead of duplicating file handling.

### Detailed phases

1. **Receive and validate.** Accept a user-selected local path through a
   Tauri command. Verify it is a regular, readable file, enforce a configurable
   size limit, and check PDF signature/structure before treating the extension
   as meaningful. Record the original display filename and import time, but do
   not expose the original absolute path to other tools by default.

2. **Deduplicate.** Calculate SHA-256 while copying or in a preceding streaming
   pass. `PaperFile.content_sha256` is unique for managed primary PDFs. On an
   exact match, return the existing paper and offer organization/metadata edits
   rather than storing a second byte-identical file. Different editions,
   preprints, or publisher PDFs remain distinct when their content differs;
   later work can provide a user-reviewed merge relationship without changing
   this rule.

3. **Stage safely.** Use a sibling staging directory on the same volume as the
   final `papers/` directory. Write each artifact to a temporary filename,
   flush it, and rename it only when complete. Do not make it visible as a
   ready paper until all required source artifacts exist. Startup recovery can
   remove an abandoned staging directory or resume/fail its corresponding
   `importing` receipt.

4. **Parse with a layout-aware adapter.** Produce ordered `PageText` with
   one-based page numbers, parser ID/version, and retained block/layout ranges.
   Multi-column PDFs must not be supplied as indiscriminately flattened text:
   interleaved columns, captions, and body text can yield incorrect fields.
   Password-protected, malformed, image-only, and partially readable PDFs are
   valid outcomes to report, not reasons to invent text. OCR is a separate,
   explicitly identified adapter when enabled.

5. **Extract deterministic initial fields.** Call `PaperExtractionTool` once
   on that parsed document with `Metadata`, `Abstract`, and
   `ContributionStatements`. Store every returned value exactly as extracted,
   including confidence, source location, parser ID/version, and extractor
   ID/version. Missing fields and warnings remain visible as such and result
   in `needs_review` when the paper itself was imported successfully.

6. **Persist source and derived artifacts.** `metadata.json` is the import
   receipt plus selected source metadata. `parsed.json` contains the
   versioned parse/extraction result, preserving the page and byte locations
   needed to recreate searchable `Section` and extraction records. Figure
   generation may be deferred or fail independently; it must not prevent a
   text-importable paper from becoming ready.

7. **Commit library state.** After the final directory is promoted, write or
   finalize the normalized `Paper`, `PaperFile`, extracted `Section`, and
   initial field records in a SQLite transaction, then build/update FTS5 from
   parser text. An import/audit record identifies the user-triggered command,
   source hash, timestamps, and outcome. A failure leaves a durable failed
   receipt with an actionable error and no incomplete paper presented as ready.

The exact ordering of promotion and SQLite finalization must be implemented as
an idempotent state machine. There is no cross-filesystem/SQLite transaction,
so each startup reconciles `importing` records with final and staging
directories. A final directory with a matching import ID can be completed;
anything ambiguous stays non-ready for recovery rather than being guessed.

## Metadata and DOI boundary

`PaperExtractionTool` is not a PDF parser or DOI client. It accepts
`ParsedPaperDocument` from an adapter and does not access the filesystem,
network, SQLite, XMP metadata, or PDF link annotations.

For the initial importer it may extract:

- a tentative title from a first-page text line;
- one DOI-shaped string visible in the parsed page text;
- source abstract text; and
- explicit contribution statements.

Those values are source extractions, not authoritative bibliographic facts.
The tool cannot validate a DOI, resolve a DOI, retrieve authors/venue/year,
discover DOI candidates that exist only in XMP or embedded links, or choose
among conflicting DOI candidates. The parser adapter should preserve all such
future candidates (visible text, XMP, and links) with location/acquisition
method, but the initial local importer performs **no network DOI enrichment**.

`metadata.json` should keep the original extraction separate from a later
accepted DOI, provider-enriched value, or user correction. No later source may
silently overwrite the PDF-derived value. Each change records its producer,
time, evidence/reference, and status.

## Minimum states and user-visible results

| State | Meaning | UI action |
| --- | --- | --- |
| `importing` | A staged import is being copied, parsed, or recovered. | Show progress; do not expose as a complete paper. |
| `ready` | Managed PDF and required durable records were stored; optional fields may be absent. | Open paper, search it, organize it. |
| `needs_review` | PDF import succeeded but extraction has warnings, conflicting metadata, or incomplete text. | Show warnings and allow metadata correction/reparse. |
| `failed` | The PDF could not be safely copied or parsed enough to create the promised record. | Preserve error receipt; allow retry or removal. |

An unsupported PDF, unreadable path, duplicate binary, cancelled import, and
parser/OCR failure have distinct machine-readable error codes. Paper text must
be treated strictly as untrusted content throughout this process.

## Initial implementation slice

The first implementation should be deliberately narrow:

- streaming managed-file copy with SHA-256 calculation (`storage::file_utils`);
- local PDF picker and typed `import_local_pdf` command;
- managed copy, opaque paper ID, SHA-256 duplicate detection, and import
  receipt/state recovery;
- a layout-aware text parser adapter;
- `PaperExtractionTool` invocation for its three supported field types;
- `paper.pdf`, `metadata.json`, and `parsed.json` persistence;
- SQLite `Paper`, `PaperFile`, extraction/provenance records, and FTS5 text;
- clear warnings and manual metadata correction.

Figure extraction, OCR, DOI/title imports, provider lookup, citation graph
construction, and model-based analysis remain later work. Their artifacts may
be added without changing the ownership rule above: Rust services own side
effects, source data stays distinct from interpretation, and every derived
field remains traceable to its source.

## Verification criteria

Before the local importer is considered usable, test at least:

- successful text PDF import creates the stated directory and searchable
  library records;
- the managed PDF remains usable after the original path is moved or deleted;
- a byte-identical re-import resolves to the existing paper;
- a multi-column PDF retains reading order suitable for extraction;
- missing abstract/contribution text creates warnings without generated
  substitutes;
- invalid, password-protected, image-only, and interrupted imports produce
  recoverable non-ready states;
- parser/extractor versions and page/byte source locations survive a restart;
- startup recovery reconciles interrupted staging and SQLite state;
- no DOI network request occurs during local import.
