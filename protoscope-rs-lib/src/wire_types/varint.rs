use crate::{ProtoscopeRsError, OutputByteIterator, Result, ByteIterator};
use num_traits::NumCast;


const MAX_NUMBER_OF_BYTES: usize = ((std::mem::size_of::<u64>() * 8) + 7 - 1) / 7;

#[unroll::unroll_for_loops]
fn encode_varint_impl(value: u64,  iter:  &mut OutputByteIterator) -> Result<usize> {
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
    fn encode(&self, iter: &mut OutputByteIterator) -> Result<usize>;
}

macro_rules! expand_encode_trait_of_unsigned_types {
    ( $( $type:ty ),* ) => {
        $(
            impl EncodeVarint for $type {
                fn encode(&self, iter:  &mut OutputByteIterator) -> Result<usize> {
                    encode_varint_impl(*self as u64, iter)
                }
            }
        )*
    };
}

expand_encode_trait_of_unsigned_types![u8, u16, u32, u64];

impl EncodeVarint for bool {
    fn encode(&self, iter:  &mut OutputByteIterator) -> Result<usize> {
        if *self {
            1i32.encode(iter)
        }else{
            0i32.encode(iter)
        }
    }
}

#[unroll::unroll_for_loops]
fn decode_varint_impl(iter:  &mut ByteIterator) -> Result<u64> {
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
    Ok(decoded_value)
}

pub trait DecodeVarint {
    fn decode(iter:  &mut ByteIterator) -> Result<Self>
    where
        Self: Sized;
}

macro_rules! expand_decode_trait_of_unsigned_types {
    ( $( $type:ty ),* ) => {
        $(
            impl DecodeVarint for $type {
                fn decode(iter:  &mut ByteIterator) -> Result<Self> {
                    let u64_value = decode_varint_impl(iter)?;
                    let output  = <$type as NumCast>::from(u64_value);
                    if let Some(output) = output {
                        return Ok(output as $type);
                    }else{
                        return Err(ProtoscopeRsError::DecodeOverflow);
                    }
                }
            }
        )*
    };
}

expand_decode_trait_of_unsigned_types![u8, u16, u32, u64];

impl DecodeVarint for bool {
    fn decode(iter:  &mut ByteIterator) -> Result<Self>
    where
        Self: Sized {
        let i32_value = i32::decode(iter)?;
        if i32_value == 0 {
            return Ok(false);
        }else if i32_value == 1 {
            return Ok(true);
        }else{
            return Err(ProtoscopeRsError::DecodeOverflow);
        }
    }
}

fn zigzag_encode(input: i64) -> u64 {
    return ((input >> (64 - 1)) as u64 /*Arithmetic right shift here just propagates the sign-bit from the most significant bit to all the other bits */)
            ^  /* XOR */
            ((input << 1) as u64)/*Regular logical bitwise left-shit operation*/;
}

fn zigzag_decode(input: u64) -> i64 {
    (input >> 1) as i64 
    ^ /* XOR */ 
    -((input & 1) as i64) /*Extract the sign bit from the least-significant bit and propagate it to the rest of the bits*/
}

