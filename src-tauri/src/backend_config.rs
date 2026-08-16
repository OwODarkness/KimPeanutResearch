//! Backend-owned configuration and managed-library location migration.

use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

/// Current on-disk library layout understood by the backend.
pub const STORAGE_LAYOUT_VERSION: u32 = 1;
const BACKEND_SETTINGS_VERSION: u32 = 1;
const BACKEND_SETTINGS_FILE: &str = "backend-settings.json";

/// Backend runtime settings. Its settings file lives outside the movable
/// research-library data directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackendConfig {
    pub storage: StorageConfig,
    settings_path: PathBuf,
}

#[derive(Debug)]
pub enum BackendConfigError {
    InvalidSettingsVersion,
    DestinationIsNotAbsolute,
    DestinationIsNotEmpty,
    DestinationIsNotDirectory,
    Io(io::Error),
    Json(serde_json::Error),
}

impl fmt::Display for BackendConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSettingsVersion => {
                f.write_str("The backend settings file has an unsupported version.")
            }
            Self::DestinationIsNotAbsolute => f.write_str("Choose an absolute storage folder."),
            Self::DestinationIsNotEmpty => {
                f.write_str("Choose an empty folder for the new library location.")
            }
            Self::DestinationIsNotDirectory => {
                f.write_str("The selected storage location is not a directory.")
            }
            Self::Io(_) => f.write_str("Could not update the local storage configuration."),
            Self::Json(_) => f.write_str("Could not read the local storage configuration."),
        }
    }
}

impl std::error::Error for BackendConfigError {}
impl From<io::Error> for BackendConfigError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<serde_json::Error> for BackendConfigError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageLocation {
    pub data_dir: String,
    pub layout_version: u32,
    pub uses_custom_location: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageMigrationReceipt {
    pub previous_data_dir: String,
    pub data_dir: String,
    pub copied_entries: u64,
    pub previous_location_retained: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredBackendSettings {
    version: u32,
    data_dir: PathBuf,
}

impl BackendConfig {
    /// Creates ephemeral backend settings for tests and local tools.
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            storage: StorageConfig::new(data_dir),
            settings_path: PathBuf::new(),
        }
    }

    /// Loads the selected storage root, falling back to the platform default.
    pub fn load(
        default_data_dir: PathBuf,
        settings_dir: PathBuf,
    ) -> Result<Self, BackendConfigError> {
        let settings_path = settings_dir.join(BACKEND_SETTINGS_FILE);
        if !settings_path.exists() {
            return Ok(Self {
                storage: StorageConfig::new(default_data_dir),
                settings_path,
            });
        }
        let stored: StoredBackendSettings = serde_json::from_slice(&fs::read(&settings_path)?)?;
        if stored.version != BACKEND_SETTINGS_VERSION {
            return Err(BackendConfigError::InvalidSettingsVersion);
        }
        Ok(Self {
            storage: StorageConfig::new(stored.data_dir),
            settings_path,
        })
    }

    pub fn storage_location(&self) -> StorageLocation {
        StorageLocation {
            data_dir: self.storage.data_dir.display().to_string(),
            layout_version: self.storage.layout_version,
            uses_custom_location: !self.settings_path.as_os_str().is_empty()
                && self.settings_path.exists(),
        }
    }

