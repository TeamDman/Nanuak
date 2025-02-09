use crate::config_entry::ConfigField;

pub struct DatabaseUrl;
impl ConfigField for DatabaseUrl {
    type Value = String;
    fn key() -> &'static str {
        "DATABASE_URL"
    }
}
