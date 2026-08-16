//! Deterministic, source-backed extraction from a parsed paper document.
//!
//! This module deliberately accepts parser output rather than a PDF path or
//! bytes. A Poppler, PDFium, OCR, or test adapter can create a
//! [`ParsedPaperDocument`] without making field extraction depend on that
//! implementation. It also performs no persistence: the library/importer owns
//! the decision to store its source-backed results.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// A caller-selectable information field.
///
/// Add new variants as new deterministic extractors become available. This is
/// preferable to an ever-growing collection of boolean request flags.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExtractionField {
    Metadata,
    Abstract,
    ContributionStatements,
}

/// Controls which fields are extracted from one parsed document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractionRequest {
    pub fields: BTreeSet<ExtractionField>,
    /// Limits the pages examined after parsing. `None` uses every supplied page.
    pub max_pages: Option<u32>,
}

impl ExtractionRequest {
    pub fn for_field(field: ExtractionField) -> Self {
        Self {
            fields: [field].into_iter().collect(),
            max_pages: None,
        }
    }

    pub fn includes(&self, field: &ExtractionField) -> bool {
        self.fields.contains(field)
    }
}

/// Parser-neutral, page-oriented text supplied by a PDF or OCR adapter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedPaperDocument {
    pub pages: Vec<PageText>,
    /// Identifies the component that converted the source into page text.
    pub parser_id: String,
    pub parser_version: String,
}

/// Text from a single source page. Page numbers are one-based PDF page numbers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageText {
    pub page_number: u32,
    pub text: String,
}

/// A precise source range in parser output. Offsets are UTF-8 byte offsets.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLocation {
    pub page_number: u32,
    pub start_byte: usize,
    pub end_byte: usize,
    pub section_heading: Option<String>,
}

/// A deterministic field value together with its extraction evidence.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Extracted<T> {
    pub value: T,
    pub confidence: f32,
    pub sources: Vec<SourceLocation>,
    pub extractor_id: String,
    pub extractor_version: String,
    pub parser_id: String,
    pub parser_version: String,
}

/// Bibliographic values which can be extracted without interpreting the paper.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedMetadata {
    pub title: Option<String>,
    pub doi: Option<String>,
}

/// Result of one extraction pass. Fields not requested remain `None`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedPaperInfo {
    pub metadata: Option<Extracted<ExtractedMetadata>>,
    pub abstract_text: Option<Extracted<String>>,
    /// Explicit source sentences or list items only; never a generated summary.
    pub contribution_statements: Option<Extracted<Vec<String>>>,
    #[serde(default)]
    pub warnings: Vec<ExtractionWarning>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExtractionWarning {
    NoAbstractHeadingFound,
    NoContributionHeadingFound,
    PageLimitReached { max_pages: u32 },
}

/// Stateless deterministic extractor. It is safe to reuse across requests.
#[derive(Clone, Debug, Default)]
pub struct PaperExtractionTool;

impl PaperExtractionTool {
    pub const ID: &'static str = "deterministic-paper-field-extractor";
    pub const VERSION: &'static str = "0.2.0";

    /// Extracts all requested fields from the supplied parsed representation.
    ///
    /// The document is filtered once, then each selected field extractor shares
    /// that in-memory view. No file read, PDF parsing, model call, or mutation
    /// occurs here.
    pub fn extract(
        &self,
        document: &ParsedPaperDocument,
        request: &ExtractionRequest,
    ) -> ExtractedPaperInfo {
        let page_limit = request.max_pages.map(|value| value as usize);
        let pages: Vec<&PageText> = match page_limit {
            Some(limit) => document.pages.iter().take(limit).collect(),
            None => document.pages.iter().collect(),
        };
        let mut result = ExtractedPaperInfo::default();

        if page_limit.is_some_and(|limit| document.pages.len() > limit) {
            result.warnings.push(ExtractionWarning::PageLimitReached {
                max_pages: request.max_pages.expect("page limit exists") as u32,
            });
        }

        if request.includes(&ExtractionField::Metadata) {
            result.metadata = self.extract_metadata(document, &pages);
        }
        if request.includes(&ExtractionField::Abstract) {
            result.abstract_text = self.extract_abstract(document, &pages);
            if result.abstract_text.is_none() {
                result
                    .warnings
                    .push(ExtractionWarning::NoAbstractHeadingFound);
            }
        }
        if request.includes(&ExtractionField::ContributionStatements) {
            result.contribution_statements = self.extract_contributions(document, &pages);
            if result.contribution_statements.is_none() {
                result
                    .warnings
                    .push(ExtractionWarning::NoContributionHeadingFound);
            }
        }

        result
    }

