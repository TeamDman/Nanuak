use crate::config_entry::ConfigField;
use async_trait::async_trait;
use toml::Value;
use toml::value::Table;

#[async_trait]
pub trait SecretProvider: std::fmt::Debug + Sized {
    /// Returns the provider name, which is used as the key for the provider’s metadata.
    fn provider_name(&self) -> &'static str;

    /// Attempt to retrieve the secret for the given configuration entry.
    ///
    /// The `entry` parameter is the full TOML table for the config key. It should contain a `"value"`
    /// field (if already set) and may also include sub‑tables for each secret provider.
    async fn get<F: ConfigField>(&self, entry: &mut Table) -> eyre::Result<Option<F::Value>>;

    /// Helper to extract (or create) the metadata table for this provider.
    fn get_metadata<'a>(&self, entry: &'a mut Table) -> &'a mut Table {
        // Use the provider name as the key.
        entry
            .entry(self.provider_name().to_string())
            .or_insert_with(|| Value::Table(Default::default()));
        entry
            .get_mut(self.provider_name())
            .and_then(Value::as_table_mut)
            .expect("We just inserted a table; qed")
    }
}
