use num_traits::NumCast;

type ByteIterator<'a> = std::slice::Iter<'a, u8>;
type OutputByteIterator<'a> = std::slice::IterMut<'a, u8>;

#[derive(PartialEq, Debug)]
pub enum ProtoscopeRsError {
    VarintOverflow,
    DecodeOverflow,
    BufferFull,
    Eof,
}

pub type Result<T> = std::result::Result<T, ProtoscopeRsError>;

const MAX_NUMBER_OF_BYTES: usize = ((std::mem::size_of::<u64>() * 8) + 7 - 1) / 7;

#[unroll::unroll_for_loops]
pub fn encode_varint_impl(value: u64, mut iter: OutputByteIterator) -> Result<usize> {
    let mut value_copy = value;
    let mut bytes_encoded = 0;
    for _ in 0..MAX_NUMBER_OF_BYTES {
        let output_byte = match iter.next() {
            Some(byte) => byte,
            None => return Err(ProtoscopeRsError::BufferFull),
        };
        bytes_encoded += 1;
        if value_copy & !0x7f == 0 {
            // No more upper bits set
            *output_byte = (value_copy & 0x7f) as u8; // Extract payload and append to output byte
            return Ok(bytes_encoded);
        }
        *output_byte = ((value_copy & 0x7f) as u8) | 0x80; // Extract payload and append to output byte and also set the continue bit
        value_copy = value_copy >> 7;
    }
    Ok(bytes_encoded)
}

pub trait EncodeVarint {
    fn encode_varint(&self, iter: OutputByteIterator) -> Result<usize>;
}

macro_rules! expand_encode_trait_of_unsigned_types {
    ( $( $type:ty ),* ) => {
        $(
            impl EncodeVarint for $type {
                fn encode_varint(&self, iter: OutputByteIterator) -> Result<usize> {
                    encode_varint_impl(*self as u64, iter)
                }
            }
        )*
    };
}

expand_encode_trait_of_unsigned_types![u8, u16, u32, u64];

#[unroll::unroll_for_loops]
fn decode_varint_impl(mut iter: ByteIterator) -> Result<(u64, ByteIterator)> {
    match iter.clone().peekable().peek() {
        None => return Err(ProtoscopeRsError::Eof),
        _ => {}
    }
    let mut decoded_value: u64 = 0;
    for byte_idx in 0..MAX_NUMBER_OF_BYTES {
        match &iter.next() {
            Some(byte) => {
                let payload = 0x7f & *byte;
                decoded_value = (decoded_value) | ((payload as u64) << (7 * byte_idx));
                if 0x80 & *byte == 0 {
                    break;
                } else if byte_idx == (MAX_NUMBER_OF_BYTES - 1) {
                    return Err(ProtoscopeRsError::VarintOverflow);
                }
            }
            None => {
                break;
            }
        }
    }
    Ok((decoded_value, iter.clone()))
}

pub trait DecodeVarint {
    fn decode_varint(iter: ByteIterator) -> Result<(Self, ByteIterator)>
    where
        Self: Sized;
}

macro_rules! expand_decode_trait_of_unsigned_types {
    ( $( $type:ty ),* ) => {
        $(
            impl DecodeVarint for $type {
                fn decode_varint(iter: ByteIterator) -> Result<(Self, ByteIterator)> {
                    let (u64_value, it) = decode_varint_impl(iter)?;
                    let output  = <$type as NumCast>::from(u64_value);
                    if let Some(output) = output {
                        return Ok((output as $type, it));
                    }else{
                        return Err(ProtoscopeRsError::DecodeOverflow);
                    }
                }
            }
        )*
    };
}

expand_decode_trait_of_unsigned_types![u8, u16, u32, u64];

fn zigzag_encode(input: i64) -> u64 {
    if input < 0 {
        (input.abs() * 2 - 1) as u64
    } else {
        (input * 2) as u64
    }
}

fn zigzag_decode(input: u64) -> i64 {
    if input % 2 == 0 {
        (input / 2) as i64
    } else {
        -(((input + 1) / 2) as i64)
    }
}

macro_rules! expand_encode_trait_of_signed_types {
    ( $( $type:ty ),* ) => {
        $(
            impl EncodeVarint for $type {
                fn encode_varint(&self, iter: OutputByteIterator) -> Result<usize> {
                    encode_varint_impl(zigzag_encode(*self as i64), iter)
                }
            }
        )*
    };
}

expand_encode_trait_of_signed_types![i8, i16, i32, i64];

macro_rules! expand_decode_trait_of_signed_types {
    ( $( $type:ty ),* ) => {
        $(
            impl DecodeVarint for $type {
                fn decode_varint(iter: ByteIterator) -> Result<(Self, ByteIterator)> {
                    let (u64_value, it) = decode_varint_impl(iter)?;
                    let i64_value = zigzag_decode(u64_value);
                    let output = <$type as NumCast>::from(i64_value);
                    if let Some(output) = output {
                        Ok((output, it))
                    }else{
                        return Err(ProtoscopeRsError::DecodeOverflow);
                    }
                }
            }
        )*
    };
}

