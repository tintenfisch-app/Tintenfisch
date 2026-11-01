#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityStatus {
    Expired(String),
    NotYetValid(String),
    Unverified(String),
}
