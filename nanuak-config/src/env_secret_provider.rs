use crate::config_entry::ConfigField;
use crate::secret_provider::SecretProvider;
use async_trait::async_trait;
use eyre::Context;
use serde::Deserialize;
use toml::Value;
use toml::value::Table;

#[derive(Debug)]
pub struct EnvSecretProvider;

#[async_trait]
impl SecretProvider for EnvSecretProvider {
    fn provider_name(&self) -> &'static str {
        "env"
    }

    async fn get<F: ConfigField>(&self, _entry: &mut Table) -> eyre::Result<Option<F::Value>> {
        // Simply try to load from the environment.
        if let Ok(env_val) = std::env::var(F::key()) {
            let value = Value::try_from(env_val)?;
            let cast = F::Value::deserialize(value).wrap_err(format!(
                "Failed to deserialize environment value for {}",
                F::key()
            ))?;
            return Ok(Some(cast));
        }
        Ok(None)
    }
}
