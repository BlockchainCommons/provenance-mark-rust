use hex_literal::hex;
use provenance_mark::{
    crypto_utils::sha256, xoshiro256starstar::Xoshiro256StarStar,
};

#[test]
fn test_rng() {
    let data = b"Hello World";
    let digest = sha256(data);
    let mut rng = Xoshiro256StarStar::from_data(&digest);
    let key = rng.next_bytes(32);
    assert_eq!(
        key,
        hex!(
            "b18b446df414ec00714f19cb0f03e45cd3c3d5d071d2e7483ba8627c65b9926a"
        )
    );
}

#[test]
fn test_save_rng_state() {
    let state: [u64; 4] = [
        17295166580085024720,
        422929670265678780,
        5577237070365765850,
        7953171132032326923,
    ];
    let data = Xoshiro256StarStar::from_state(&state).to_data();
    assert_eq!(
        data,
        hex!(
            "d0e72cf15ec604f0bcab28594b8cde05dab04ae79053664d0b9dadc201575f6e"
        )
    );
    let state2 = Xoshiro256StarStar::from_data(&data).to_state();
    let data2 = Xoshiro256StarStar::from_state(&state2).to_data();
    assert_eq!(data, data2);
}
