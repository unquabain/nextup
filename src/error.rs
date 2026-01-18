use thiserror::Error as ThisError;
#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Error: {0}")]
    Message(String),

    #[error("Choice already set")]
    ChoiceAlreadySet,

     #[error("No choice made")]
    NoChoiceMade,

    #[error("No secret entered")]
    NoSecretEntered,

    #[error("UI Error: {0}")]
    UiError(#[from] raccacoonie::error::Error),

    #[error("Database Error: {0}")]
    DbError(#[from] tokio_postgres::Error),

    #[error("TLS Error: {0}")]
    TlsError(#[from] native_tls::Error),

    #[error("Could not prepare {0} query: {1}")]
    PrepareQueryError(&'static str, tokio_postgres::Error),
}

impl Error {
    pub fn new(message: &str) -> Error {
        Self::Message(message.to_string())
    }
    pub fn from_error(e: impl std::error::Error) -> Error {
        Self::new(&e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
