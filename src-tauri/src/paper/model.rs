//! Stable paper-library records.
//!
//! These types describe research data only. Database mappings, file storage,
//! external API clients, and LLM-derived analysis belong to their own layers.

use serde::{Deserialize, Serialize};

/// Stable identifier assigned by the local research library.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PaperId(pub String);

/// A paper stored in the local research library.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Paper {
    /// Local library identity. This must remain stable even when metadata changes.
    pub id: PaperId,
    /// Bibliographic data collected from the user, a PDF, or an external provider.
    pub metadata: PaperMetadata,
    /// Unix time in milliseconds when this record entered the local library.
    pub created_at_unix_ms: i64,
    /// Unix time in milliseconds of the latest library-side record update.
    pub updated_at_unix_ms: i64,
}

/// Bibliographic metadata for a paper.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaperMetadata {
    pub title: String,
    #[serde(default)]
    pub authors: Vec<Author>,
    pub abstract_text: Option<String>,
    pub publication: Option<Publication>,
    #[serde(default)]
    pub identifiers: Vec<PaperIdentifier>,
    pub landing_page_url: Option<String>,
    pub language: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Record where this metadata was obtained; it is not an assertion of truth.
    pub provenance: MetadataProvenance,
}

/// A named author in the bibliographic record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub display_name: String,
    pub orcid: Option<String>,
}

/// Publication venue and date information.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Publication {
    pub venue: Option<String>,
    pub publisher: Option<String>,
    /// ISO 8601 date when known. Partial dates such as `2026` are allowed.
    pub published_on: Option<String>,
}

/// A standardized external identity for a paper.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PaperIdentifier {
    Doi(String),
    Arxiv(String),
    OpenAlex(String),
    SemanticScholar(String),
    Url(String),
}

/// Origin information for bibliographic metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataProvenance {
    /// Provider or acquisition method, for example `manual`, `pdf`, or `crossref`.
    pub source: String,
    pub source_url: Option<String>,
    /// Unix time in milliseconds when the metadata was acquired or confirmed.
    pub observed_at_unix_ms: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_for_tauri_ipc_in_camel_case() {
        let paper = Paper {
            id: PaperId("paper_01".into()),
            metadata: PaperMetadata {
                title: "Local-first research".into(),
                authors: vec![Author {
                    display_name: "Kim Peanut".into(),
                    orcid: None,
                }],
                abstract_text: None,
                publication: None,
                identifiers: vec![PaperIdentifier::Arxiv("2608.00001".into())],
                landing_page_url: None,
                language: Some("en".into()),
                keywords: vec!["research".into()],
                provenance: MetadataProvenance {
                    source: "manual".into(),
                    source_url: None,
                    observed_at_unix_ms: 0,
                },
            },
            created_at_unix_ms: 0,
            updated_at_unix_ms: 0,
        };

        let value = serde_json::to_value(paper).expect("paper must serialize");
        assert!(value.get("createdAtUnixMs").is_some());
        assert!(value.get("updatedAtUnixMs").is_some());
        assert!(value.get("created_at_unix_ms").is_none());
    }
}
