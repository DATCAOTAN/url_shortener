/// Base62 alphabet for encoding: 0-9, a-z, A-Z
const ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const BASE: u64 = 62;

/// Encode a numeric ID into a Base62 string
///
/// # Arguments
/// * `id` - The database ID to encode (must be positive)
///
/// # Returns
/// A Base62 encoded string representing the ID
///
/// # Example
/// ```
/// let code = encode(12345);
/// assert_eq!(code, "3d7");
/// ```
pub fn encode(id: i64) -> String {
    if id == 0 {
        return "0".to_string();
    }

    let mut num = id as u64;
    let mut result = Vec::new();

    while num > 0 {
        let remainder = (num % BASE) as usize;
        result.push(ALPHABET[remainder]);
        num /= BASE;
    }

    // Reverse to get the correct order
    result.reverse();
    String::from_utf8(result).unwrap_or_else(|_| "error".to_string())
}

/// Decode a Base62 string back to a numeric ID
///
/// # Arguments
/// * `code` - The Base62 encoded string
///
/// # Returns
/// The decoded numeric ID, or None if the string is invalid
pub fn decode(code: &str) -> Option<i64> {
    let mut result: u64 = 0;

    for c in code.chars() {
        let value = match c {
            '0'..='9' => c as u64 - '0' as u64,
            'a'..='z' => c as u64 - 'a' as u64 + 10,
            'A'..='Z' => c as u64 - 'A' as u64 + 36,
            _ => return None,
        };

        result = result.checked_mul(BASE)?.checked_add(value)?;
    }

    Some(result as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_zero() {
        assert_eq!(encode(0), "0");
    }

    #[test]
    fn test_encode_single_digit() {
        assert_eq!(encode(1), "1");
        assert_eq!(encode(9), "9");
        assert_eq!(encode(10), "a");
        assert_eq!(encode(35), "z");
        assert_eq!(encode(36), "A");
        assert_eq!(encode(61), "Z");
    }

    #[test]
    fn test_encode_multi_digit() {
        assert_eq!(encode(62), "10");
        assert_eq!(encode(12345), "3d7");
    }

    #[test]
    fn test_decode_roundtrip() {
        for id in [1, 62, 100, 12345, 999999, 1000000000] {
            let encoded = encode(id);
            let decoded = decode(&encoded);
            assert_eq!(decoded, Some(id));
        }
    }
}
