use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    pub dir: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DownloadConfig {
    pub concurrency: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct PerformanceConfig {
    pub tree_strategy_threshold: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub auth: Option<AuthConfig>,
    pub output: Option<OutputConfig>,
    pub download: Option<DownloadConfig>,
    pub performance: Option<PerformanceConfig>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config file at {}", path.display()))?;
        let cfg = toml::from_str(&content).context("failed to parse config.toml")?;
        Ok(cfg)
    }

    fn config_path() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .context("could not determine config directory")?
            .join("fumi")
            .join("config.toml");
        Ok(dir)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auth: None,
            output: None,
            download: None,
            performance: None,
        }
    }
}