    /// Copies managed files to an empty target and switches future operations.
    /// The previous directory is retained as a recovery fallback.
    pub fn migrate_storage_to(
        &mut self,
        destination: PathBuf,
    ) -> Result<StorageMigrationReceipt, BackendConfigError> {
        if !destination.is_absolute() {
            return Err(BackendConfigError::DestinationIsNotAbsolute);
        }
        if destination == self.storage.data_dir {
            return Ok(StorageMigrationReceipt {
                previous_data_dir: destination.display().to_string(),
                data_dir: destination.display().to_string(),
                copied_entries: 0,
                previous_location_retained: false,
            });
        }
        if destination.exists() {
            if !destination.is_dir() {
                return Err(BackendConfigError::DestinationIsNotDirectory);
            }
            if fs::read_dir(&destination)?.next().transpose()?.is_some() {
                return Err(BackendConfigError::DestinationIsNotEmpty);
            }
        } else {
            fs::create_dir_all(&destination)?;
        }

        let previous_data_dir = self.storage.data_dir.clone();
        let migration_result = (|| {
            let copied_entries = if previous_data_dir.exists() {
                copy_directory_contents(&previous_data_dir, &destination)?
            } else {
                0
            };
            self.persist_storage_location(&destination)?;
            Ok(copied_entries)
        })();
        let copied_entries = match migration_result {
            Ok(copied_entries) => copied_entries,
            Err(error) => {
                // The destination was verified empty before this migration,
                // so clearing only its newly-created entries is safe and
                // leaves it ready for a retry.
                clear_directory_contents(&destination)?;
                return Err(error);
            }
        };
        self.storage = StorageConfig::new(destination.clone());
        Ok(StorageMigrationReceipt {
            previous_data_dir: previous_data_dir.display().to_string(),
            data_dir: destination.display().to_string(),
            copied_entries,
            previous_location_retained: previous_data_dir.exists(),
        })
    }

    fn persist_storage_location(&self, data_dir: &Path) -> Result<(), BackendConfigError> {
        if self.settings_path.as_os_str().is_empty() {
            return Ok(());
        }
        let parent = self
            .settings_path
            .parent()
            .expect("settings path has parent");
        fs::create_dir_all(parent)?;
        let bytes = serde_json::to_vec_pretty(&StoredBackendSettings {
            version: BACKEND_SETTINGS_VERSION,
            data_dir: data_dir.to_path_buf(),
        })?;
        fs::write(&self.settings_path, bytes)?;
        Ok(())
    }
}

/// Location and format contract for managed research-library files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub layout_version: u32,
}
impl StorageConfig {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            layout_version: STORAGE_LAYOUT_VERSION,
        }
    }
    pub fn library_database_path(&self) -> PathBuf {
        self.data_dir.join("library.db")
    }
    pub fn papers_dir(&self) -> PathBuf {
        self.data_dir.join("papers")
    }
    pub fn papers_staging_dir(&self) -> PathBuf {
        self.papers_dir().join(".staging")
    }
    pub fn paper_dir(&self, paper_id: &str) -> PathBuf {
        self.papers_dir().join(paper_id)
    }
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }
}

fn copy_directory_contents(source: &Path, destination: &Path) -> Result<u64, BackendConfigError> {
    let mut copied = 0;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            fs::create_dir(&destination_path)?;
            copied += copy_directory_contents(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &destination_path)?;
            copied += 1;
        }
    }
    Ok(copied)
}

fn clear_directory_contents(path: &Path) -> Result<(), BackendConfigError> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry.file_type()?.is_dir() {
            fs::remove_dir_all(entry_path)?;
        } else {
            fs::remove_file(entry_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{BackendConfig, STORAGE_LAYOUT_VERSION};
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicUsize, Ordering},
    };
    static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
    struct TestDirectory(PathBuf);
    impl TestDirectory {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "kimpeanut-backend-config-{}-{}",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&p).unwrap();
            Self(p)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    #[test]
    fn derives_library_paths() {
        let c = BackendConfig::new("research-data");
        assert_eq!(c.storage.layout_version, STORAGE_LAYOUT_VERSION);
        assert_eq!(
            c.storage.library_database_path(),
            PathBuf::from("research-data/library.db")
        );
    }
    #[test]
    fn copies_library_and_persists_custom_location() {
        let t = TestDirectory::new();
        let source = t.path().join("old");
        let target = t.path().join("new");
        let settings = t.path().join("settings");
        fs::create_dir_all(source.join("papers/paper-a")).unwrap();
        fs::write(source.join("papers/paper-a/paper.pdf"), b"pdf").unwrap();
        let mut c = BackendConfig::load(source.clone(), settings).unwrap();
        let r = c.migrate_storage_to(target.clone()).unwrap();
        assert_eq!(r.copied_entries, 1);
        assert_eq!(
            fs::read(target.join("papers/paper-a/paper.pdf")).unwrap(),
            b"pdf"
        );
        assert!(source.exists());
        let loaded = BackendConfig::load(source, t.path().join("settings")).unwrap();
        assert_eq!(loaded.storage.data_dir, target);
    }
}
