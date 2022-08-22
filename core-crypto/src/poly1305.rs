// (c) 2020-2022 ZeroTier, Inc. -- currently propritery pending actual release and licensing. See LICENSE.md.

use poly1305::universal_hash::NewUniversalHash;

/// The poly1305 message authentication function.
pub struct Poly1305(poly1305::Poly1305, [u8; 16], usize);

pub const POLY1305_ONE_TIME_KEY_SIZE: usize = 32;
pub const POLY1305_MAC_SIZE: usize = 16;

#[inline(always)]
pub fn compute(one_time_key: &[u8], message: &[u8]) -> [u8; POLY1305_MAC_SIZE] {
    poly1305::Poly1305::new(poly1305::Key::from_slice(one_time_key)).compute_unpadded(message).into_bytes().into()
}

#[cfg(test)]
mod tests {
    use crate::poly1305::*;

    const TV0_INPUT: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    const TV0_KEY: [u8; 32] = [
        0x74, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, 0x33, 0x32, 0x2d, 0x62, 0x79, 0x74, 0x65, 0x20, 0x6b, 0x65, 0x79, 0x20, 0x66, 0x6f, 0x72, 0x20, 0x50, 0x6f, 0x6c, 0x79, 0x31, 0x33, 0x30, 0x35,
    ];
    const TV0_TAG: [u8; 16] = [0x49, 0xec, 0x78, 0x09, 0x0e, 0x48, 0x1e, 0xc6, 0xc2, 0x6b, 0x33, 0xb9, 0x1c, 0xcc, 0x03, 0x07];

    const TV1_INPUT: [u8; 12] = [0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0x21];
    const TV1_KEY: [u8; 32] = [
        0x74, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, 0x33, 0x32, 0x2d, 0x62, 0x79, 0x74, 0x65, 0x20, 0x6b, 0x65, 0x79, 0x20, 0x66, 0x6f, 0x72, 0x20, 0x50, 0x6f, 0x6c, 0x79, 0x31, 0x33, 0x30, 0x35,
    ];
    const TV1_TAG: [u8; 16] = [0xa6, 0xf7, 0x45, 0x00, 0x8f, 0x81, 0xc9, 0x16, 0xa2, 0x0d, 0xcc, 0x74, 0xee, 0xf2, 0xb2, 0xf0];

    #[test]
    fn poly1305() {
        assert_eq!(TV0_TAG, compute(&TV0_KEY, &TV0_INPUT));
        assert_eq!(TV1_TAG, compute(&TV1_KEY, &TV1_INPUT));
    }
}
