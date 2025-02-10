use crate::config_entry::ConfigField;

pub struct DatabasePassword;
impl ConfigField for DatabasePassword {
    type Value = String;
    fn key() -> &'static str {
        "DATABASE_PASSWORD"
    }
}
impl DatabasePassword {
    pub fn format_url(password: &str) -> String {
        format!("postgres://postgres:{}@localhost/nanuak", password)
    }
}
