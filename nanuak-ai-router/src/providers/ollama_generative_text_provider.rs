use crate::answer::Answer;
use crate::attributes::Residency;
use crate::generative_text_model::GenerativeTextModel;
use crate::generative_text_provider::GenerativeTextProvider;
use crate::model_attributes::ModelAttributes;
use crate::question::Question;
use async_trait::async_trait;
use eyre::bail;
use ollama_rs::Ollama;
use ollama_rs::generation::chat::ChatMessage;
use ollama_rs::generation::chat::request::ChatMessageRequest;
use std::time::Instant;
use tracing::debug;

pub struct OllamaGenerativeTextProvider;
#[async_trait]
impl GenerativeTextProvider for OllamaGenerativeTextProvider {
    async fn is_supported(&self, model: &dyn GenerativeTextModel) -> eyre::Result<bool> {
        todo!()
    }
    async fn get_attributes(
        &self,
        model: &dyn GenerativeTextModel,
    ) -> eyre::Result<ModelAttributes> {
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
    async fn answer_question(
        &self,
        model: &dyn GenerativeTextModel,
        question: Question,
    ) -> eyre::Result<Answer> {
        let ollama = Ollama::default();
        let mut messages: Vec<ChatMessage> = Vec::new();
        let user_message = ChatMessage::user(self.format_question(question).await?);
        messages.push(user_message);
        let request = ChatMessageRequest::new(model.name().to_string(), messages);
        let start = Instant::now();
        let response = ollama.send_chat_messages(request).await?;
        let elapsed = start.elapsed();
        debug!(
            "Answering question with model {} took {:?}",
            model.name(),
            elapsed
        );
        let Some(answer) = response.message else {
            bail!("No answer found in response");
        };
        Ok(Answer::new(answer.content))
    }
    async fn format_question(&self, question: Question) -> eyre::Result<String> {
        let mut text = String::new();
        for (i, context) in question.context.iter().enumerate() {
            text.push_str(&format!("<context{}>\n{}\n</context{}>\n", i, context, i));
        }
        text.push_str(&format!("<question>\n{}\n</question>", question.text));
        Ok(text)
    }
}
#[cfg(test)]
mod test {
    use crate::generative_text_provider::GenerativeTextProvider;
    use crate::models::gemma2_2b_generative_text_model::Gemma2_2BGenerativeTextModel;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        let provider = super::OllamaGenerativeTextProvider;
        let model = Gemma2_2BGenerativeTextModel;
        let question = super::Question {
            context: vec![],
            text: "Why is the sky blue?".to_string(),
        };
        let answer = provider.answer_question(&model, question).await?;
        println!("Answer: {}", answer.body);
        let seeking = "rayleigh";
        assert!(answer.body.to_lowercase().contains(seeking));
        Ok(())
    }
}
