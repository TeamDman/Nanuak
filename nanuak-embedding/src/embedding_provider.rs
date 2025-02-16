use async_trait::async_trait;

use crate::attributes::Residency;
use crate::embedding::Embedding;
use crate::embedding_model::EmbeddingModel;
use crate::embedding_request::EmbeddingPayload;
use crate::model_attributes::ModelAttributes;

#[async_trait]
pub trait EmbeddingProvider {
    async fn is_supported(
        &self,
        model: &dyn EmbeddingModel,
    ) -> eyre::Result<bool>;
    async fn get_embeddings(
        &self,
        model: &dyn EmbeddingModel,
        payloads: Vec<EmbeddingPayload>,
    ) -> eyre::Result<Vec<Embedding>>;
    async fn get_attributes(
        &self,
        model: &dyn EmbeddingModel,
    ) -> eyre::Result<ModelAttributes>;
    fn get_residency(&self) -> Residency;
}
