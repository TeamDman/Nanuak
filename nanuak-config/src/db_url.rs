use crate::config_entry::ConfigField;

pub struct DatabasePassword;
impl ConfigField for DatabasePassword {
    type Value = String;
    fn key() -> &'static str {
        "DATABASE_PASSWORD"
    }
}
