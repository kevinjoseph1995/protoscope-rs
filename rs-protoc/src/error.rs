use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum RsProtocError {
    FilesystemError(String),
    LexError(String),
    ParseError(String),
}

impl Display for RsProtocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RsProtocError::")?;
        match self {
            RsProtocError::FilesystemError(error_message) => {
                write!(f, "FilesystemError[{}]", error_message)
            }
            RsProtocError::LexError(error_message) => {
                write!(f, "LexError[{}]", error_message)
            }
            RsProtocError::ParseError(error_message) => {
                write!(f, "ParseError[{}]", error_message)
            }
        }
    }
}

impl Error for RsProtocError {}

pub type Result<T> = std::result::Result<T, RsProtocError>;
