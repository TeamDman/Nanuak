use itertools::Itertools;
use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;
use ollama_rs::Ollama;
use simsimd::SpatialSimilarity;

#[tokio::main]
async fn main() {
    // Initialize the Ollama client
    let ollama = Ollama::default();
    let model = "bge-m3:latest";

    // Define the duration times and words
    let duration_times = [
        "00:07",
        "01:00",
        "02:00",
        "03:00",
        "03:30",
        "04:00",
        "05:00",
        "1:23:00",
        "45:13",
        "27:25",
        "00:00:05",
        "1m3s",
        "27s",
        "50s",
        "5m",
        "5 minutes 7 seconds",
        "1 hours 4 minutes 21 seconds",
    ];
    let duration_words = [
        "short",
        "quick",
        "ten seconds",
        "one second",
        "one minute",
        "a few minutes",
        "seven minutes",
        "fifteen minutes",
        "ten+ minutes",
        "ten plus minutes",
        "over 10 minutes",
        "over ten minutes",
        "half an hour",
        "presentation length",
        "conference talk",
    ];

    // Generate embeddings for duration times
    let time_embeddings = ollama
        .generate_embeddings(GenerateEmbeddingsRequest::new(
            model.to_string(),
            duration_times.iter().cloned().collect_vec().into(),
        ))
        .await
        .expect("Failed to generate time embeddings");

    // Generate embeddings for duration words
    let word_embeddings = ollama
        .generate_embeddings(GenerateEmbeddingsRequest::new(
            model.to_string(),
            duration_words.iter().cloned().collect_vec().into(),
        ))
        .await
        .expect("Failed to generate word embeddings");

    // Ensure that the number of embeddings matches the number of inputs
    assert_eq!(
        time_embeddings.embeddings.len(),
        duration_times.len(),
        "Number of time embeddings does not match number of duration times"
    );
    assert_eq!(
        word_embeddings.embeddings.len(),
        duration_words.len(),
        "Number of word embeddings does not match number of duration words"
    );

    // Print the table header
    print!("{:<24}", ""); // Empty space for the row labels
    for &time in &duration_times {
        print!("{:<10}", time);
    }
    println!(); // Newline after header

    // Iterate over each word and its corresponding embedding
    for (word, word_embedding) in duration_words.iter().zip(word_embeddings.embeddings.iter()) {
        print!("{:<24}", word); // Print the word with fixed width

        // Iterate over each time and its corresponding embedding
        for time_embedding in &time_embeddings.embeddings {
            // Compute cosine similarity between word and time embeddings
            let similarity = f32::cosine(word_embedding, time_embedding).unwrap_or(0.0);
            print!("{:<10.2}", similarity); // Print similarity with 2 decimal places
        }
        println!(); // Newline after each row
    }
}
