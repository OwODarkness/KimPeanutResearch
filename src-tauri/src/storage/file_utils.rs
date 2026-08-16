//! Streaming file-copy helpers for managed library storage.

use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Write},
    path::Path,
};

use sha2::{Digest, Sha256};

const COPY_BUFFER_SIZE: usize = 64 * 1024;

/// Result of copying a file into managed storage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopiedFile {
    /// Number of bytes written to the destination.
    pub byte_count: u64,
    /// Lowercase hexadecimal SHA-256 of exactly the copied bytes.
    pub sha256: String,
}

/// Copies `source` to a new `destination` while calculating its SHA-256.
///
/// The destination parent must already exist. This function never overwrites
/// an existing destination; callers should create a unique staging path before
/// calling it. The destination is flushed and synchronized before success is
/// returned.
pub fn copy_file_with_sha256(source: &Path, destination: &Path) -> io::Result<CopiedFile> {
    let source_file = File::open(source)?;
    let destination_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;

    let mut reader = BufReader::new(source_file);
    let mut writer = BufWriter::new(destination_file);
    let mut hasher = Sha256::new();
    let mut byte_count = 0_u64;
    let mut buffer = [0_u8; COPY_BUFFER_SIZE];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        writer.write_all(&buffer[..bytes_read])?;
        hasher.update(&buffer[..bytes_read]);
        byte_count += bytes_read as u64;
    }

    writer.flush()?;
    writer.get_ref().sync_all()?;

    Ok(CopiedFile {
        byte_count,
        sha256: format!("{:x}", hasher.finalize()),
    })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::ErrorKind,
        path::{Path, PathBuf},
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::copy_file_with_sha256;

    static TEST_DIRECTORY_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "kimpeanut-file-utils-{}-{sequence}",
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
    fn copies_content_and_reports_sha256() {
        let directory = TestDirectory::new();
        let source = directory.path().join("source.pdf");
        let destination = directory.path().join("paper.pdf");
        fs::write(&source, b"hello world").expect("source must be written");

        let copied = copy_file_with_sha256(&source, &destination).expect("copy must succeed");

        assert_eq!(copied.byte_count, 11);
        assert_eq!(
            copied.sha256,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
        assert_eq!(
            fs::read(destination).expect("destination must be readable"),
            b"hello world"
        );
    }

    #[test]
    fn does_not_overwrite_an_existing_destination() {
        let directory = TestDirectory::new();
        let source = directory.path().join("source.pdf");
        let destination = directory.path().join("paper.pdf");
        fs::write(&source, b"source").expect("source must be written");
        fs::write(&destination, b"existing").expect("destination must be written");

        let error = copy_file_with_sha256(&source, &destination).expect_err("copy must fail");

        assert_eq!(error.kind(), ErrorKind::AlreadyExists);
        assert_eq!(
            fs::read(destination).expect("destination must be readable"),
            b"existing"
        );
    }
}
