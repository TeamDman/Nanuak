use std::time::Instant;

use crate::attributes::Residency;
use crate::embedding::Embedding;
use crate::embedding_model::EmbeddingModel;
use crate::embedding_provider::EmbeddingProvider;
use crate::embedding_request::EmbeddingPayload;
use crate::model_attributes::ModelAttributes;
use async_trait::async_trait;
use ollama_rs::Ollama;
use ollama_rs::generation::embeddings::request::EmbeddingsInput;
use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;
use tracing::debug;

pub struct OllamaEmbeddingProvider;
#[async_trait]
impl EmbeddingProvider for OllamaEmbeddingProvider {
    async fn is_supported(&self, model: &dyn EmbeddingModel) -> eyre::Result<bool> {
        Ok(model.name() == "ollama")
    }
    async fn get_embeddings(
        &self,
        model: &dyn EmbeddingModel,
        payloads: Vec<EmbeddingPayload>,
    ) -> eyre::Result<Vec<Embedding>> {
        let mut rtn = Vec::with_capacity(payloads.len());
        let mut string_payloads: Vec<String> = Vec::new();
        for payload in payloads {
            match payload {
                EmbeddingPayload::Text(text) => {
                    string_payloads.push(text);
                }
                EmbeddingPayload::ImagePath(_image) => {
                    todo!()
                }
            }
        }

        let request = GenerateEmbeddingsRequest::new(
            model.name().to_string(),
            EmbeddingsInput::Multiple(string_payloads),
        );
        let start = Instant::now();
        let response = Ollama::default().generate_embeddings(request).await?;
        let elapsed = start.elapsed();
        debug!(
            "Embedding generation size {} with model {} took {:?}",
            response.embeddings.len(),
            model.name(),
            elapsed
        );
        for embedding in response.embeddings {
            rtn.push(Embedding(embedding));
        }
        Ok(rtn)
    }
    async fn get_attributes(&self, model: &dyn EmbeddingModel) -> eyre::Result<ModelAttributes> {
        Ok(ModelAttributes {
            vram_requirement: None,
            latency: None,
            accuracy: None,
            throughput: None,
            context_size: model.get_context_size(),
        })
    }
    fn get_residency(&self) -> Residency {
        Residency::Local
    }
}
