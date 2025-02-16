use std::path::PathBuf;

use crate::modality::Modality;

pub enum EmbeddingPayload {
    Text(String),
    ImagePath(PathBuf),
}
impl EmbeddingPayload {
    pub fn modality(&self) -> Modality {
        match self {
            EmbeddingPayload::Text(_) => Modality::Text,
            EmbeddingPayload::ImagePath(_) => Modality::Image,
        }
    }
}
