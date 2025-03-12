pub struct Question {
    pub text: String,
    pub context: Vec<String>,
}

impl Question {
    pub fn new(question: String) -> Self {
        Question {
            text: question,
            context: Vec::new(),
        }
    }
    pub fn with_context(mut self, context: Vec<String>) -> Self {
        self.context = context;
        self
    }
}
