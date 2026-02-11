use crate::constants::APP_NAME;
use directories::ProjectDirs;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub username: String,
    // Use Option to handle both cases (Some = text file, None = keyring)
    pub password: Option<String>,
}

// Updated signature to accept the flag
pub fn save_credentials(
    username: &str,
    password: &str,
    skip_encryption: bool,
) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    let config_path = config_dir.join("config.json");

    // LOGIC: If skipping encryption, save password in struct. Otherwise, keep it None.
    let config = Config {
        username: username.to_string(),
        password: if skip_encryption {
            Some(password.to_string())
        } else {
            None
        },
    };

    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&config_path, json).map_err(|e| e.to_string())?;

    if skip_encryption {
        println!("⚠️  WARNING: Password saved in plain text (Insecure Mode)");
    } else {
        // Only try to use the OS keyring if the user didn't skip it
        let entry = Entry::new(APP_NAME, username).map_err(|e| e.to_string())?;
        entry.set_password(password).map_err(|e| e.to_string())?;
        println!("🔒 Password securely saved in OS Keyring");
    }

    Ok(())
}

pub fn get_credentials() -> Result<(String, String), String> {
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("config.json");

    let file_content = fs::read_to_string(&config_path)
        .map_err(|_| format!("No config found. Run 'uqgrd credentials' first."))?;

    let config: Config = serde_json::from_str(&file_content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    // LOGIC: Check config file first. If None, check Keyring.
    let password = match config.password {
        Some(p) => p, // Found in file (Insecure mode)
        None => {
            // Not in file, check Keyring (Secure mode)
            let entry = Entry::new(APP_NAME, &config.username).map_err(|e| e.to_string())?;
            entry
                .get_password()
                .map_err(|e| format!("Keyring Error for user '{}': {}", config.username, e))?
        }
    };

    Ok((config.username, password))
}

pub fn get_config_dir() -> Result<PathBuf, String> {
    let dirs = ProjectDirs::from("", "", APP_NAME).ok_or("Could not determine home directory")?;
    Ok(dirs.config_dir().to_path_buf())
}
