use eyre::OptionExt;
use itertools::Itertools;
use nanuak_embedding::embedding::Embedding;
use nanuak_embedding::embedding_request::EmbeddingPayload;
use nanuak_embedding::embedding_strategy::WellKnownEmbeddingStrategy;
use tracing::info;
use strum::VariantNames;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[derive(Debug, VariantNames, Hash, Eq, PartialEq, Clone, Copy)]
#[allow(dead_code)]
enum Word {
    Dog,
    Cat,
    Spider,
    Wolf,
    Human,
    Elephant,
    Snake,
    Whale,
    House,
    Car,
    Airplane,
    Tractor,
    Boat,
    Refrigerator,
    Toaster,
}

#[tokio::test]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env_lossy(),
        )
        .without_time()
        .init();

    let words = Word::VARIANTS;
    let word_embedding_request = words
        .iter()
        .map(|word| EmbeddingPayload::Text(word.to_string()));
    let query = "animal";
    let query_embedding_request = std::iter::once(EmbeddingPayload::Text(query.to_string()));
    let embeddings = Embedding::try_generate(
        WellKnownEmbeddingStrategy::BestLocal,
        query_embedding_request
            .chain(word_embedding_request)
            .collect_vec(),
    )
    .await?;
    let (query_embedding, word_embeddings) = embeddings
        .split_first()
        .ok_or_eyre("bad number of response items")?;
    let mut similarities = Vec::new();
    for embedding in word_embeddings {
        let similarity = query_embedding.cosine_similarity(embedding)?;
        similarities.push((similarity, embedding));
    }
    let sorted_words = words
        .iter()
        .zip(similarities.into_iter())
        .sorted_by(|(_, (a, _)), (_, (b, _))| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(word, (dist, embedding))| (word, dist))
        .collect_vec();
    info!("Query: {query}");
    for (word, dist) in sorted_words {
        info!("{}: {}", word, dist);
    }

    Ok(())
}
