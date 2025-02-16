use eyre::OptionExt;
use eyre::bail;
use strum::VariantArray;

use crate::attributes::Residency;
use crate::embedding_provider::EmbeddingProvider;
use crate::embedding_request::EmbeddingPayload;
use crate::embedding_strategy::WellKnownEmbeddingStrategy;
use crate::well_known_providers::WellKnownEmbeddingProviders;

pub struct Embedding(pub Vec<f32>);

impl Embedding {
    pub async fn try_generate(
        strategy: WellKnownEmbeddingStrategy,
        payloads: Vec<EmbeddingPayload>,
    ) -> eyre::Result<Vec<Self>> {
        let WellKnownEmbeddingStrategy::BestLocal = strategy else {
            bail!("Strategy not supported: {:?}", strategy);
        };
        let mut providers = WellKnownEmbeddingProviders::VARIANTS
            .iter()
            .map(|provider| provider.get())
            .filter(|provider| provider.get_residency() == Residency::Local);
        let Some(chosen_provider) = providers.next() else {
            bail!(
                "Failed to find suitable provider for strategy: {:?}",
                strategy
            );
        };
        let model = strategy.get_model();
        let expected = payloads.len();
        let embeddings = chosen_provider
            .get_embeddings(model.as_ref(), payloads)
            .await?;
        if embeddings.len() != expected {
            bail!("Expected {} embeddings, got {}", expected, embeddings.len());
        };
        Ok(embeddings)
    }
    pub fn cosine_similarity(&self, other: &Self) -> eyre::Result<f64> {
        use simsimd::SpatialSimilarity;
        f32::cosine(&self.0, &other.0).ok_or_eyre("Vectors must be of the same length")
    }
    pub fn sq_euclidean_distance(&self, other: &Self) -> eyre::Result<f64> {
        use simsimd::SpatialSimilarity;
        f32::sqeuclidean(&self.0, &other.0).ok_or_eyre("Vectors must be of the same length")
    }
}
