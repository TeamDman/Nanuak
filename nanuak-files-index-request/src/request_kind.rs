pub enum RequestKind {
    Embed,
    Caption
}
impl std::fmt::Display for RequestKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestKind::Embed => write!(f, "embed"),
            RequestKind::Caption => write!(f, "caption"),
        }
    }
}