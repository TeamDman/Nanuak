use crate::attributes::ContextSize;
use crate::embedding_model::EmbeddingModel;
use crate::embedding_space::EmbeddingSpace;
use crate::modality::Modality;

pub struct BgeM3EmbeddingSpace;
impl EmbeddingSpace for BgeM3EmbeddingSpace {
    fn get_modalities(&self) -> Vec<Modality> {
        vec![Modality::Text]
    }

    fn get_dimensions(&self) -> Vec<u16> {
        vec![1024]
    }
}

pub struct BgeM3EmbeddingModel;
impl EmbeddingModel for BgeM3EmbeddingModel {
    fn get_embedding_space(&self) -> Box<dyn EmbeddingSpace> {
        Box::new(BgeM3EmbeddingSpace)
    }

    fn name(&self) -> &'static str {
        "bge-m3:latest"
    }
    
    fn get_context_size(&self) -> ContextSize {
        ContextSize(8192)
    }
}
