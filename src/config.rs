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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_cert_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_key_path: Option<PathBuf>,
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
            tls_cert_path: None,
            tls_key_path: None,
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
        config.validate()?;
        Ok((config, generated))
    } else {
        let config = Config::default();
        save(path, &config)?;
        config.validate()?;
        Ok((config, true))
    }
}

impl Config {
    pub fn tls_paths(&self, config_path: &Path) -> anyhow::Result<Option<(PathBuf, PathBuf)>> {
        match (&self.tls_cert_path, &self.tls_key_path) {
            (None, None) => Ok(None),
            (Some(cert), Some(key))
                if !cert.as_os_str().is_empty() && !key.as_os_str().is_empty() =>
            {
                let base = config_path.parent().unwrap_or_else(|| Path::new("."));
                let resolve = |path: &Path| {
                    if path.is_absolute() {
                        path.to_path_buf()
                    } else {
                        base.join(path)
                    }
                };
                Ok(Some((resolve(cert), resolve(key))))
            }
            _ => anyhow::bail!("tls_cert_path and tls_key_path must both be configured"),
        }
    }

    fn validate(&self) -> anyhow::Result<()> {
        match (&self.tls_cert_path, &self.tls_key_path) {
            (None, None) => Ok(()),
            (Some(cert), Some(key))
                if !cert.as_os_str().is_empty() && !key.as_os_str().is_empty() =>
            {
                Ok(())
            }
            _ => anyhow::bail!("tls_cert_path and tls_key_path must both be configured"),
        }
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
    use rand::Rng;
    use rand::distr::Alphanumeric;
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_tls_certificate_and_key_together() {
        let config = Config {
            tls_cert_path: Some(PathBuf::from("certs/server.crt")),
            tls_key_path: Some(PathBuf::from("certs/server.key")),
            ..Config::default()
        };

        let paths = config
            .tls_paths(Path::new("/etc/sprout/config.toml"))
            .unwrap()
            .unwrap();
        assert_eq!(paths.0, PathBuf::from("/etc/sprout/certs/server.crt"));
        assert_eq!(paths.1, PathBuf::from("/etc/sprout/certs/server.key"));
    }

    #[test]
    fn rejects_incomplete_tls_configuration() {
        let config = Config {
            tls_cert_path: Some(PathBuf::from("server.crt")),
            ..Config::default()
        };

        assert!(config.tls_paths(Path::new("config.toml")).is_err());
    }
}
