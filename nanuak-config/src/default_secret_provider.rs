use crate::env_secret_provider::EnvSecretProvider;
use crate::my_1password_secret_provider::My1PasswordSecretProvider;
use crate::secret_provider::SecretProvider;
use async_trait::async_trait;
use tracing::debug;
use crate::config_entry::ConfigField;

#[derive(Debug)]
pub struct DefaultSecretProvider;

#[async_trait]
impl SecretProvider for DefaultSecretProvider {
    async fn get<F: ConfigField>(&self) -> eyre::Result<Option<F::Value>> {
        let sp = EnvSecretProvider;
        debug!("Trying {:?} for {}", &sp, F::key());
        if let Ok(Some(value)) = sp.get::<F>().await {
            return Ok(Some(value));
        }

        let sp = My1PasswordSecretProvider;
        debug!("Trying {:?} for {}", &sp, F::key());
        if let Ok(Some(value)) = sp.get::<F>().await {
            return Ok(Some(value));
        }
        Ok(None)
    }
}
