//! Virtual path system for QMD
//!
//! Provides utilities for handling qmd:// virtual paths that represent
//! documents in the knowledge base.
//!
//! Virtual paths follow the format: `qmd://collection-name/path/to/file.md`
//! Or bare format: `collection-name/path/to/file.md`

use serde::{Deserialize, Serialize};

/// Represents a parsed virtual path
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VirtualPath {
    /// The collection name
    pub collection: String,
    /// The relative path within the collection
    pub path: String,
}

/// Normalize a virtual path by handling various formats.
///
/// Handles:
/// - `qmd:////collection/path` -> `qmd://collection/path`
/// - `//collection/path` -> `qmd://collection/path`
/// - Bare paths are returned unchanged
pub fn normalize_virtual_path(input: &str) -> String {
    let path = input.trim();

    // Handle qmd:// with extra slashes: qmd:////collection/path -> qmd://collection/path
    if path.starts_with("qmd:") {
        // Remove qmd: prefix and normalize slashes
        let mut path = path[4..].to_string();
        // Remove leading slashes and re-add exactly two
        path = path.trim_start_matches('/').to_string();
        return format!("qmd://{path}");
    }

    // Handle //collection/path (missing qmd: prefix)
    if path.starts_with("//") {
        let path = path.trim_start_matches('/').to_string();
        return format!("qmd://{path}");
    }

    // Return as-is for other cases (filesystem paths, docids, bare collection/path, etc.)
    path.to_string()
}

/// Parse a virtual path like "qmd://collection-name/path/to/file.md"
/// into its components. Also supports collection root:
/// "qmd://collection-name/" or "qmd://collection-name"
pub fn parse_virtual_path(virtual_path: &str) -> Option<VirtualPath> {
    // Normalize the path first
    let normalized = normalize_virtual_path(virtual_path);

    // Match: qmd://collection-name[/optional-path]
    // Allows: qmd://name, qmd://name/, qmd://name/path
    let pattern = normalized.strip_prefix("qmd://")?;

    // Find the first slash to separate collection from path
    if let Some((collection, path)) = pattern.split_once('/') {
        Some(VirtualPath {
            collection: collection.to_string(),
            path: path.to_string(),
        })
    } else {
        // Collection root: qmd://collection-name
        Some(VirtualPath {
            collection: pattern.to_string(),
            path: String::new(),
        })
    }
}

/// Build a virtual path from collection name and relative path.
pub fn build_virtual_path(collection_name: &str, path: &str) -> String {
    if path.is_empty() {
        format!("qmd://{}/", collection_name)
    } else {
        format!("qmd://{}/{}", collection_name, path)
    }
}

/// Check if a path is explicitly a virtual path.
///
/// Only recognizes explicit virtual path formats:
/// - qmd://collection/path.md
/// - //collection/path.md
///
/// Does NOT consider bare collection/path.md as virtual - that should be
/// handled separately by checking if the first component is a collection name.
pub fn is_virtual_path(path: &str) -> bool {
    let trimmed = path.trim();

    // Explicit qmd:// prefix (with any number of slashes)
    if trimmed.starts_with("qmd:") {
        return true;
    }

    // //collection/path format (missing qmd: prefix)
    if trimmed.starts_with("//") {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_virtual_path() {
        // Test qmd:// with extra slashes
        assert_eq!(
            normalize_virtual_path("qmd:////collection/path"),
            "qmd://collection/path"
        );
        assert_eq!(
            normalize_virtual_path("qmd:////collection/"),
            "qmd://collection/"
        );

        // Test // prefix (missing qmd:)
        assert_eq!(
            normalize_virtual_path("//collection/path"),
            "qmd://collection/path"
        );

        // Test bare paths (unchanged)
        assert_eq!(normalize_virtual_path("collection/path"), "collection/path");
        assert_eq!(normalize_virtual_path("/absolute/path"), "/absolute/path");
    }

    #[test]
    fn test_parse_virtual_path() {
        // Full path
        let result = parse_virtual_path("qmd://collection/path/to/file.md");
        assert!(result.is_some());
        let vp = result.unwrap();
        assert_eq!(vp.collection, "collection");
        assert_eq!(vp.path, "path/to/file.md");

        // Collection root with trailing slash
        let result = parse_virtual_path("qmd://collection/");
        assert!(result.is_some());
        let vp = result.unwrap();
        assert_eq!(vp.collection, "collection");
        assert_eq!(vp.path, "");

        // Collection root without trailing slash
        let result = parse_virtual_path("qmd://collection");
        assert!(result.is_some());
        let vp = result.unwrap();
        assert_eq!(vp.collection, "collection");
        assert_eq!(vp.path, "");

        // Normalize // prefix
        let result = parse_virtual_path("//collection/path");
        assert!(result.is_some());
        let vp = result.unwrap();
        assert_eq!(vp.collection, "collection");
        assert_eq!(vp.path, "path");

        // Invalid virtual path
        let result = parse_virtual_path("collection/path");
        assert!(result.is_none());
    }

    #[test]
    fn test_build_virtual_path() {
        assert_eq!(
            build_virtual_path("collection", "path/to/file.md"),
            "qmd://collection/path/to/file.md"
        );
        assert_eq!(
            build_virtual_path("collection", ""),
            "qmd://collection/"
        );
    }

    #[test]
    fn test_is_virtual_path() {
        // True for explicit virtual paths
        assert!(is_virtual_path("qmd://collection/path"));
        assert!(is_virtual_path("qmd://collection/"));
        assert!(is_virtual_path("//collection/path"));
        assert!(is_virtual_path("qmd:///collection/path")); // extra slashes

        // False for bare paths
        assert!(!is_virtual_path("collection/path"));
        assert!(!is_virtual_path("/absolute/path"));
        assert!(!is_virtual_path("file.md"));
    }
}
