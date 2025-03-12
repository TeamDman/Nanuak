use crate::attributes::ContextSize;

pub trait GenerativeTextModel: Send + Sync {
    fn name(&self) -> &'static str;
    fn get_context_size(&self) -> ContextSize;
}
