#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SignatureStatus {
    Intact,
    Broken(String),
    Unsupported(String),
}
