use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("MacLarian error: {0}")]
    MacLarian(#[from] maclarian::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Workbench error: {0}")]
    Workbench(String),

    #[error("Index error: {0}")]
    Index(String),
}

pub type Result<T> = std::result::Result<T, Error>;
