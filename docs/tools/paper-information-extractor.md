# Paper Information Extractor

`PaperExtractionTool` extracts source-backed paper fields without an LLM. It
is implemented in `src-tauri/src/tool/paper_extraction.rs` and is a pure,
stateless Rust component.

## Boundary

```text
PDF/OCR parser adapter → ParsedPaperDocument → PaperExtractionTool → caller
```

The tool does not read files, parse PDFs, call a model, contact a network
service, or write to SQLite. The importer, a user-triggered command, or a
future controlled agent supplies `ParsedPaperDocument` and chooses whether to
persist the result.

External paper content is untrusted data. Text extracted from a paper is never
treated as an instruction.

## Request

`ExtractionRequest` selects fields through a `BTreeSet<ExtractionField>`:

```rust
let request = ExtractionRequest {
    fields: [
        ExtractionField::Metadata,
        ExtractionField::Abstract,
        ExtractionField::ContributionStatements,
    ].into_iter().collect(),
    max_pages: None,
};
```

Supported fields are:

- `Metadata`: title and DOI when they appear in the parsed text.
- `Abstract`: source abstract text.
- `ContributionStatements`: explicit source contribution statements, not a
  generated summary.

Callers may request one field only. The parser should run once and provide an
in-memory page representation shared by all requested extractors.

## Parser contract

`ParsedPaperDocument` contains ordered `PageText` values and the parser ID and
version. Each page has its original one-based page number.

For multi-column PDFs, the parser adapter must preserve reading blocks or
layout regions. Flattened page text can interleave columns, figure captions,
and body text; passing it directly can produce a wrong field even when the
extraction rule is correct. Poppler's `-bbox-layout` output is one suitable
source for an adapter because it retains block, line, and word geometry.

The production adapter should retain the page, block, and byte range used to
construct each `PageText`. PDF/OCR implementation is intentionally outside
this tool.

## DOI discovery and optional enrichment

DOI discovery belongs in the local parsing path, not in a network call inside
`PaperExtractionTool`. This keeps normal extraction fast, offline, repeatable,
and private.

The production parser adapter should provide the complete front matter, not
only the blocks selected for abstract extraction. It should collect DOI
candidates from, in order:

1. PDF document metadata and XMP metadata;
2. visible text in title, copyright, citation, and reference-format blocks;
3. embedded `https://doi.org/...` links and annotations.

Normalize a candidate by removing a `doi:` or `doi.org/` prefix and trailing
display punctuation. Preserve every candidate with its source location and
acquisition method. A later revision of `ExtractedMetadata` should model these
as `DoiCandidate` records rather than retaining only the first matching string;
the library service can then select a verified DOI without losing evidence.

After a DOI is accepted, an **optional metadata-enrichment service** may query
Crossref, OpenAlex, or a publisher adapter. That service is separate from this
tool because it needs network capability, provider-specific error handling,
rate limits, caching, and a user-visible privacy/approval policy. Record the
provider, request URL, retrieval time, response identifier, and the source DOI
used for every enriched field. Never silently overwrite PDF-extracted or
user-corrected metadata with provider data.

Future verification must cover a DOI in visible text, a `doi.org` link, XMP
metadata, malformed candidates, conflicting candidates, and a multi-column
paper such as the 3DGS test PDF. The complete parsed document—not an
abstract-only subset—must be passed to the metadata extractor.

## Deterministic extraction rules

The tool uses these ordered, non-generative rules:

1. **Abstract heading:** recognize `Abstract`, `Summary`, or a same-line form
   such as `Abstract—...`; stop at a recognized next section.
2. **ACM front matter:** when an abstract is unlabeled, take the preceding
   structural paragraph before `CCS Concepts:` or `Additional Key Words and
   Phrases:`. This requires layout-aware parser input so a figure caption is a
   separate block.
3. **Contribution heading:** take text below `Contributions` or `Our
   Contributions`, splitting ordinal list items when present.
4. **Enumerated contributions:** extract source statements marked `First,`,
   `Second,`, and so on after an explicit author lead-in about key elements,
   components, or contributions.

The tool joins only line-wrap hyphenation such as `ren-\nder`; it preserves
ordinary inline hyphens. It does not rewrite, condense, infer, or improve paper
text.

## Result and provenance

`ExtractedPaperInfo` has an optional result per requested field, plus warnings.
Every found value includes:

- `confidence`: rule confidence, not a statement of factual truth;
- `sources`: page number, UTF-8 parser byte range, and matched section rule;
- extractor ID and version;
- parser ID and version.

If no supported source pattern matches, the field remains `None` and the tool
returns `NoAbstractHeadingFound` or `NoContributionHeadingFound`. Callers must
show that state or offer user correction; they must not silently substitute a
model-generated value.

## Persistence and correction

The current tool returns extracted data only. A later library service may store
accepted values, but it must retain the original extraction, all provenance,
and any user correction as distinct records. A correction never overwrites the
source extraction without an audit trail.

## Verification

Unit tests cover heading extraction, section boundaries, unlabeled ACM front
matter, contribution lists, ordinal contribution statements, hyphenation, and
missing-field warnings. Run them with:

```powershell
Set-Location src-tauri
cargo test
```
