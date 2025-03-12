pub struct Answer {
    pub body: String,
}
impl Answer {
    pub fn new(body: String) -> Self {
        Answer {
            body,
        }
    }
}