macro_rules! expand_encode_trait_of_signed_types {
    ( $( $type:ty ),* ) => {
        $(
            impl EncodeVarint for $type {
                fn encode(&self, iter:&mut OutputByteIterator) -> Result<usize> {
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
                fn decode(iter: &mut ByteIterator) -> Result<Self> {
                    let u64_value = decode_varint_impl(iter)?;
                    let i64_value = zigzag_decode(u64_value);
                    let output = <$type as NumCast>::from(i64_value);
                    if let Some(output) = output {
                        Ok(output)
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
        let value = decode_varint_impl(&mut [0x96u8, 0x01u8].iter());
        assert!(value.is_ok_and(|value| value == 150));

        let value = decode_varint_impl(&mut [0xc8u8, 0x03u8].iter());
        assert!(value.is_ok_and(|value| value == 456));

        let value = decode_varint_impl(&mut [0xacu8, 0x02u8].iter());
        assert!(value.is_ok_and(|value| value == 300));
    }

    #[test]
    fn test_extract_from_encoded_varint_multiple() {
        let mut iter = [0x96u8, 0x01u8, 0xc8u8, 0x03u8, 0xacu8, 0x02u8].iter();
        
        let value = decode_varint_impl(&mut iter);
        assert!(value.is_ok_and(|value| {
            value == 150
        }));
        

        let value = decode_varint_impl(&mut iter);
        assert!(value.is_ok_and(|value| {
            value == 456
        }));

        let value = decode_varint_impl(&mut iter);
        assert!(value.is_ok_and(|value| {
            value == 300
        }));
        let value = decode_varint_impl(&mut iter);
        assert!(value.is_err_and(|err| { err == ProtoscopeRsError::Eof }));
    }

    #[test]
    fn test_extract_from_encoded_varint_overflow() {
        let mut iter = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff].iter();
        let value = decode_varint_impl(&mut iter);
        assert!(value.is_err_and(|err| { err == ProtoscopeRsError::VarintOverflow }));
    }
    #[test]
    fn test_encode_varint() {
        let mut buffer: Vec<u8> = vec![0; 10];
        for value in (std::u64::MAX - 2000)..=std::u64::MAX {
            assert!(
                encode_varint_impl(value, &mut buffer.iter_mut()).is_ok_and(|num_bytes_encode| {
                    decode_varint_impl(&mut buffer[0..num_bytes_encode].iter())
                        .is_ok_and(|value| value == value)
                })
            );
        }
        for value in 0..2000 {
            assert!(
                encode_varint_impl(value, &mut buffer.iter_mut()).is_ok_and(|num_bytes_encode| {
                    decode_varint_impl(&mut buffer[0..num_bytes_encode].iter())
                        .is_ok_and(|value| value == value)
                })
            );
        }
    }

    #[test]
    fn test_encode_varint_buffer_full() {
        let mut buffer: Vec<u8> = vec![];
        assert!(encode_varint_impl(0, &mut buffer.iter_mut())
            .is_err_and(|err| err == ProtoscopeRsError::BufferFull));
    }

    #[test]
    fn test_unsiged_encode_decode_varint_trait_implementation() {
        let mut buffer: Vec<u8> = vec![0; 10];
        assert!((150u8)
            .encode(&mut buffer.iter_mut())
            .is_ok_and(|num_bytes_encode| {
                u8::decode(&mut buffer[0..num_bytes_encode].iter())
                    .is_ok_and(|value| value == 150)
            }));

        assert!((150u16)
            .encode(&mut buffer.iter_mut())
            .is_ok_and(|num_bytes_encode| {
                u16::decode(&mut buffer[0..num_bytes_encode].iter())
                    .is_ok_and(|value| value == 150)
            }));

        assert!((150u32)
            .encode(&mut buffer.iter_mut())
            .is_ok_and(|num_bytes_encode| {
                u32::decode(&mut buffer[0..num_bytes_encode].iter())
                    .is_ok_and(|value| value == 150)
            }));

        assert!((150u64)
            .encode(&mut buffer.iter_mut())
            .is_ok_and(|num_bytes_encode| {
                u64::decode(&mut buffer[0..num_bytes_encode].iter())
                    .is_ok_and(|value| value == 150)
            }));
    }

    #[test]
    fn test_decode_overflow() {
        let mut buffer: Vec<u8> = vec![0; 10];
        assert!((u64::MAX)
            .encode(&mut buffer.iter_mut())
            .is_ok_and(|num_bytes_encode| {
                u8::decode(&mut buffer[0..num_bytes_encode].iter())
                    .is_err_and(|err| err == ProtoscopeRsError::DecodeOverflow)
            }));

        assert!((u64::MAX)
            .encode(&mut buffer.iter_mut())
            .is_ok_and(|num_bytes_encode| {
                u16::decode(&mut buffer[0..num_bytes_encode].iter())
                    .is_err_and(|err| err == ProtoscopeRsError::DecodeOverflow)
            }));
    }

    #[test]
    fn test_zigzag() {
        assert!(zigzag_decode(zigzag_encode(10)) == 10);
        assert!(zigzag_decode(zigzag_encode(-10)) == -10);
    }

    #[test]
    fn test_signed_encode_decode_trait_implementation() {
        let mut buffer: Vec<u8> = vec![0; 10];

        for input in i64::MIN..(i64::MIN + 100) {
            assert!(input
                .encode(&mut buffer.iter_mut())
                .is_ok_and(|num_bytes_encoded| {
                    i64::decode(&mut buffer[0..num_bytes_encoded].iter())
                        .is_ok_and(|output| output == input)
                }));
        }

        for input in (i64::MAX - 100)..=(i64::MAX) {
            assert!(input
                .encode( &mut buffer.iter_mut())
                .is_ok_and(|num_bytes_encoded| {
                    i64::decode(&mut buffer[0..num_bytes_encoded].iter())
                        .is_ok_and(|output| output == input)
                }));
        }
    }

    #[test]
    fn test_signed_encode_decode_overflow() {
        let mut buffer: Vec<u8> = vec![0; 10];
        assert!((i16::MIN)
            .encode(&mut buffer.iter_mut())
            .is_ok_and(|num_bytes_encode| {
                i8::decode(&mut buffer[0..num_bytes_encode].iter())
                    .is_err_and(|err| err == ProtoscopeRsError::DecodeOverflow)
            }));
    }

    #[test]
    fn test_encode_decode_bool() {
        let mut buffer: Vec<u8> = vec![0; 10];
        assert!(true.encode(&mut buffer.iter_mut()).is_ok_and(|num_bytes_encoded|{
            bool::decode(&mut buffer[0..num_bytes_encoded].iter()).is_ok_and(|value|value)
        }));

        assert!(false.encode(&mut buffer.iter_mut()).is_ok_and(|num_bytes_encoded|{
            bool::decode(&mut buffer[0..num_bytes_encoded].iter()).is_ok_and(|value|value)
        }) == false);
    }

    #[test]
    fn test_encode_decode_bool_overflow() {
        let mut buffer: Vec<u8> = vec![0; 10];
        assert!(2i32.encode(&mut buffer.iter_mut()).is_ok_and(|num_bytes_encoded|{
            bool::decode(&mut buffer[0..num_bytes_encoded].iter()).is_err_and(|err|err == ProtoscopeRsError::DecodeOverflow)
        }));
    }
}
