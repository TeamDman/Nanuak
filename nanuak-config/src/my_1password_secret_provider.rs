use crate::secret_provider::SecretProvider;

#[derive(Debug)]
pub struct My1PasswordSecretProvider;
impl SecretProvider for My1PasswordSecretProvider {
    fn get<F: crate::config_entry::ConfigField>(&self) -> eyre::Result<Option<F::Value>> {
        todo!()
    }
}