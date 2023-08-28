use std::{error::Error, fmt::Display};

pub mod wire_types;

#[derive(PartialEq, Debug)]
pub enum ProtoscopeRsError {
    VarintOverflow,
    DecodeOverflow,
    EncodeOverflow,
    BufferFull,
    LengthMismatch,
    UtfDecoding,
    InvalidWireType,
    Eof,
}

impl Display for ProtoscopeRsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtoscopeRsError::VarintOverflow => write!(f, "ProtoscopeRsError::VarintOverflow"),
            ProtoscopeRsError::DecodeOverflow => write!(f, "ProtoscopeRsError::DecodeOverflow"),
            ProtoscopeRsError::BufferFull => write!(f, "ProtoscopeRsError::BufferFull"),
            ProtoscopeRsError::LengthMismatch => write!(f, "ProtoscopeRsError::LengthMismatch"),
            ProtoscopeRsError::UtfDecoding => write!(f, "ProtoscopeRsError::UtfDecoding"),
            ProtoscopeRsError::InvalidWireType => write!(f, "ProtoscopeRsError::InvalidWireType"),
            ProtoscopeRsError::EncodeOverflow => write!(f, "ProtoscopeRsError::EncodeOverflow"),
            ProtoscopeRsError::Eof => write!(f, "ProtoscopeRsError::Eof"),
        }
    }
}

impl Error for ProtoscopeRsError {}

pub type ByteIterator<'a> = std::slice::Iter<'a, u8>;
pub type OutputByteIterator<'a> = std::slice::IterMut<'a, u8>;
pub type Result<T> = std::result::Result<T, ProtoscopeRsError>;
