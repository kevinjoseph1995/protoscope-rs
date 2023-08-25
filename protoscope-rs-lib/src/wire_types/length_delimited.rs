use crate::wire_types::varint::{DecodeVarint, EncodeVarint};
use crate::{ByteIterator, OutputByteIterator, ProtoscopeRsError, Result};

pub trait EncodeLengthDelimited<'a> {
    fn encode(&'a self, iter: &mut OutputByteIterator) -> Result<usize> {
        let mut total_number_of_bytes_encoded = 0;
        let length = self.get_length()?;
        total_number_of_bytes_encoded += length.encode(iter)?;
        let mut payload_iterator = self.get_payload_iterator();
        for _ in 0..length {
            let payload_byte = match payload_iterator.next() {
                Some(byte) => byte.clone(),
                None => return Err(ProtoscopeRsError::LengthMismatch),
            };
            let output_byte = match iter.next() {
                Some(byte) => byte,
                None => return Err(ProtoscopeRsError::BufferFull),
            };
            *output_byte = payload_byte;
        }
        total_number_of_bytes_encoded += length as usize;
        Ok(total_number_of_bytes_encoded)
    }
    fn get_length(&self) -> Result<i32>;
    fn get_payload_iterator(&'a self) -> ByteIterator<'a>;
}

pub trait DecodeLengthDelimited: Sized {
    fn decode(iter: &mut ByteIterator) -> Result<Self> {
        let length = i32::decode(iter)?;
        let output_buffer: Vec<u8> = iter
            .map(|byte| byte.clone())
            .take(length as usize)
            .collect();
        if output_buffer.len() != length as usize {
            return Err(ProtoscopeRsError::LengthMismatch);
        }
        Self::from_raw_buffer(output_buffer)
    }
    fn from_raw_buffer(buffer: Vec<u8>) -> Result<Self>;
}

impl<'a> EncodeLengthDelimited<'a> for String {
    fn get_length(&self) -> Result<i32> {
        Ok(self.len() as i32)
    }

    fn get_payload_iterator(&'a self) -> ByteIterator<'a> {
        let bytes: &'a [u8] = self.as_bytes();
        let iter: ByteIterator<'a> = bytes.iter();
        iter
    }
}

impl DecodeLengthDelimited for String {
    fn from_raw_buffer(buffer: Vec<u8>) -> Result<Self> {
        String::from_utf8(buffer).map_err(|_| ProtoscopeRsError::UtfDecoding)
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_string_encode_decode() {
        let mut buffer: Vec<u8> = vec![0; 100];
        assert!(String::from("Hello_world")
            .encode(&mut buffer.iter_mut())
            .is_ok_and(|num_bytes_encoded| {
                String::decode(&mut buffer[0..num_bytes_encoded].into_iter())
                    .is_ok_and(|output_string| output_string == "Hello_world")
            }));
    }

    #[test]
    fn test_large_string_encode_decode() {
        let mut buffer: Vec<u8> = vec![0; 20000];
        let large_string = String::from_utf8(vec![0u8; 10000]).unwrap();
        assert!(large_string
            .encode(&mut buffer.iter_mut())
            .is_ok_and(|num_bytes_encoded| {
                String::decode(&mut buffer[0..num_bytes_encoded].into_iter())
                    .is_ok_and(|output_string| output_string == large_string)
            }));
    }

    #[test]
    fn test_string_encode_decode_insufficentspace() {
        let mut buffer: Vec<u8> = vec![0; 1];
        assert!(String::from("Hello_world")
            .encode(&mut buffer.iter_mut())
            .is_err_and(|err| err == ProtoscopeRsError::BufferFull));
    }
}
