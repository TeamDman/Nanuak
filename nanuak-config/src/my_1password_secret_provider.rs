use crate::config_entry::ConfigField;
use crate::secret_provider::SecretProvider;
use async_trait::async_trait;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use cloud_terrastodon_core_user_input::prelude::pick;
use eyre::Context;
use itertools::Itertools;
use serde::Deserialize;
use strum::VariantArray;
use toml::Value;
use toml::value::Table;

#[derive(Debug)]
pub struct My1PasswordSecretProvider;

#[async_trait]
impl SecretProvider for My1PasswordSecretProvider {
    fn provider_name(&self) -> &'static str {
        "onepassword"
    }

    async fn get<F: ConfigField>(&self, entry: &mut Table) -> eyre::Result<Option<F::Value>> {
        // Use the helper to get this provider’s metadata block.
        let meta = self.get_metadata(entry);

        // If a reference is present, use it.
        if let Some(Value::String(reference)) = meta.get("reference") {
            // Read the secret using the reference.
            let secret_str = nanuak_1password::op_read::op_read(reference).await?;
            let value = Value::String(secret_str);
            let cast = F::Value::deserialize(value).wrap_err(format!(
                "Failed to deserialize 1Password secret for {}",
                F::key()
            ))?;
            return Ok(Some(cast));
        }

        #[derive(VariantArray)]
        enum Action {
            DontPick,
            Pick,
        }
        impl std::fmt::Display for Action {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Action::DontPick => write!(f, "Don't pick, skip 1Password provider"),
                    Action::Pick => write!(f, "Pick a secret from 1Password"),
                }
            }
        }
        let chosen = pick(FzfArgs {
            choices: Action::VARIANTS.iter().collect_vec(),
            prompt: None,
            header: None,
        })?;
        match chosen {
            Action::DontPick => return Ok(None),
            Action::Pick => {}
        }

        // Otherwise, prompt the user to pick a secret.
        let field = nanuak_1password::pick_secret::pick_secret().await?;
        let secret_value = match field.value {
            Some(val) => val,
            None => return Ok(None),
        };

        // Store the picked reference into metadata.
        meta.insert("reference".to_string(), Value::String(field.reference));

        let value = Value::try_from(secret_value)?;
        let cast = F::Value::deserialize(value).wrap_err(format!(
            "Failed to deserialize 1Password value for {}",
            F::key()
        ))?;
        Ok(Some(cast))
    }
}
