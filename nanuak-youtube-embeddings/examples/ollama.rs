use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;
use ollama_rs::Ollama;

#[tokio::main]
async fn main() {
    // https://github.com/pepperoni21/ollama-rs
    let ollama = Ollama::default();
    let request =
        GenerateEmbeddingsRequest::new("bge-m3:latest".to_string(), "Why is the sky blue?".into());
    let res = ollama.generate_embeddings(request).await.unwrap();
    println!("{:?}", res);
    println!("Embedding length: {}", res.embeddings[0].len());
}
