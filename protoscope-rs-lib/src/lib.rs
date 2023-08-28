pub mod wire_types;

#[derive(PartialEq, Debug)]
pub enum ProtoscopeRsError {
    VarintOverflow,
    DecodeOverflow,
    BufferFull,
    LengthMismatch,
    UtfDecoding,
    InvalidWireType,
    Eof,
}

pub type ByteIterator<'a> = std::slice::Iter<'a, u8>;
pub type OutputByteIterator<'a> = std::slice::IterMut<'a, u8>;
pub type Result<T> = std::result::Result<T, ProtoscopeRsError>;
