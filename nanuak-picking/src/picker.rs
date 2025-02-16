use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use eyre::bail;
use nanuak_embedding::embedding::Embedding;
use nanuak_embedding::embedding_request::EmbeddingPayload;
use nanuak_embedding::embedding_strategy::WellKnownEmbeddingStrategy;

pub async fn pick<T>(_args: FzfArgs<T>) -> eyre::Result<T> {
    todo!()
}
pub async fn pick_many<T>(args: FzfArgs<Choice<T>>) -> eyre::Result<Vec<Choice<T>>> {
    let mut query = String::new();
    if let Some(header) = args.header {
        query.push_str(&header);
        query.push_str("\n");
    }
    if let Some(prompt) = args.prompt {
        query.push_str(&prompt);
        query.push_str("\n");
    }

    let mut to_embed = vec![EmbeddingPayload::Text(query.clone())];
    for choice in &args.choices {
        to_embed.push(EmbeddingPayload::Text(choice.key.clone()));
    }
    let embeddings =
        Embedding::try_generate(WellKnownEmbeddingStrategy::BestLocal, to_embed).await?;
    let ([query], choices_embeddings) = embeddings.split_at(1) else {
        bail!("Embedding shape was incorrect, got {}", embeddings.len());
    };
    let threshold = 0.65;
    let mut chosen = Vec::new();
    for (embedding, choice) in choices_embeddings.iter().zip(args.choices.into_iter()) {
        let similarity = query.cosine_similarity(embedding)?;
        println!("{}: {}", choice.key, similarity);
        if similarity >= threshold {
            chosen.push(choice);
        }
    }
    Ok(chosen)
}
