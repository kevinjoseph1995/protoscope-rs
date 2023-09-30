use crate::wire_types::{Decode, Encode};
use crate::{ByteIterator, OutputByteIterator, ProtoscopeRsError, Result};

fn encode_internal<'a, T: EncodeLengthDelimited<'a>>(
    value: &'a T,
    iter: &mut OutputByteIterator,
) -> Result<usize> {
    let mut total_number_of_bytes_encoded = 0;
    let length = value.get_length()?;
    total_number_of_bytes_encoded += length.encode(iter)?;
    let mut payload_iterator = value.get_payload_iterator();
    for _ in 0..length {
        let payload_byte = match payload_iterator.next() {
            Some(byte) => *byte,
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

trait EncodeLengthDelimited<'a>: Encode<'a> {
    fn get_length(&self) -> Result<i32>;
    fn get_payload_iterator(&'a self) -> ByteIterator<'a>;
}

fn decode_internal<T: DecodeLengthDelimited>(iter: &mut ByteIterator) -> Result<T> {
    let length = i32::decode(iter)?;
    let output_buffer: Vec<u8> = iter.copied().take(length as usize).collect();
    if output_buffer.len() != length as usize {
        return Err(ProtoscopeRsError::LengthMismatch);
    }
    T::from_raw_buffer(output_buffer)
}

trait DecodeLengthDelimited: Decode {
    fn from_raw_buffer(buffer: Vec<u8>) -> Result<Self>;
}

impl<'a> EncodeLengthDelimited<'a> for String {
    fn get_length(&self) -> Result<i32> {
        let length = self.len();
        if length <= i32::MAX as usize {
            Ok(self.len() as i32)
        } else {
            Err(ProtoscopeRsError::EncodeOverflow)
        }
    }
    fn get_payload_iterator(&'a self) -> ByteIterator<'a> {
        let bytes: &'a [u8] = self.as_bytes();
        let iter: ByteIterator<'a> = bytes.iter();
        iter
    }
}

impl<'a> Encode<'a> for String {
    fn encode(&'a self, iter: &mut OutputByteIterator) -> Result<usize> {
        encode_internal(self, iter)
    }
}

impl Decode for String {
    fn decode(iter: &mut ByteIterator) -> Result<Self> {
        decode_internal(iter)
    }
}

impl DecodeLengthDelimited for String {
    fn from_raw_buffer(buffer: Vec<u8>) -> Result<Self> {
        String::from_utf8(buffer).map_err(|_| ProtoscopeRsError::UtfDecoding)
    }
}

impl<'a> Encode<'a> for Vec<u8> {
    fn encode(&'a self, iter: &mut OutputByteIterator) -> Result<usize> {
        encode_internal(self, iter)
    }
}

impl<'a> EncodeLengthDelimited<'a> for Vec<u8> {
    fn get_length(&self) -> Result<i32> {
        let length = self.len();
        if length <= i32::MAX as usize {
            Ok(self.len() as i32)
        } else {
            Err(ProtoscopeRsError::EncodeOverflow)
        }
    }

    fn get_payload_iterator(&'a self) -> ByteIterator<'a> {
        self.iter()
    }
}

impl Decode for Vec<u8> {
    fn decode(iter: &mut ByteIterator) -> Result<Self> {
        decode_internal(iter)
    }
}

impl DecodeLengthDelimited for Vec<u8> {
    fn from_raw_buffer(buffer: Vec<u8>) -> Result<Self> {
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {

    use crate::wire_types::Decode;
    use crate::wire_types::Encode;
    use crate::wire_types::ProtoscopeRsError;

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

    #[test]
    fn test_bytes_encode_decode() {
        let mut buffer: Vec<u8> = vec![0; 100];
        let message_buffer: Vec<u8> = vec![2; 3];
        assert!(message_buffer
            .encode(&mut buffer.iter_mut())
            .is_ok_and(|num_bytes_encoded| {
                Vec::<u8>::decode(&mut buffer[0..num_bytes_encoded].into_iter())
                    .is_ok_and(|decoded_buffer| decoded_buffer.into_iter().all(|byte| byte == 2))
            }));
    }

    #[test]
    fn test_large_bytes_array_encode_decode() {
        let mut buffer: Vec<u8> = vec![0; 10000];
        let message_buffer: Vec<u8> = vec![2; 1000];
        assert!(message_buffer
            .encode(&mut buffer.iter_mut())
            .is_ok_and(|num_bytes_encoded| {
                Vec::<u8>::decode(&mut buffer[0..num_bytes_encoded].into_iter())
                    .is_ok_and(|decoded_buffer| decoded_buffer.into_iter().all(|byte| byte == 2))
            }));
    }

    #[test]
    fn test_bytes_array_encode_decode_insufficentspace() {
        let mut buffer: Vec<u8> = vec![0; 1];
        let message_buffer: Vec<u8> = vec![2; 1000];
        assert!(message_buffer
            .encode(&mut buffer.iter_mut())
            .is_err_and(|err| err == ProtoscopeRsError::BufferFull));
    }
}
