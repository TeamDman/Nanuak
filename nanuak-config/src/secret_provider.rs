use tracing::debug;

use crate::{config_entry::ConfigField, env_secret_provider::EnvSecretProvider, my_1password_secret_provider::My1PasswordSecretProvider};

pub trait SecretProvider: std::fmt::Debug + Sized {
    fn get<F: ConfigField>(&self) -> eyre::Result<Option<F::Value>>;
}

#[derive(Debug)]
pub struct DefaultSecretProvider;

impl SecretProvider for DefaultSecretProvider {
    fn get<F: ConfigField>(&self) -> eyre::Result<Option<F::Value>> {
        let sp = EnvSecretProvider;
        debug!("Trying {:?} for {}", &sp, F::key());
        if let Ok(Some(x)) = sp.get::<F>() {
            return Ok(Some(x));
        }

        let sp = My1PasswordSecretProvider;
        debug!("Trying {:?} for {}", &sp, F::key());
        if let Ok(Some(x)) = sp.get::<F>() {
            return Ok(Some(x));
        }
        Ok(None)
    }
}