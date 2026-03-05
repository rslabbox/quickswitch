use anyhow::Result;
use std::{fs, path::PathBuf};
use tracing::{debug, instrument};

/// Get the data directory for quickswitch
///
/// This function reads the `_QUICKSWITCH_DATA_DIR` environment variable.
/// If the environment variable is not set or empty, it returns a suitable default directory:
/// - On Unix-like systems: `~/.local/share/quickswitch`
/// - On Windows: `%APPDATA%\quickswitch`
///
/// The function will create the directory if it doesn't exist.
pub fn get_data_dir() -> Result<PathBuf> {
    // First, try to read from environment variable
    if let Ok(env_dir) = std::env::var("_QUICKSWITCH_DATA_DIR")
        && !env_dir.trim().is_empty() {
            let data_dir = PathBuf::from(env_dir);
            // Create directory if it doesn't exist
            if !data_dir.exists() {
                fs::create_dir_all(&data_dir)?;
            }
            return Ok(data_dir);
        }

    // If environment variable is not set or empty, use default directory
    let data_dir = get_default_data_dir()?;

    // Create directory if it doesn't exist
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
    }

    Ok(data_dir)
}

/// Get the default data directory based on the operating system
fn get_default_data_dir() -> Result<PathBuf> {
    #[cfg(windows)]
    {
        // On Windows, use %APPDATA%\quickswitch
        if let Ok(appdata) = std::env::var("APPDATA") {
            Ok(PathBuf::from(appdata).join("quickswitch"))
        } else {
            // Fallback to temp directory if APPDATA is not available
            Ok(std::env::temp_dir().join("quickswitch"))
        }
    }

    #[cfg(not(windows))]
    {
        // On Unix-like systems, use ~/.local/share/quickswitch
        if let Ok(home) = std::env::var("HOME") {
            Ok(PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("quickswitch"))
        } else if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
            // Follow XDG Base Directory Specification
            Ok(PathBuf::from(xdg_data_home).join("quickswitch"))
        } else {
            // Fallback to temp directory if HOME is not available
            Ok(std::env::temp_dir().join("quickswitch"))
        }
    }
}

/// Configuration for history functionality
#[derive(Debug)]
pub struct HistoryConfig {
    /// Maximum number of history entries to keep
    pub max_entries: usize,
    /// Sort mode for history entries
    pub sort_mode: crate::utils::HistorySortMode,
    /// Number of days for time decay calculation
    pub time_decay_days: u32,
    /// Minimum frequency threshold for keeping entries
    pub min_frequency_threshold: u32,
}

impl Default for HistoryConfig {
    #[instrument]
    fn default() -> Self {
        let config = Self {
            max_entries: 100,
            sort_mode: crate::utils::HistorySortMode::FrequencyRecent,
            time_decay_days: 30,
            min_frequency_threshold: 1,
        };
        debug!(?config, "Created default HistoryConfig");
        config
    }
}

/// Get the history configuration
pub fn get_history_config() -> HistoryConfig {
    // In the future, this could read from a config file
    HistoryConfig::default()
}
