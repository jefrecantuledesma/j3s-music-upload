use crate::config::Config;
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;

/// Get the user's music directory
/// If user has a library_path set, use that
/// Otherwise, fall back to the global music_dir from config
/// Special case: If library_path equals the global music_dir, append "/default"
/// to prevent files from being dumped in the root directory
pub fn get_user_music_dir(config: &Config, library_path: &Option<String>) -> PathBuf {
    match library_path {
        Some(path) => {
            let user_path = PathBuf::from(path);
            // Prevent library_path from being exactly the global music_dir
            // This avoids dumping files into the root music directory
            if user_path == config.paths.music_dir {
                let mut default_path = config.paths.music_dir.clone();
                default_path.push("default");
                default_path
            } else {
                user_path
            }
        }
        None => config.paths.music_dir.clone(),
    }
}

/// Get the user's temporary directory
/// If user has a library_path set, use {library_path}/tmp
/// Otherwise, fall back to the global temp_dir from config
/// Special case: If library_path equals global music_dir, use global temp_dir
pub fn get_user_temp_dir(config: &Config, library_path: &Option<String>) -> PathBuf {
    match library_path {
        Some(path) => {
            let user_path = PathBuf::from(path);
            // If library_path equals global music_dir, use global temp_dir
            // (we don't want to create a tmp folder inside the "default" folder)
            if user_path == config.paths.music_dir {
                config.paths.temp_dir.clone()
            } else {
                // Use user's library_path + tmp
                let mut temp_path = user_path;
                temp_path.push("tmp");
                temp_path
            }
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
    fn test_get_user_music_dir_equals_global_music_dir() {
        let config = Config::default();
        // When library_path equals global music_dir, it should append "/default"
        let library_path = Some(config.paths.music_dir.to_string_lossy().to_string());

        let result = get_user_music_dir(&config, &library_path);
        let mut expected = config.paths.music_dir.clone();
        expected.push("default");
        assert_eq!(result, expected);
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

    #[test]
    fn test_get_user_temp_dir_equals_global_music_dir() {
        let config = Config::default();
        // When library_path equals global music_dir, use global temp_dir
        // (don't create a tmp folder inside the "default" folder)
        let library_path = Some(config.paths.music_dir.to_string_lossy().to_string());

        let result = get_user_temp_dir(&config, &library_path);
        assert_eq!(result, config.paths.temp_dir);
    }
}
