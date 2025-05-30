use nanuak_ai_router::embedding::Embedding;
use nanuak_ai_router::embedding_request::EmbeddingPayload;
use nanuak_ai_router::embedding_strategy::WellKnownEmbeddingStrategy;

#[tokio::test]
async fn it_works() -> eyre::Result<()> {
    let x = "howdy";
    let embeddings = Embedding::try_generate(
        WellKnownEmbeddingStrategy::BestLocal,
        vec![EmbeddingPayload::Text(x.to_string())],
    )
    .await?;
    assert_eq!(embeddings.len(), 1);
    assert_eq!(
        vec![embeddings.first().unwrap().0.len() as u16],
        WellKnownEmbeddingStrategy::BestLocal
            .get_model()
            .get_embedding_space()
            .get_dimensions()
    );
    Ok(())
}
