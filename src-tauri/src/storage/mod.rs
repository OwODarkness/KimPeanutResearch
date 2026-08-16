//! Local managed-file storage primitives.
//!
//! Higher-level import services decide where files belong and whether a copy is
//! valid. This module provides small filesystem operations without paper or
//! database knowledge.

pub mod file_utils;

pub use file_utils::{copy_file_with_sha256, CopiedFile};
