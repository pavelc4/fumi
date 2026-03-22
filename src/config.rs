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
            .with_context(|| format!("failed to read config at {}", path.display()))?;
        toml::from_str(&content).context("failed to parse config.toml")
    }

    pub fn load_or_create() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, Self::default_template())?;
            eprintln!("created default config at {}", path.display());
        }
        let content = fs::read_to_string(&path)?;
        toml::from_str(&content).context("failed to parse config.toml")
    }

    fn default_template() -> &'static str {
        r#"# fumi config

        [auth]
        # token = "ghp_your_token_here"  # optional, for private repos / higher rate limit

        [output]
        # dir = "~/Downloads/fumi"       # default: ~/Downloads

        [download]
        concurrency = 4

        [performance]
        # tree_strategy_threshold = 50   # switch to full-tree fetch if dir > N entries
        "#
    }

    fn config_path() -> Result<PathBuf> {
        Ok(dirs::config_dir()
            .context("could not determine config directory")?
            .join("fumi")
            .join("config.toml"))
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
