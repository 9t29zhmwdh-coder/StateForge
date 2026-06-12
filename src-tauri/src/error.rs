use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum SfError {
    #[error("Database: {0}")] Db(#[from] sqlx::Error),
    #[error("IO: {0}")]       Io(#[from] std::io::Error),
    #[error("{0}")]           Anyhow(#[from] anyhow::Error),
    #[error("Keyring: {0}")] Keyring(#[from] keyring::Error),
    #[error("{0}")]           Other(String),
}

impl Serialize for SfError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, SfError>;
