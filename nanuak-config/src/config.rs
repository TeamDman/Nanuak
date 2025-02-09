use std::path::PathBuf;

use eyre::Context;
use eyre::bail;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tracing::debug;
use tracing::warn;

use crate::config_entry::ConfigField;
use crate::dirs::get_config_path;
use crate::secret_provider::DefaultSecretProvider;
use crate::secret_provider::SecretProvider;

#[derive(Debug)]
pub struct NanuakConfig<P: SecretProvider> {
    pub save_path: PathBuf,
    secret_provider: P,
    inner: toml::value::Table,
}

impl<P: SecretProvider> NanuakConfig<P> {
    /// Retrieves the configuration value for the given entry.
    pub async fn get<T: ConfigField>(&mut self) -> eyre::Result<Option<T::Value>>
    where
        T::Value: DeserializeOwned,
    {
        debug!("Getting config value for {}", T::key());

        // Try getting from the config
        if let Some(value) = self.inner.get(T::key()) {
            debug!("Found value in config: {:?}", value);
            let cast = T::Value::deserialize(value.clone()).wrap_err(format!(
                "Failed to deserialize configuration value for {}",
                T::key()
            ))?;
            return Ok(Some(cast));
        }

        // Try getting from environment variables
        debug!("Config value wasn't present for {}, trying secret provider", T::key());
        if let Some(value) = self.secret_provider.get::<T>()? {
            debug!("Config value supplied by secret provider {:?}: {}", &self.secret_provider, T::key());

            // update self
            self.set::<T>(&value).await?;

            // return the value
            return Ok(Some(value));
        } else {
            warn!("No value found for {} in config or secret provider", T::key());
        }

        Ok(None)
    }

    /// Sets the configuration value for the given entry.
    pub async fn set<T: ConfigField>(&mut self, value: &T::Value) -> eyre::Result<()>
    where
        T::Value: Serialize,
    {
        debug!("Setting config value for {}", T::key());
        let toml_val = toml::Value::try_from(value).wrap_err(format!(
            "Failed to convert value to toml::Value for {}",
            T::key()
        ))?;
        self.inner.insert(T::key().to_string(), toml_val);
        Ok(())
    }

    /// Persists the configuration to disk.
    pub async fn save(&self) -> eyre::Result<()> {
        debug!("Saving config to disk at {}", self.save_path.display());
        save_config(self).await?;
        Ok(())
    }
}
impl NanuakConfig<DefaultSecretProvider> {
    /// Loads the configuration from disk.
    pub async fn acquire() -> eyre::Result<Self> {
        get_config().await
    }
}

/// Call this once at startup:
/// - Finds your platform config dir (e.g. ~/.config/myorg/myapp/config.toml on Linux).
/// - Reads the config file if it exists; otherwise uses default.
pub async fn get_config() -> eyre::Result<NanuakConfig<DefaultSecretProvider>> {
    let save_path = get_config_path().await?;
    let inner = if save_path.exists() {
        // Load config from disk
        let mut file = OpenOptions::new()
            .read(true)
            .open(&save_path)
            .await
            .wrap_err_with(|| format!("Failed to open config file: {}", save_path.display()))?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .await
            .wrap_err("Failed to read config file")?;
        toml::from_str(&buf).wrap_err("Failed to parse TOML")?
    } else {
        // Use defaults if there's no file yet
        Default::default()
    };
    Ok(NanuakConfig {
        inner,
        save_path,
        secret_provider: DefaultSecretProvider,
    })
}

/// Call this after modifying the config to persist changes.
pub async fn save_config<P: SecretProvider>(config: &NanuakConfig<P>) -> eyre::Result<()> {
    let toml_str =
        toml::to_string_pretty(&config.inner).wrap_err("Failed to serialize config to TOML")?;
    let Some(parent) = &config.save_path.parent() else {
        bail!(
            "Config file path has no parent directory: {}",
            config.save_path.display()
        );
    };
    tokio::fs::create_dir_all(&parent).await?;
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true) // overwrite
        .open(&config.save_path)
        .await
        .wrap_err_with(|| {
            format!(
                "Failed to open/create config file: {}",
                config.save_path.display()
            )
        })?;

    file.write_all(toml_str.as_bytes())
        .await
        .wrap_err("Failed to write config to file")?;

    Ok(())
}
