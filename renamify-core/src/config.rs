use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub defaults: DefaultsConfig,

    /// List of atomic identifiers (treated as indivisible units)
    #[serde(default)]
    pub atomic: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    /// Default preview format: "table", "diff", "json", or "summary"
    #[serde(default = "default_preview")]
    pub preview_format: String,

    /// Whether to rename files by default
    #[serde(default = "default_true")]
    pub rename_files: bool,

    /// Whether to rename directories by default
    #[serde(default = "default_true")]
    pub rename_dirs: bool,

    /// Default unrestricted level (0=respect gitignore, 1=-u, 2=-uu, 3=-uuu)
    #[serde(default)]
    pub unrestricted_level: u8,

    /// Whether to use color output by default (None = auto-detect)
    #[serde(default)]
    pub use_color: Option<bool>,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            preview_format: default_preview(),
            rename_files: true,
            rename_dirs: true,
            unrestricted_level: 0,
            use_color: None,
        }
    }
}

fn default_preview() -> String {
    "diff".to_string()
}

fn default_true() -> bool {
    true
}

impl Config {
    /// Load config from .renamify/config.toml if it exists
    pub fn load() -> Result<Self> {
        if let Ok(cwd) = std::env::current_dir() {
            let config_path = cwd.join(".renamify").join("config.toml");
            if config_path.exists() {
                return Self::load_from_path(&config_path);
            }
        }

        // Return default config if no config file exists
        Ok(Self::default())
    }

    /// Load config from a specific path
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save config to .renamify/config.toml
    pub fn save(&self) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let config_dir = cwd.join(".renamify");
        let config_path = config_dir.join("config.toml");

        // Create .renamify directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        self.save_to_path(&config_path)
    }

    /// Save config to a specific path
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.defaults.preview_format, "diff");
        assert!(config.defaults.rename_files);
        assert!(config.defaults.rename_dirs);
        assert_eq!(config.defaults.unrestricted_level, 0);
        assert_eq!(config.defaults.use_color, None);
    }

    #[test]
    fn test_load_save_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mut config = Config::default();
        config.defaults.preview_format = "table".to_string();
        config.defaults.rename_files = false;
        config.defaults.unrestricted_level = 1;
        config.defaults.use_color = Some(true);

        config.save_to_path(&config_path).unwrap();

        let loaded_config = Config::load_from_path(&config_path).unwrap();
        assert_eq!(loaded_config.defaults.preview_format, "table");
        assert!(!loaded_config.defaults.rename_files);
        assert_eq!(loaded_config.defaults.unrestricted_level, 1);
        assert_eq!(loaded_config.defaults.use_color, Some(true));
    }

    #[test]
    fn test_partial_config() {
        let toml_content = r#"
[defaults]
preview_format = "json"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.defaults.preview_format, "json");
        // Other fields should have their defaults
        assert!(config.defaults.rename_files);
        assert!(config.defaults.rename_dirs);
        assert_eq!(config.defaults.unrestricted_level, 0);
    }

    #[test]
    fn test_atomic_config() {
        let toml_content = r#"
atomic = ["DocSpring", "GitHub", "FormAPI"]

[defaults]
preview_format = "table"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.atomic.len(), 3);
        assert!(config.atomic.contains(&"DocSpring".to_string()));
        assert!(config.atomic.contains(&"GitHub".to_string()));
        assert!(config.atomic.contains(&"FormAPI".to_string()));
        assert_eq!(config.defaults.preview_format, "table");
    }
}
