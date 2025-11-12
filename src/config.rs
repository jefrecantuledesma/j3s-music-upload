use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    #[serde(default = "YoutubeConfig::default_format_selector")]
    pub format_selector: String,
    #[serde(default = "YoutubeConfig::default_player_client")]
    pub player_client: Option<String>,
    #[serde(default)]
    pub extra_args: Vec<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path =
            std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());

        let mut config: Config = if std::path::Path::new(&config_path).exists() {
            let content = std::fs::read_to_string(&config_path)
                .context(format!("Failed to read config file at {}", config_path))?;
            toml::from_str(&content).context("Failed to parse config file")?
        } else {
            tracing::warn!("Config file not found at {}, using defaults", config_path);
            Self::default()
        };

        // Override with environment variables if present
        config.apply_env_overrides();

        // Auto-generate JWT secret if it's the default placeholder or empty
        if config.security.jwt_secret.is_empty()
            || config.security.jwt_secret == "your-secret-key-here-change-this"
        {
            tracing::warn!("JWT secret not configured, generating random secret");
            config.security.jwt_secret = Self::generate_jwt_secret();
        }

        // Validate paths exist or can be created
        config.validate()?;

        Ok(config)
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(host) = std::env::var("SERVER_HOST") {
            self.server.host = host;
        }
        if let Ok(port) = std::env::var("SERVER_PORT") {
            if let Ok(port_num) = port.parse() {
                self.server.port = port_num;
            }
        }
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            self.database.url = db_url;
        }
        if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
            self.security.jwt_secret = jwt_secret;
        }
        if let Ok(music_dir) = std::env::var("MUSIC_DIR") {
            self.paths.music_dir = std::path::PathBuf::from(music_dir);
        }
        if let Ok(temp_dir) = std::env::var("TEMP_DIR") {
            self.paths.temp_dir = std::path::PathBuf::from(temp_dir);
        }
    }

    fn generate_jwt_secret() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut rng = rand::thread_rng();
        (0..43) // Base64 encoded 32 bytes = 43 characters
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    fn validate(&self) -> Result<()> {
        // Create temp_dir if it doesn't exist
        if !self.paths.temp_dir.exists() {
            tracing::info!("Creating temp directory: {:?}", self.paths.temp_dir);
            std::fs::create_dir_all(&self.paths.temp_dir)
                .context("Failed to create temp directory")?;
        }

        // Create music_dir if it doesn't exist (for easier setup)
        if !self.paths.music_dir.exists() {
            tracing::warn!(
                "Music directory does not exist, creating: {:?}",
                self.paths.music_dir
            );
            std::fs::create_dir_all(&self.paths.music_dir)
                .context("Failed to create music directory")?;
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

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
            },
            database: DatabaseConfig {
                url: "sqlite:./data/music_upload.db".to_string(),
                max_connections: 5,
            },
            paths: PathsConfig {
                music_dir: PathBuf::from("/tmp/music"),
                temp_dir: PathBuf::from("/tmp/music_upload"),
                ferric_path: PathBuf::from("/usr/local/bin/ferric"),
            },
            security: SecurityConfig {
                jwt_secret: "your-secret-key-here-change-this".to_string(),
                session_timeout_hours: 24,
            },
            upload: UploadConfig {
                max_file_size_mb: 500,
                allowed_extensions: vec![
                    "mp3".to_string(),
                    "flac".to_string(),
                    "ogg".to_string(),
                    "opus".to_string(),
                    "m4a".to_string(),
                    "wav".to_string(),
                    "aac".to_string(),
                ],
            },
            youtube: YoutubeConfig {
                enabled: true,
                ytdlp_path: "yt-dlp".to_string(),
                audio_format: "best".to_string(),
                format_selector: YoutubeConfig::default_format_selector(),
                player_client: YoutubeConfig::default_player_client(),
                extra_args: Vec::new(),
            },
        }
    }
}

impl YoutubeConfig {
    fn default_format_selector() -> String {
        "bestaudio/best".to_string()
    }

    fn default_player_client() -> Option<String> {
        Some("web".to_string())
    }
}
