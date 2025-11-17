use crate::config::Config;
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;

/// Get the user's music directory
/// If user has a library_path set, use that
/// Otherwise, fall back to the global music_dir from config
pub fn get_user_music_dir(config: &Config, library_path: &Option<String>) -> PathBuf {
    match library_path {
        Some(path) => PathBuf::from(path),
        None => config.paths.music_dir.clone(),
    }
}

/// Get the user's temporary directory
/// If user has a library_path set, use {library_path}/tmp
/// Otherwise, fall back to the global temp_dir from config
pub fn get_user_temp_dir(config: &Config, library_path: &Option<String>) -> PathBuf {
    match library_path {
        Some(path) => {
            let mut temp_path = PathBuf::from(path);
            temp_path.push("tmp");
            temp_path
        }
        None => config.paths.temp_dir.clone(),
    }
}

/// Ensure a directory exists, creating it if necessary
/// Returns an error if the directory cannot be created
pub async fn ensure_directory_exists(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .await
            .context(format!("Failed to create directory: {}", path.display()))?;
        tracing::info!("Created directory: {}", path.display());
    }
    Ok(())
}

/// Get both the music and temp directories for a user, ensuring they exist
/// Returns (music_dir, temp_dir)
pub async fn get_user_directories(
    config: &Config,
    library_path: &Option<String>,
) -> Result<(PathBuf, PathBuf)> {
    let music_dir = get_user_music_dir(config, library_path);
    let temp_dir = get_user_temp_dir(config, library_path);

    // Ensure both directories exist
    ensure_directory_exists(&music_dir).await?;
    ensure_directory_exists(&temp_dir).await?;

    Ok((music_dir, temp_dir))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_get_user_music_dir_with_library_path() {
        let config = Config::default();
        let library_path = Some("/srv/navidrome/music/jcledesma".to_string());

        let result = get_user_music_dir(&config, &library_path);
        assert_eq!(result, Path::new("/srv/navidrome/music/jcledesma"));
    }

    #[test]
    fn test_get_user_music_dir_without_library_path() {
        let config = Config::default();
        let library_path = None;

        let result = get_user_music_dir(&config, &library_path);
        assert_eq!(result, config.paths.music_dir);
    }

    #[test]
    fn test_get_user_temp_dir_with_library_path() {
        let config = Config::default();
        let library_path = Some("/srv/navidrome/music/jcledesma".to_string());

        let result = get_user_temp_dir(&config, &library_path);
        assert_eq!(result, Path::new("/srv/navidrome/music/jcledesma/tmp"));
    }

    #[test]
    fn test_get_user_temp_dir_without_library_path() {
        let config = Config::default();
        let library_path = None;

        let result = get_user_temp_dir(&config, &library_path);
        assert_eq!(result, config.paths.temp_dir);
    }
}
