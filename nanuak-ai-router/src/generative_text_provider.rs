use crate::answer::Answer;
use crate::attributes::Residency;
use crate::generative_text_model::GenerativeTextModel;
use crate::model_attributes::ModelAttributes;
use crate::question::Question;
use async_trait::async_trait;

#[async_trait]
pub trait GenerativeTextProvider {
    async fn is_supported(&self, model: &dyn GenerativeTextModel) -> eyre::Result<bool>;
    async fn get_attributes(
        &self,
        model: &dyn GenerativeTextModel,
    ) -> eyre::Result<ModelAttributes>;
    fn get_residency(&self) -> Residency;
    async fn answer_question(
        &self, 
        model: &dyn GenerativeTextModel,
        question: Question
    ) -> eyre::Result<Answer>;
    async fn format_question(
        &self,
        question: Question,
    ) -> eyre::Result<String>;
}
