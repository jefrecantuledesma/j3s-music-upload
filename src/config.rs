use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub paths: PathsConfig,
    pub security: SecurityConfig,
    pub upload: UploadConfig,
    pub youtube: YoutubeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    pub music_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub ferric_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub session_timeout_hours: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadConfig {
    pub max_file_size_mb: u64,
    pub allowed_extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoutubeConfig {
    pub enabled: bool,
    pub ytdlp_path: String,
    pub audio_format: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = std::env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config.toml".to_string());

        let content = std::fs::read_to_string(&config_path)
            .context(format!("Failed to read config file at {}", config_path))?;

        let config: Config = toml::from_str(&content)
            .context("Failed to parse config file")?;

        // Validate paths exist or can be created
        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        // Create temp_dir if it doesn't exist
        if !self.paths.temp_dir.exists() {
            std::fs::create_dir_all(&self.paths.temp_dir)
                .context("Failed to create temp directory")?;
        }

        // Check that music_dir exists
        if !self.paths.music_dir.exists() {
            anyhow::bail!("Music directory does not exist: {:?}", self.paths.music_dir);
        }

        Ok(())
    }

    pub fn max_file_size_bytes(&self) -> usize {
        (self.upload.max_file_size_mb * 1024 * 1024) as usize
    }
}

// Database-stored configuration (for runtime editable settings)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbConfig {
    pub key: String,
    pub value: String,
}
