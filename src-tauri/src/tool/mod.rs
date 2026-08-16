//! Narrow, side-effect-free application tools.
//!
//! A tool coordinates a specific application capability. It does not own PDF
//! parsing, storage, database access, or Tauri IPC; callers inject already
//! parsed data and decide whether and how to persist the result.

pub mod paper_extraction;

pub use paper_extraction::{
    Extracted, ExtractedPaperInfo, ExtractionField, ExtractionRequest, ExtractionWarning, PageText,
    PaperExtractionTool, ParsedPaperDocument, SourceLocation,
};
