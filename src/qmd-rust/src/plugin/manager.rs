// Plugin Manager - Wasmtime integration for QMD plugins

use crate::plugin::error::{PluginError, Result};
use crate::plugin::types::TransformResult;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use wasmtime::{Engine, Module};

/// Plugin information
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub description: String,
    pub author: String,
}

/// Loaded plugin (simplified - stores module reference)
pub struct Plugin {
    info: PluginInfo,
    module: Module,
}

/// Plugin manager for loading and managing Wasm plugins
pub struct PluginManager {
    plugins: RwLock<HashMap<String, Plugin>>,
    plugins_dir: PathBuf,
    engine: Engine,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(plugins_dir: impl Into<PathBuf>) -> Result<Self> {
        let plugins_dir = plugins_dir.into();

        // Create plugins directory if it doesn't exist
        if !plugins_dir.exists() {
            std::fs::create_dir_all(&plugins_dir)?;
        }

        // Create wasmtime engine
        let engine = Engine::default();

        Ok(Self {
            plugins: RwLock::new(HashMap::new()),
            plugins_dir,
            engine,
        })
    }

    /// Get the plugins directory
    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }

    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        let plugins = self.plugins.read().unwrap();
        plugins.values().map(|p| p.info.clone()).collect()
    }

    /// List all available plugin files in the plugins directory
    pub fn list_available_plugins(&self) -> Result<Vec<PluginInfo>> {
        let mut plugins = Vec::new();

        if !self.plugins_dir.exists() {
            return Ok(plugins);
        }

        for entry in std::fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "wasm") {
                // Try to extract info from filename (format: name-version.wasm)
                let filename = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");

                let parts: Vec<&str> = filename.splitn(2, '-').collect();
                let (name, version) = if parts.len() == 2 {
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    (filename.to_string(), "0.1.0".to_string())
                };

                plugins.push(PluginInfo {
                    name,
                    version,
                    path: path.clone(),
                    description: String::new(),
                    author: String::new(),
                });
            }
        }

        Ok(plugins)
    }

    /// Load a plugin from file
    pub fn load_plugin(&self, name: &str, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();

        // Check if already loaded
        {
            let plugins = self.plugins.read().unwrap();
            if plugins.contains_key(name) {
                return Err(PluginError::AlreadyExists(name.to_string()));
            }
        }

        // Load wasm module
        let module = Module::from_file(&self.engine, path)
            .map_err(|e| PluginError::LoadFailed(format!("Failed to load wasm module: {}", e)))?;

        // Get plugin info from module name
        let info = PluginInfo {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            path: path.to_path_buf(),
            description: String::new(),
            author: String::new(),
        };

        // Store plugin
        let plugin = Plugin {
            info,
            module,
        };

        let mut plugins = self.plugins.write().unwrap();
        plugins.insert(name.to_string(), plugin);

        Ok(())
    }

    /// Unload a plugin
    pub fn unload_plugin(&self, name: &str) -> Result<()> {
        let mut plugins = self.plugins.write().unwrap();

        if plugins.remove(name).is_none() {
            return Err(PluginError::NotFound(name.to_string()));
        }

        Ok(())
    }

    /// Call plugin scorer function
    /// Note: Full implementation requires managing WASM memory for string passing
    pub fn call_scorer(&self, name: &str, _query: &str, _title: &str, _body: &str) -> Result<f32> {
        let plugins = self.plugins.read().unwrap();
        let _plugin = plugins.get(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        // Check if scorer function exists in module
        // For now, return a default score
        // Full implementation would require:
        // 1. Allocate memory in WASM module
        // 2. Copy strings to WASM memory
        // 3. Call scorer function
        // 4. Read result from memory
        tracing::debug!("Calling scorer for plugin: {}", name);
        Ok(1.0)
    }

    /// Call plugin filter function
    pub fn call_filter(&self, name: &str, _title: &str, _body: &str) -> Result<bool> {
        let plugins = self.plugins.read().unwrap();
        let _plugin = plugins.get(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        // Check if filter function exists
        // For now, return true (include all)
        tracing::debug!("Calling filter for plugin: {}", name);
        Ok(true)
    }

    /// Call plugin transform function
    pub fn call_transform(&self, name: &str, title: &str, body: &str) -> Result<TransformResult> {
        let plugins = self.plugins.read().unwrap();
        let _plugin = plugins.get(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        // Check if transform function exists
        // For now, return original content
        tracing::debug!("Calling transform for plugin: {}", name);
        Ok(TransformResult {
            title: title.to_string(),
            body: body.to_string(),
            metadata: vec![],
        })
    }

    /// Check if a plugin is loaded
    pub fn is_loaded(&self, name: &str) -> bool {
        let plugins = self.plugins.read().unwrap();
        plugins.contains_key(name)
    }

    /// Get plugin info
    pub fn get_plugin_info(&self, name: &str) -> Result<PluginInfo> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(name)
            .map(|p| p.info.clone())
            .ok_or_else(|| PluginError::NotFound(name.to_string()))
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new("~/.cache/qmd/plugins").unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_default() {
        let manager = PluginManager::default();
        assert!(manager.list_plugins().is_empty());
    }

    #[test]
    fn test_plugin_manager_custom_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = PluginManager::new(temp_dir.path()).unwrap();
        assert_eq!(manager.plugins_dir(), temp_dir.path());
    }

    #[test]
    fn test_plugin_manager_list_available_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = PluginManager::new(temp_dir.path()).unwrap();
        assert!(manager.list_available_plugins().unwrap().is_empty());
    }

    #[test]
    fn test_is_loaded_false() {
        let manager = PluginManager::default();
        assert!(!manager.is_loaded("nonexistent"));
    }

    #[test]
    fn test_get_plugin_info_not_found() {
        let manager = PluginManager::default();
        assert!(manager.get_plugin_info("nonexistent").is_err());
    }
}