    fn extract_metadata(
        &self,
        document: &ParsedPaperDocument,
        pages: &[&PageText],
    ) -> Option<Extracted<ExtractedMetadata>> {
        let first_page = pages.first()?;
        let title = first_page
            .text
            .lines()
            .map(str::trim)
            .find(|line| is_title_candidate(line))
            .map(ToOwned::to_owned);
        let doi = pages
            .iter()
            .flat_map(|page| page.text.split_whitespace())
            .find_map(normalize_doi);

        if title.is_none() && doi.is_none() {
            return None;
        }
        let end_byte = title.as_ref().map_or(0, String::len);
        Some(self.with_provenance(
            document,
            ExtractedMetadata { title, doi },
            0.55,
            SourceLocation {
                page_number: first_page.page_number,
                start_byte: 0,
                end_byte,
                section_heading: None,
            },
        ))
    }

    fn extract_abstract(
        &self,
        document: &ParsedPaperDocument,
        pages: &[&PageText],
    ) -> Option<Extracted<String>> {
        self.extract_heading_block(document, pages, is_abstract_heading, "Abstract", 0.9)
            .or_else(|| self.extract_front_matter_abstract(document, pages))
    }

    fn extract_contributions(
        &self,
        document: &ParsedPaperDocument,
        pages: &[&PageText],
    ) -> Option<Extracted<Vec<String>>> {
        let block = self.extract_heading_block(
            document,
            pages,
            is_contribution_heading,
            "Contributions",
            0.75,
        );
        if let Some(block) = block {
            let statements: Vec<String> = split_numbered_items(&block.value)
                .into_iter()
                .map(|line| clean_list_marker(line).trim_end())
                .filter(|line| !line.is_empty())
                .map(ToOwned::to_owned)
                .collect();
            return (!statements.is_empty()).then_some(Extracted {
                value: statements,
                confidence: block.confidence,
                sources: block.sources,
                extractor_id: block.extractor_id,
                extractor_version: block.extractor_version,
                parser_id: block.parser_id,
                parser_version: block.parser_version,
            });
        }
        self.extract_enumerated_contributions(document, pages)
    }

