use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait ConfigField {
    type Value: DeserializeOwned + Serialize; // + Clone + std::fmt::Debug;

    fn key() -> &'static str {
        std::any::type_name::<Self>()
    }
}
