use crate::attributes::ContextSize;
use crate::generative_text_model::GenerativeTextModel;

pub struct Gemma2_2BGenerativeTextModel;
impl GenerativeTextModel for Gemma2_2BGenerativeTextModel {
    fn name(&self) -> &'static str {
        "gemma2:2b"
    }

    fn get_context_size(&self) -> ContextSize {
        ContextSize(8192)
    }
}
