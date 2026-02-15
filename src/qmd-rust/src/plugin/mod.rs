// QMD Plugin System
// Provides Wasm plugin support for extending search functionality

pub mod manager;
pub mod error;
pub mod types;

pub use error::{PluginError, Result};
pub use manager::{Plugin, PluginManager, PluginInfo};
pub use types::{Scorer, Filter, Transform, Preprocessor, Postprocessor};
