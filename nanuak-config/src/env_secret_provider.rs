use crate::config_entry::ConfigField;
use crate::secret_provider::SecretProvider;
use async_trait::async_trait;
use eyre::Context;
use serde::Deserialize;

#[derive(Debug)]
pub struct EnvSecretProvider;

#[async_trait]
impl SecretProvider for EnvSecretProvider {
    async fn get<F: ConfigField>(&self) -> eyre::Result<Option<F::Value>> {
        let Ok(env_val) = std::env::var(F::key()) else {
            return Ok(None);
        };
        let value = toml::Value::try_from(env_val)?;
        let cast = F::Value::deserialize(value).wrap_err(format!(
            "Failed to deserialize environment value for {}",
            F::key()
        ))?;
        return Ok(Some(cast));
    }
}
