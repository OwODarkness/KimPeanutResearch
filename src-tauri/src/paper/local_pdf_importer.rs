//! Local-PDF ingestion service.
//!
//! This service owns only managed-file creation. Database records, parsing,
//! metadata extraction, and import-state recovery are deliberately separate
//! steps that can build on the returned receipt.

use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

use crate::{
    backend_config::StorageConfig,
    storage::{copy_file_with_sha256, CopiedFile},
};

static PAPER_ID_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const MAX_PAPER_ID_ATTEMPTS: u32 = 16;

/// Result of storing a selected local PDF in managed library storage.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalPdfImportReceipt {
    /// Opaque managed-library identity for the imported PDF.
    pub paper_id: String,
    /// Original filename for display only; the original absolute path is not returned.
    pub source_file_name: String,
    pub byte_count: u64,
    pub sha256: String,
}

/// Errors a UI or caller can safely present while importing a local PDF.
#[derive(Debug)]
pub enum LocalPdfImportError {
    SourceIsNotAFile,
    SourceIsNotPdf,
    CouldNotAllocatePaperDirectory,
    Io(io::Error),
}

impl fmt::Display for LocalPdfImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceIsNotAFile => {
                formatter.write_str("The selected item is not a readable file.")
            }
            Self::SourceIsNotPdf => formatter.write_str("Select a file with a .pdf extension."),
            Self::CouldNotAllocatePaperDirectory => {
                formatter.write_str("Could not allocate managed storage for this paper.")
            }
            Self::Io(_) => {
                formatter.write_str("The PDF could not be copied into the local library.")
            }
        }
    }
}

impl std::error::Error for LocalPdfImportError {}

impl From<io::Error> for LocalPdfImportError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Copies selected PDFs into one managed directory per paper.
#[derive(Clone, Debug)]
pub struct LocalPdfImporter {
    storage: StorageConfig,
}

impl LocalPdfImporter {
    pub fn new(storage: StorageConfig) -> Self {
        Self { storage }
    }

    /// Imports a user-selected PDF without reading metadata or touching SQLite.
    pub fn import_file(&self, source: &Path) -> Result<LocalPdfImportReceipt, LocalPdfImportError> {
        validate_pdf_file(source)?;

        fs::create_dir_all(self.storage.papers_dir())?;
        let (paper_id, paper_dir) = self.create_paper_directory()?;
        let destination = paper_dir.join("paper.pdf");

        let copied = match copy_file_with_sha256(source, &destination) {
            Ok(copied) => copied,
            Err(error) => {
                // This directory was allocated by this call and contains no
                // other managed artifacts, so removing it cannot affect an
                // existing imported paper.
                let _ = fs::remove_dir_all(&paper_dir);
                return Err(error.into());
            }
        };

        Ok(receipt(paper_id, source, copied))
    }

    fn create_paper_directory(&self) -> Result<(String, PathBuf), LocalPdfImportError> {
        for _ in 0..MAX_PAPER_ID_ATTEMPTS {
            let paper_id = next_paper_id();
            let paper_dir = self.storage.paper_dir(&paper_id);
            match fs::create_dir(&paper_dir) {
                Ok(()) => return Ok((paper_id, paper_dir)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }

        Err(LocalPdfImportError::CouldNotAllocatePaperDirectory)
    }
}

fn validate_pdf_file(source: &Path) -> Result<(), LocalPdfImportError> {
    let metadata = fs::metadata(source)?;
    if !metadata.is_file() {
        return Err(LocalPdfImportError::SourceIsNotAFile);
    }

    let is_pdf = source
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"));
    if !is_pdf {
        return Err(LocalPdfImportError::SourceIsNotPdf);
    }

    Ok(())
}

fn next_paper_id() -> String {
    let timestamp_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let sequence = PAPER_ID_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("{timestamp_ns:032x}{sequence:016x}")
}

fn receipt(paper_id: String, source: &Path, copied: CopiedFile) -> LocalPdfImportReceipt {
    LocalPdfImportReceipt {
        paper_id,
        source_file_name: source
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default(),
        byte_count: copied.byte_count,
        sha256: copied.sha256,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicUsize, Ordering},
    };

    use crate::backend_config::StorageConfig;

    use super::{LocalPdfImportError, LocalPdfImporter};

    static TEST_DIRECTORY_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "kimpeanut-local-pdf-importer-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("test directory must be created");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("test directory must be removed");
        }
    }

    #[test]
    fn copies_a_pdf_into_an_opaque_managed_directory() {
        let directory = TestDirectory::new();
        let source = directory.path().join("source.pdf");
        let data_dir = directory.path().join("library-data");
        fs::write(&source, b"%PDF-test-content").expect("source must be written");
        let importer = LocalPdfImporter::new(StorageConfig::new(&data_dir));

        let imported = importer.import_file(&source).expect("import must succeed");

        assert_eq!(imported.source_file_name, "source.pdf");
        assert_eq!(imported.byte_count, 17);
        assert_eq!(
            fs::read(
                data_dir
                    .join("papers")
                    .join(&imported.paper_id)
                    .join("paper.pdf")
            )
            .expect("managed PDF must be readable"),
            b"%PDF-test-content"
        );
    }

    #[test]
    fn rejects_a_non_pdf_file_before_creating_storage() {
        let directory = TestDirectory::new();
        let source = directory.path().join("notes.txt");
        let data_dir = directory.path().join("library-data");
        fs::write(&source, b"not a PDF").expect("source must be written");
        let importer = LocalPdfImporter::new(StorageConfig::new(&data_dir));

        let error = importer.import_file(&source).expect_err("import must fail");

        assert!(matches!(error, LocalPdfImportError::SourceIsNotPdf));
        assert!(!data_dir.exists());
    }
}
