use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("MacLarian error: {0}")]
    MacLarian(#[from] maclarian::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Workspace error: {0}")]
    Workspace(String),
    
    #[error("Index error: {0}")]
    Index(String),
}

pub type Result<T> = std::result::Result<T, Error>;
