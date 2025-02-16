use crate::embedding_provider::EmbeddingProvider;
use crate::providers::ollama_embedding_provider::OllamaEmbeddingProvider;
use strum::VariantArray;

#[derive(Debug, VariantArray)]
#[non_exhaustive]
pub enum WellKnownEmbeddingProviders {
    Ollama,
}
impl WellKnownEmbeddingProviders {
    pub fn get(&self) -> impl EmbeddingProvider {
        match self {
            WellKnownEmbeddingProviders::Ollama => OllamaEmbeddingProvider,
        }
    }
}
