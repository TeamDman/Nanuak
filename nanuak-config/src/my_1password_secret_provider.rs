use async_trait::async_trait;
use nanuak_1password::pick_secret::pick_secret;

use crate::config_entry::ConfigField;
use crate::secret_provider::SecretProvider;
use eyre::Context;
use serde::Deserialize;

#[derive(Debug)]
pub struct My1PasswordSecretProvider;

#[async_trait]
impl SecretProvider for My1PasswordSecretProvider {
    async fn get<F: ConfigField>(&self) -> eyre::Result<Option<F::Value>> {
        let field = pick_secret().await?;
        let Some(value) = field.value else {
            return Ok(None);
        };
        let value = toml::Value::try_from(value)?;
        let cast = F::Value::deserialize(value).wrap_err(format!(
            "Failed to deserialize 1Password value for {}",
            F::key()
        ))?;
        Ok(Some(cast))
    }
}
