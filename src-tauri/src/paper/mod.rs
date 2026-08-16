//! Paper-library domain types and services.

pub mod local_pdf_importer;
pub mod model;

pub use local_pdf_importer::{LocalPdfImportError, LocalPdfImportReceipt, LocalPdfImporter};
pub use model::{
    Author, MetadataProvenance, Paper, PaperId, PaperIdentifier, PaperMetadata, Publication,
};
