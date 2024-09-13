use thiserror::Error;

#[derive(Error, Debug)]
pub enum RustyChunkEncError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parsing error: {0}")]
    ParsingError(String),

    #[error("Incorrect index data")]
    IncorrectIndexData(),

    #[error("Invalid file name")]
    InvalidFileName(),
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for RustyChunkEncError {
    fn from(err: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        RustyChunkEncError::ParsingError(format!("Nom error: {:?}", err))
    }
}
