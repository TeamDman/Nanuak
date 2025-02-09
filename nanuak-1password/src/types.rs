use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Vault {
    pub id: String,
    pub name: String,
}
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Url {
    pub label: Option<String>,
    pub primary: Option<bool>,
    pub href: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub tags: Option<Vec<String>>,
    pub version: u32,
    pub vault: Vault,
    pub category: String,
    pub last_edited_by: String,
    pub created_at: String,
    pub updated_at: String,
    pub additional_information: Option<String>,
    pub urls: Option<Vec<Url>>,
    pub fields: Option<Vec<Field>>,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Field {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub purpose: String,
    pub label: String,
    pub value: Option<String>,
    pub reference: String,
    pub entropy: Option<f64>,
    pub password_details: Option<PasswordDetails>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct PasswordDetails {
    pub entropy: u32,
    pub generated: bool,
    pub strength: String,
}
