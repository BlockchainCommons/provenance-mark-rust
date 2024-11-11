use bc_crypto::{ hkdf_hmac_sha256, SHA256_SIZE };
use chacha20::ChaCha20;
use chacha20::cipher::{ KeyIvInit, StreamCipher };

pub fn sha256(data: impl AsRef<[u8]>) -> [u8; SHA256_SIZE] {
    bc_crypto::sha256(data.as_ref())
}

pub fn sha256_prefix(data: impl AsRef<[u8]>, prefix: usize) -> Vec<u8> {
    let digest = sha256(data);
    digest.iter().take(prefix).copied().collect()
}

pub fn extend_key(data: impl AsRef<[u8]>) -> [u8; 32] {
    let a = hkdf_hmac_sha256(data.as_ref(), [], 32);
    let mut b = [0u8; 32];
    b.copy_from_slice(&a);
    b
}

pub fn obfuscate(key: impl AsRef<[u8]>, message: impl AsRef<[u8]>) -> Vec<u8> {
    let key = key.as_ref();
    let message = message.as_ref();

    if message.is_empty() {
        return message.to_vec();
    }

    let extended_key = extend_key(key);
    let iv = extended_key.iter().rev().take(12).copied().collect::<Vec<u8>>();
    let iv2: [u8; 12] = iv.as_slice().try_into().unwrap();
    let mut cipher = ChaCha20::new(&extended_key.into(), &iv2.into());
    let mut buffer = message.to_vec();
    cipher.apply_keystream(&mut buffer);
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_sha256() {
        let data = b"Hello World";
        assert_eq!(
            sha256(data),
            hex!("a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e")
        );
    }

    #[test]
    fn test_extend_key() {
        let data = b"Hello World";
        assert_eq!(
            extend_key(data),
            hex!("813085a508d5fec645abe5a1fb9a23c2a6ac6bef0a99650017b3ef50538dba39")
        );
    }

    #[test]
    fn test_obfuscate() {
        let key = b"Hello";
        let message = b"World";
        let obfuscated = obfuscate(key, message);
        assert_eq!(obfuscated, hex!("c43889aafa"));

        let deobfuscated = obfuscate(key, obfuscated);
        assert_eq!(deobfuscated, message);
    }
}
