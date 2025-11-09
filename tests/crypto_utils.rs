use hex_literal::hex;
use provenance_mark::crypto_utils::*;

#[test]
fn test_sha256() {
    let data = b"Hello World";
    assert_eq!(
        sha256(data),
        hex!(
            "a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e"
        )
    );
}

#[test]
fn test_extend_key() {
    let data = b"Hello World";
    assert_eq!(
        extend_key(data),
        hex!(
            "813085a508d5fec645abe5a1fb9a23c2a6ac6bef0a99650017b3ef50538dba39"
        )
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
