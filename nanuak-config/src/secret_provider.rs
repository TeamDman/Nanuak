use crate::config_entry::ConfigField;
use async_trait::async_trait;

#[async_trait]
pub trait SecretProvider: std::fmt::Debug + Sized {
    async fn get<F: ConfigField>(&self) -> eyre::Result<Option<F::Value>>;
}
