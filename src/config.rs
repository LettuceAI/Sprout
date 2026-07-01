use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_require_auth")]
    pub require_auth: bool,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8477
}

fn default_require_auth() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            api_key: generate_api_key(),
            require_auth: default_require_auth(),
        }
    }
}

pub fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .map(|dir| dir.join("sprout").join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("config.toml"))
}

pub fn load_or_create(path: &Path) -> anyhow::Result<(Config, bool)> {
    if path.exists() {
        let text = fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&text)?;
        let mut generated = false;
        if config.api_key.trim().is_empty() {
            config.api_key = generate_api_key();
            generated = true;
            save(path, &config)?;
        }
        Ok((config, generated))
    } else {
        let config = Config::default();
        save(path, &config)?;
        Ok((config, true))
    }
}

fn save(path: &Path, config: &Config) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = toml::to_string_pretty(config)?;
    fs::write(path, &text)?;
    restrict_permissions(path)?;
    Ok(())
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}

pub fn generate_api_key() -> String {
    use rand::distr::Alphanumeric;
    use rand::Rng;
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}
