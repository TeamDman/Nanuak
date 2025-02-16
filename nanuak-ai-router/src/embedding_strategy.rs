use crate::embedding_model::EmbeddingModel;
use crate::models::bge_m3_embedding_model::BgeM3EmbeddingModel;

#[derive(Debug)]
pub enum WellKnownEmbeddingStrategy {
    BestLocal,
    BestRemote,
}
impl WellKnownEmbeddingStrategy {
    pub fn get_model(&self) -> Box<dyn EmbeddingModel> {
        match self {
            WellKnownEmbeddingStrategy::BestLocal => Box::new(BgeM3EmbeddingModel),
            WellKnownEmbeddingStrategy::BestRemote => todo!(),
        }
    }
}
