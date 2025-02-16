use crate::modality::Modality;

pub trait EmbeddingSpace {
    fn get_modalities(&self) -> Vec<Modality>;
    fn get_dimensions(&self) -> Vec<u16>;
}
