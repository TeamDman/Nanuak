use crate::attributes::ContextSize;
use crate::embedding_space::EmbeddingSpace;

pub trait EmbeddingModel: Send + Sync {
    fn name(&self) -> &'static str;
    fn get_embedding_space(&self) -> Box<dyn EmbeddingSpace>;
    fn get_context_size(&self) -> ContextSize;
}
