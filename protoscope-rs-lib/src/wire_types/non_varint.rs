use crate::{ByteIterator, OutputByteIterator, ProtoscopeRsError, Result};

fn encode_internal<const N: usize>(
    encoded_bytes: [u8; N],
    mut iter: OutputByteIterator,
) -> Result<usize> {
    for byte in encoded_bytes {
        match iter.next() {
            Some(output_byte) => {
                *output_byte = byte;
            }
            None => return Err(ProtoscopeRsError::BufferFull),
        }
    }
    Ok(N)
}

trait EncodeI64: Sized {
    fn encode(&self, iter: OutputByteIterator) -> Result<usize> {
        encode_internal(self.get_little_endian_byte_representation(), iter)
    }
    fn get_little_endian_byte_representation(&self) -> [u8; 8];
}

trait EncodeI32: Sized {
    fn encode(&self, iter: OutputByteIterator) -> Result<usize> {
        encode_internal(self.get_little_endian_byte_representation(), iter)
    }
    fn get_little_endian_byte_representation(&self) -> [u8; 4];
}

impl EncodeI64 for f64 {
    fn get_little_endian_byte_representation(&self) -> [u8; 8] {
        self.to_le_bytes()
    }
}
impl EncodeI32 for f32 {
    fn get_little_endian_byte_representation(&self) -> [u8; 4] {
        self.to_le_bytes()
    }
}

trait DecodeI64 {
    fn decode(mut iter: ByteIterator) -> Result<(Self, ByteIterator)>
    where
        Self: Sized,
    {
        let mut raw_bytes = [0u8; 8];
        for output_byte in &mut raw_bytes {
            *output_byte = match iter.next() {
                None => return Err(ProtoscopeRsError::Eof),
                Some(input_byte) => (*input_byte).clone(),
            };
        }
        Ok((Self::decode_from_bytes(raw_bytes), iter))
    }

    fn decode_from_bytes(raw_bytes: [u8; 8]) -> Self;
}

trait DecodeI32 {
    fn decode(mut iter: ByteIterator) -> Result<(Self, ByteIterator)>
    where
        Self: Sized,
    {
        let mut raw_bytes = [0u8; 4];
        for output_byte in &mut raw_bytes {
            *output_byte = match iter.next() {
                None => return Err(ProtoscopeRsError::Eof),
                Some(input_byte) => (*input_byte).clone(),
            };
        }
        Ok((Self::decode_from_bytes(raw_bytes), iter))
    }

    fn decode_from_bytes(raw_bytes: [u8; 4]) -> Self;
}

impl DecodeI64 for f64 {
    fn decode_from_bytes(raw_bytes: [u8; 8]) -> Self {
        f64::from_le_bytes(raw_bytes)
    }
}

impl DecodeI32 for f32 {
    fn decode_from_bytes(raw_bytes: [u8; 4]) -> Self {
        f32::from_le_bytes(raw_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floating_types() {
        let mut buffer: Vec<u8> = vec![0, 8];
        let a = 1.0f32
            .encode(buffer.iter_mut())
            .is_ok_and(|num_bytes_encoded| {
                f32::decode(buffer[0..num_bytes_encoded].into_iter()).is_ok_and(|(f32_value, _)| {
                    println!("{}", f32_value);
                    f32_value
                        .to_le_bytes()
                        .into_iter()
                        .zip(1.0f32.to_le_bytes().iter())
                        .all(|(float1_u8, float2_u8)| {
                            println!("{} {}", float1_u8, float2_u8);
                            float1_u8 == *float2_u8
                        })
                })
            });
        assert!(a);
    }
}