expand_decode_trait_of_signed_types![i8, i16, i32, i64];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_from_encoded_varint() {
        let value = decode_varint_impl([0x96u8, 0x01u8].iter());
        assert!(value.is_ok_and(|(value, _)| value == 150));

        let value = decode_varint_impl([0xc8u8, 0x03u8].iter());
        assert!(value.is_ok_and(|(value, _)| value == 456));

        let value = decode_varint_impl([0xacu8, 0x02u8].iter());
        assert!(value.is_ok_and(|(value, _)| value == 300));
    }

    #[test]
    fn test_extract_from_encoded_varint_multiple() {
        let mut iter = [0x96u8, 0x01u8, 0xc8u8, 0x03u8, 0xacu8, 0x02u8].iter();
        let value = decode_varint_impl(iter.clone());
        assert!(value.is_ok_and(|(value, it)| {
            iter = it;
            value == 150
        }));

        let value = decode_varint_impl(iter.clone());
        assert!(value.is_ok_and(|(value, it)| {
            iter = it;
            value == 456
        }));

        let value = decode_varint_impl(iter.clone());
        assert!(value.is_ok_and(|(value, it)| {
            iter = it;
            value == 300
        }));
        let value = decode_varint_impl(iter.clone());
        assert!(value.is_err_and(|err| { err == ProtoscopeRsError::Eof }));
    }

    #[test]
    fn test_extract_from_encoded_varint_overflow() {
        let iter = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff].iter();
        let value = decode_varint_impl(iter.clone());
        assert!(value.is_err_and(|err| { err == ProtoscopeRsError::VarintOverflow }));
    }
    #[test]
    fn test_encode_varint() {
        let mut buffer: Vec<u8> = vec![0; 10];
        for value in (std::u64::MAX - 2000)..=std::u64::MAX {
            assert!(
                encode_varint_impl(value, buffer.iter_mut()).is_ok_and(|num_bytes_encode| {
                    decode_varint_impl(buffer[0..num_bytes_encode].iter())
                        .is_ok_and(|(value, _)| value == value)
                })
            );
        }
        for value in 0..2000 {
            assert!(
                encode_varint_impl(value, buffer.iter_mut()).is_ok_and(|num_bytes_encode| {
                    decode_varint_impl(buffer[0..num_bytes_encode].iter())
                        .is_ok_and(|(value, _)| value == value)
                })
            );
        }
    }

    #[test]
    fn test_encode_varint_buffer_full() {
        let mut buffer: Vec<u8> = vec![];
        assert!(encode_varint_impl(0, buffer.iter_mut())
            .is_err_and(|err| err == ProtoscopeRsError::BufferFull));
    }

    #[test]
    fn test_unsiged_encode_decode_varint_trait_implementation() {
        let mut buffer: Vec<u8> = vec![0; 10];
        assert!((150u8)
            .encode_varint(buffer.iter_mut())
            .is_ok_and(|num_bytes_encode| {
                u8::decode_varint(buffer[0..num_bytes_encode].iter())
                    .is_ok_and(|(value, _)| value == 150)
            }));

        assert!((150u16)
            .encode_varint(buffer.iter_mut())
            .is_ok_and(|num_bytes_encode| {
                u16::decode_varint(buffer[0..num_bytes_encode].iter())
                    .is_ok_and(|(value, _)| value == 150)
            }));

        assert!((150u32)
            .encode_varint(buffer.iter_mut())
            .is_ok_and(|num_bytes_encode| {
                u32::decode_varint(buffer[0..num_bytes_encode].iter())
                    .is_ok_and(|(value, _)| value == 150)
            }));

        assert!((150u64)
            .encode_varint(buffer.iter_mut())
            .is_ok_and(|num_bytes_encode| {
                u64::decode_varint(buffer[0..num_bytes_encode].iter())
                    .is_ok_and(|(value, _)| value == 150)
            }));
    }

    #[test]
    fn test_decode_overflow() {
        let mut buffer: Vec<u8> = vec![0; 10];
        assert!((u64::MAX)
            .encode_varint(buffer.iter_mut())
            .is_ok_and(|num_bytes_encode| {
                u8::decode_varint(buffer[0..num_bytes_encode].iter())
                    .is_err_and(|err| err == ProtoscopeRsError::DecodeOverflow)
            }));

        assert!((u64::MAX)
            .encode_varint(buffer.iter_mut())
            .is_ok_and(|num_bytes_encode| {
                u16::decode_varint(buffer[0..num_bytes_encode].iter())
                    .is_err_and(|err| err == ProtoscopeRsError::DecodeOverflow)
            }));
    }

    #[test]
    fn test_zigzag() {
        assert!(zigzag_decode(zigzag_encode(10)) == 10);
        assert!(zigzag_decode(zigzag_encode(-10)) == -10);
    }

    #[test]
    fn test_siged_encode_decode_varint_trait_implementation() {
        let mut buffer: Vec<u8> = vec![0; 10];
        assert!((-126i8)
            .encode_varint(buffer.iter_mut())
            .is_ok_and(|num_bytes_encode| {
                i8::decode_varint(buffer[0..num_bytes_encode].iter())
                    .is_ok_and(|(value, _)| value == -126i8)
            }));
    }

    #[test]
    fn test_siged_encode_decode_overflow() {
        let mut buffer: Vec<u8> = vec![0; 10];
        assert!((i16::MIN)
            .encode_varint(buffer.iter_mut())
            .is_ok_and(|num_bytes_encode| {
                i8::decode_varint(buffer[0..num_bytes_encode].iter())
                    .is_err_and(|err| err == ProtoscopeRsError::DecodeOverflow)
            }));
    }
}
