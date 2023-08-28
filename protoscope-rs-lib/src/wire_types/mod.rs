use crate::{ByteIterator, OutputByteIterator, ProtoscopeRsError, Result};

pub mod length_delimited;
pub mod non_varint;
pub mod varint;

#[derive(Debug, PartialEq, Clone)]
pub enum WireTypeEnum {
    Varint,
    I64,
    Len,
    I32,
}

#[derive(Debug)]
pub struct Tag {
    pub field_number: u64,
    pub wire_type: WireTypeEnum,
}

impl From<WireTypeEnum> for u64 {
    fn from(value: WireTypeEnum) -> Self {
        match value {
            WireTypeEnum::Varint => 0,
            WireTypeEnum::I64 => 1,
            WireTypeEnum::Len => 2,
            WireTypeEnum::I32 => 5,
        }
    }
}

impl TryFrom<u64> for WireTypeEnum {
    type Error = ProtoscopeRsError;
    fn try_from(value: u64) -> std::result::Result<WireTypeEnum, ProtoscopeRsError> {
        match value {
            0 => Ok(WireTypeEnum::Varint),
            1 => Ok(WireTypeEnum::I64),
            2 => Ok(WireTypeEnum::Len),
            5 => Ok(WireTypeEnum::I32),
            _ => Err(ProtoscopeRsError::InvalidWireType),
        }
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for i32 {}
    impl Sealed for i64 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
    impl Sealed for bool {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
    impl Sealed for String {}
}

pub trait Encode<'a>: Sized + private::Sealed {
    fn encode(&'a self, iter: &mut OutputByteIterator) -> Result<usize>;
}

pub trait Decode: Sized + private::Sealed {
    fn decode(iter: &mut ByteIterator) -> Result<Self>;
}

pub fn encode_tag(tag: &Tag, iter: &mut OutputByteIterator) -> Result<usize> {
    let tag_repr: u64 = (tag.field_number << 3) & u64::from(tag.wire_type.clone());
    tag_repr.encode(iter)
}

pub fn decode_tag(iter: &mut crate::ByteIterator) -> crate::Result<Tag> {
    let tag_u64 = u64::decode(iter)?;
    let wire_type = WireTypeEnum::try_from(tag_u64 & 0b111)?;
    let field_number = tag_u64 >> 3;
    Ok(Tag {
        field_number,
        wire_type,
    })
}

mod tests {
    use super::*;

    #[test]
    fn test_decode_tag() {
        let encoded_bytes: Vec<u8> = vec![0x08, 0x96, 0x01];
        let mut iter = encoded_bytes.iter();
        let tag = decode_tag(&mut iter);
        assert!(tag.is_ok());
        assert!(tag.unwrap().wire_type == WireTypeEnum::Varint);
        let payload = u64::decode(&mut iter);
        assert!(payload.is_ok_and(|payload| payload == 150));
    }
}
