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
}

impl Error {
    pub fn new(message: &str) -> Error {
        Self::Message(message.to_string())
    }
    pub fn from_error(e: impl std::error::Error) -> Error {
        Self::new(&e.to_string())
    }
}

