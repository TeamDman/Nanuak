use serde::{de::DeserializeOwned, Serialize};

pub trait ConfigField {
    type Value: DeserializeOwned + Serialize;// + Clone + std::fmt::Debug;

    fn key() -> &'static str {
        std::any::type_name::<Self>()
    }
}