    /// Handles publisher front matter whose abstract paragraph has no heading.
    /// ACM's two-column template commonly puts it immediately before
    /// `CCS Concepts:`, which is a structural publisher field.
    fn extract_front_matter_abstract(
        &self,
        document: &ParsedPaperDocument,
        pages: &[&PageText],
    ) -> Option<Extracted<String>> {
        for page in pages {
            let lines: Vec<(usize, &str)> = line_offsets(&page.text).collect();
            let Some(anchor) = lines.iter().position(|(_, line)| {
                let normalized = line.trim().to_ascii_lowercase();
                normalized.starts_with("ccs concepts:")
                    || normalized.starts_with("additional key words and phrases:")
            }) else {
                continue;
            };
            let mut start = anchor;
            while start > 0 && lines[start - 1].1.trim().is_empty() {
                start -= 1;
            }
            while start > 0 && !lines[start - 1].1.trim().is_empty() {
                start -= 1;
            }
            let value = normalize_block(
                &lines[start..anchor]
                    .iter()
                    .map(|(_, line)| *line)
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
            // A short segment is more likely to be author/venue text or a
            // figure fragment than an abstract.
            if value.len() >= 80 {
                let start_byte = lines[start].0;
                let end_byte = lines[anchor - 1].0 + lines[anchor - 1].1.len();
                return Some(self.with_provenance(
                    document,
                    value,
                    0.72,
                    SourceLocation {
                        page_number: page.page_number,
                        start_byte,
                        end_byte,
                        section_heading: Some("Abstract (front-matter fallback)".into()),
                    },
                ));
            }
        }
        None
    }

    /// Extracts source text from an explicit ordinal list following an author
    /// statement about key elements, components, or contributions.
    fn extract_enumerated_contributions(
        &self,
        document: &ParsedPaperDocument,
        pages: &[&PageText],
    ) -> Option<Extracted<Vec<String>>> {
        const ORDINALS: [&str; 5] = ["first,", "second,", "third,", "fourth,", "fifth,"];

        for page in pages {
            let text = normalize_block(&page.text);
            let lower = text.to_ascii_lowercase();
            let Some(lead_in) = lower.find("we introduce") else {
                continue;
            };
            let lead_in_tail = &lower[lead_in..];
            if !["key elements", "key components", "key contributions"]
                .iter()
                .any(|phrase| lead_in_tail.contains(phrase))
            {
                continue;
            }
            let markers: Vec<usize> = ORDINALS
                .iter()
                .filter_map(|ordinal| lower[lead_in..].find(ordinal).map(|index| lead_in + index))
                .collect();
            if markers.len() < 2 {
                continue;
            }
            let last = *markers.last().expect("at least two ordinal markers");
            let Some(end) = lower[last..]
                .find("we demonstrate")
                .map(|index| last + index)
                .or_else(|| lower[last..].find('.').map(|index| last + index + 1))
            else {
                continue;
            };
            let statements = markers
                .iter()
                .enumerate()
                .map(|(index, start)| {
                    let next = markers.get(index + 1).copied().unwrap_or(end);
                    text[*start..next]
                        .trim()
                        .trim_end_matches(';')
                        .trim()
                        .to_string()
                })
                .filter(|statement| !statement.is_empty())
                .collect::<Vec<_>>();
            if statements.len() >= 2 {
                return Some(self.with_provenance(
                    document,
                    statements,
                    0.65,
                    SourceLocation {
                        page_number: page.page_number,
                        start_byte: markers[0],
                        end_byte: end,
                        section_heading: Some("Contributions (enumerated fallback)".into()),
                    },
                ));
            }
        }
        None
    }

    fn extract_heading_block(
        &self,
        document: &ParsedPaperDocument,
        pages: &[&PageText],
        heading_matcher: fn(&str) -> Option<&str>,
        heading_name: &str,
        confidence: f32,
    ) -> Option<Extracted<String>> {
        for (page_index, page) in pages.iter().enumerate() {
            let mut start = None;
            let mut collected = String::new();
            let mut end = 0;
            for (offset, raw_line) in line_offsets(&page.text) {
                let line = raw_line.trim();
                if let Some(after_heading) = heading_matcher(line) {
                    start = Some(offset);
                    let content = after_heading.trim();
                    if !content.is_empty() {
                        append_line(&mut collected, content);
                    }
                    end = offset + raw_line.len();
                    continue;
                }
                if start.is_some() && is_section_boundary(line) {
                    break;
                }
                if start.is_some() {
                    append_line(&mut collected, line);
                    end = offset + raw_line.len();
                }
            }
            if let Some(start_byte) = start {
                let value = normalize_block(&collected);
                if !value.is_empty() {
                    return Some(self.with_provenance(
                        document,
                        value,
                        confidence,
                        SourceLocation {
                            page_number: page.page_number,
                            start_byte,
                            end_byte: end,
                            section_heading: Some(heading_name.into()),
                        },
                    ));
                }
            }
            // Headings can start on one page and continue on the next. This
            // intentionally does not guess across pages without a heading.
            let _ = page_index;
        }
        None
    }

    fn with_provenance<T>(
        &self,
        document: &ParsedPaperDocument,
        value: T,
        confidence: f32,
        source: SourceLocation,
    ) -> Extracted<T> {
        Extracted {
            value,
            confidence,
            sources: vec![source],
            extractor_id: Self::ID.into(),
            extractor_version: Self::VERSION.into(),
            parser_id: document.parser_id.clone(),
            parser_version: document.parser_version.clone(),
        }
    }
}

fn line_offsets(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.split_inclusive('\n').scan(0, |offset, line| {
        let current = *offset;
        *offset += line.len();
        Some((current, line.trim_end_matches(['\r', '\n'])))
    })
}

fn append_line(target: &mut String, line: &str) {
    if !target.is_empty() {
        target.push('\n');
    }
    target.push_str(line);
}

fn normalize_block(block: &str) -> String {
    // PDF text extraction commonly retains a hyphen only because a word was
    // split at the visual line boundary (for example, `ren-\nder`). Preserve
    // ordinary in-line hyphens while joining that deterministic layout artifact.
    block
        .replace("-\r\n", "")
        .replace("-\n", "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalized_heading(line: &str) -> String {
    line.trim()
        .trim_end_matches([':', '.', '—', '-'])
        .to_ascii_lowercase()
}

fn is_abstract_heading(line: &str) -> Option<&str> {
    let lower = line.to_ascii_lowercase();
    for prefix in ["abstract", "summary"] {
        if lower == prefix {
            return Some("");
        }
        if let Some(remainder) = line.get(prefix.len()..) {
            if lower.starts_with(prefix)
                && matches!(remainder.chars().next(), Some(':') | Some('—') | Some('-'))
            {
                return Some(&remainder[remainder.chars().next().unwrap().len_utf8()..]);
            }
        }
    }
    None
}

fn is_contribution_heading(line: &str) -> Option<&str> {
    let heading = normalized_heading(line);
    (heading == "contributions" || heading == "our contributions").then_some("")
}

fn is_section_boundary(line: &str) -> bool {
    let heading = normalized_heading(line);
    if matches!(
        heading.as_str(),
        "keywords" | "index terms" | "introduction" | "references"
    ) || heading.starts_with("keywords:")
        || heading.starts_with("index terms:")
    {
        return true;
    }
    let numbered_title = heading
        .trim_start_matches(|character: char| character.is_ascii_digit() || character == '.')
        .trim_start();
    numbered_title != heading
        && matches!(
            numbered_title,
            "introduction"
                | "background"
                | "related work"
                | "method"
                | "methods"
                | "methodology"
                | "experiments"
                | "results"
                | "conclusion"
                | "conclusions"
                | "references"
        )
}

fn is_title_candidate(line: &&str) -> bool {
    !line.is_empty()
        && line.len() >= 8
        && line.len() <= 300
        && !line.starts_with("http")
        && !line.to_ascii_lowercase().starts_with("doi")
        && is_abstract_heading(line).is_none()
}

fn normalize_doi(token: &str) -> Option<String> {
    let candidate = token
        .trim_matches(|character: char| matches!(character, '.' | ',' | ';' | ')' | ']' | '}'))
        .strip_prefix("https://doi.org/")
        .or_else(|| token.strip_prefix("http://doi.org/"))
        .unwrap_or(token)
        .trim_start_matches("doi:");
    let lower = candidate.to_ascii_lowercase();
    (lower.starts_with("10.") && lower.contains('/')).then_some(candidate.to_string())
}

fn clean_list_marker(line: &str) -> &str {
    line.trim_start_matches(|character: char| {
        character.is_ascii_digit()
            || matches!(character, '.' | ')' | '-' | '*' | '•' | '(')
            || character.is_whitespace()
    })
}

/// Splits a flattened contribution block only at an ordinal list marker, such
/// as `2. `, so ordinary sentence punctuation remains untouched.
fn split_numbered_items(text: &str) -> Vec<&str> {
    let mut boundaries = vec![0];
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_digit() && (index == 0 || bytes[index - 1].is_ascii_whitespace()) {
            let digits_end = bytes[index..]
                .iter()
                .position(|byte| !byte.is_ascii_digit())
                .map(|offset| index + offset)
                .unwrap_or(bytes.len());
            if bytes.get(digits_end) == Some(&b'.')
                && bytes
                    .get(digits_end + 1)
                    .is_some_and(u8::is_ascii_whitespace)
                && index != 0
            {
                boundaries.push(index);
            }
            index = digits_end;
        }
        index += 1;
    }
    boundaries
        .iter()
        .enumerate()
        .map(|(position, start)| {
            let end = boundaries.get(position + 1).copied().unwrap_or(text.len());
            &text[*start..end]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(text: &str) -> ParsedPaperDocument {
        ParsedPaperDocument {
            pages: vec![PageText {
                page_number: 1,
                text: text.into(),
            }],
            parser_id: "test-parser".into(),
            parser_version: "1".into(),
        }
    }

    #[test]
    fn extracts_only_requested_abstract_with_provenance() {
        let input = document("A Useful Paper\nAbstract\nA repeatable method.\nKeywords: parsing\n");
        let result = PaperExtractionTool.extract(
            &input,
            &ExtractionRequest::for_field(ExtractionField::Abstract),
        );

        let abstract_text = result.abstract_text.expect("abstract");
        assert_eq!(abstract_text.value, "A repeatable method.");
        assert!(result.metadata.is_none());
        assert_eq!(abstract_text.sources[0].page_number, 1);
    }

    #[test]
    fn stops_abstract_at_numbered_section() {
        let input = document("Abstract—A useful method.\n1. Introduction\nNot abstract text.");
        let result = PaperExtractionTool.extract(
            &input,
            &ExtractionRequest::for_field(ExtractionField::Abstract),
        );

        assert_eq!(
            result.abstract_text.expect("abstract").value,
            "A useful method."
        );
    }

    #[test]
    fn extracts_explicit_contribution_items_without_summarizing() {
        let input =
            document("Our Contributions\n1. We release data.\n2. We evaluate it.\n2. Method\n");
        let result = PaperExtractionTool.extract(
            &input,
            &ExtractionRequest::for_field(ExtractionField::ContributionStatements),
        );

        assert_eq!(
            result.contribution_statements.expect("contributions").value,
            vec!["We release data.", "We evaluate it."]
        );
    }

    #[test]
    fn extracts_unlabeled_acm_front_matter_abstract() {
        let input = document(
            "Paper title\n\nThis is a long enough abstract paragraph with several source-backed details. \
             It remains a single paragraph and ends before the ACM classification field.\n\n\
             CCS Concepts: • Computing methodologies → Rendering;\n",
        );
        let result = PaperExtractionTool.extract(
            &input,
            &ExtractionRequest::for_field(ExtractionField::Abstract),
        );

        let abstract_text = result.abstract_text.expect("front-matter abstract");
        assert!(abstract_text
            .value
            .starts_with("This is a long enough abstract"));
        assert_eq!(
            abstract_text.sources[0].section_heading.as_deref(),
            Some("Abstract (front-matter fallback)")
        );
    }

    #[test]
    fn extracts_explicit_ordinal_contribution_statements() {
        let input = document(
            "We introduce three key elements for the system. First, we represent data. \
             Second, we optimize it. Third, we render it quickly. We demonstrate the result.",
        );
        let result = PaperExtractionTool.extract(
            &input,
            &ExtractionRequest::for_field(ExtractionField::ContributionStatements),
        );

        assert_eq!(
            result.contribution_statements.expect("contributions").value,
            vec![
                "First, we represent data.",
                "Second, we optimize it.",
                "Third, we render it quickly."
            ]
        );
    }

    #[test]
    fn joins_pdf_line_wrap_hyphenation_without_changing_inline_hyphens() {
        assert_eq!(
            normalize_block("real-\ntime and state-of-the-art"),
            "realtime and state-of-the-art"
        );
    }

    #[test]
    fn reports_missing_requested_field() {
        let input = document("A Useful Paper\nNo labeled fields here.");
        let result = PaperExtractionTool.extract(
            &input,
            &ExtractionRequest::for_field(ExtractionField::Abstract),
        );

        assert_eq!(
            result.warnings,
            vec![ExtractionWarning::NoAbstractHeadingFound]
        );
    }
}
