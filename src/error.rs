use thiserror::Error;

#[derive(Debug, Error)]
pub enum WingetError {
    #[error("failed to run winget process: {0}")]
    Process(#[from] std::io::Error),

    #[error("winget output is not valid UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("failed to parse winget output: {0}")]
    Parse(#[from] ParseWingetError),
}

#[derive(Debug, Error)]
pub enum ParseWingetError {
    #[error("could not determine column positions from header line")]
    HeaderParseFailed,

    #[error("invalid package source: {0}")]
    InvalidSource(#[from] ParsePackageSourceError),
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("unexpected package source value: '{input}'")]
pub struct ParsePackageSourceError {
    pub input: String,
}
