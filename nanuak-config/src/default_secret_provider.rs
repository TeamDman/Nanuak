use crate::config_entry::ConfigField;
use crate::env_secret_provider::EnvSecretProvider;
use crate::my_1password_secret_provider::My1PasswordSecretProvider;
use crate::secret_provider::SecretProvider;
use async_trait::async_trait;
use eyre::Context;
use serde::Deserialize;
use toml::value::Table;

#[derive(Debug)]
pub struct DefaultSecretProvider;

#[async_trait]
impl SecretProvider for DefaultSecretProvider {
    fn provider_name(&self) -> &'static str {
        "default"
    }

    async fn get<F: ConfigField>(&self, entry: &mut Table) -> eyre::Result<Option<F::Value>> {
        // First, if the config entry already contains a "value", return it.
        if let Some(val) = entry.get("value") {
            let value = F::Value::deserialize(val.clone()).wrap_err(format!(
                "Failed to deserialize configuration value for {}",
                F::key()
            ))?;
            return Ok(Some(value));
        }

        // Chain: try environment provider.
        let env_provider = EnvSecretProvider;
        if let Some(value) = env_provider.get::<F>(entry).await? {
            return Ok(Some(value));
        }

        // Then, try the 1Password provider.
        let op_provider = My1PasswordSecretProvider;
        if let Some(value) = op_provider.get::<F>(entry).await? {
            return Ok(Some(value));
        }
        Ok(None)
    }
}
