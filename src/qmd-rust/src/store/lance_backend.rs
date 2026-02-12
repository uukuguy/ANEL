//! LanceDB backend implementation for QMD
//!
//! This module provides LanceDB as an alternative backend for both
//! full-text search (BM25) and vector similarity search.

#[cfg(feature = "lancedb")]
pub mod lance_backend;

#[cfg(feature = "lancedb")]
pub use lance_backend::LanceDbBackend;
