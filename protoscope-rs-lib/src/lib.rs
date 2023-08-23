type ByteIterator<'a> = std::slice::Iter<'a, u8>;

#[derive(PartialEq)]
pub enum ProtoscopeError {
    VarintOverflow,
    Eof,
}

pub type Result<T> = std::result::Result<T, ProtoscopeError>;

#[unroll::unroll_for_loops]
pub fn extract_from_encoded_varint(mut iter: ByteIterator) -> Result<(u64, ByteIterator)> {
    match iter.clone().peekable().peek() {
        None => return Err(ProtoscopeError::Eof),
        _ => {}
    }

    const MAX_NUMBER_OF_BYTES: usize = ((std::mem::size_of::<u64>() * 8) + 7 - 1) / 7;

    let mut decoded_value: u64 = 0;

    for byte_idx in 0..MAX_NUMBER_OF_BYTES {
        match &iter.next() {
            Some(byte) => {
                let payload = 0b01111111 & *byte;
                decoded_value = (decoded_value) | ((payload as u64) << (7 * byte_idx));
                if 0b10000000 & *byte == 0 {
                    break;
                }else if byte_idx == (MAX_NUMBER_OF_BYTES - 1) {
                    return Err(ProtoscopeError::VarintOverflow);
                }
            }
            None => {
                break;
            }
        }
    }
    Ok((decoded_value, iter.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_from_encoded_varint() {
        let value = extract_from_encoded_varint([0x96u8, 0x01u8].iter());
        assert!(value.is_ok_and(|(value, _)| value == 150));

        let value = extract_from_encoded_varint([0xc8u8, 0x03u8].iter());
        assert!(value.is_ok_and(|(value, _)| value == 456));

        let value = extract_from_encoded_varint([0xacu8, 0x02u8].iter());
        assert!(value.is_ok_and(|(value, _)| value == 300));
    }

    #[test]
    fn test_extract_from_encoded_varint_multiple() {
        let mut iter = [0x96u8, 0x01u8, 0xc8u8, 0x03u8, 0xacu8, 0x02u8].iter();
        let value = extract_from_encoded_varint(iter.clone());
        assert!(value.is_ok_and(|(value, it)| {
            iter = it;
            value == 150
        }));

        let value = extract_from_encoded_varint(iter.clone());
        assert!(value.is_ok_and(|(value, it)| {
            iter = it;
            value == 456
        }));

        let value = extract_from_encoded_varint(iter.clone());
        assert!(value.is_ok_and(|(value, it)| {
            iter = it;
            value == 300
        }));
        let value = extract_from_encoded_varint(iter.clone());
        assert!(value.is_err_and(|err| {
           err == ProtoscopeError::Eof
        }));
    }

    #[test]
    fn test_extract_from_encoded_varint_overflow() {
        let iter = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff].iter();
        let value = extract_from_encoded_varint(iter.clone());
        assert!(value.is_err_and(|err| { err == ProtoscopeError::VarintOverflow }));
    }
}
