//! What the page needs from a ROM's bytes: its game code, for the shell colour, and its CRC32,
//! to find it in the No-Intro DAT.

use slot_store::Platform;

/// The four character game code at 0xAC, read the way `slot_store::gba` reads it: up to the
/// first NUL, trimmed, and empty when there's nothing there. A cart then gets the same shell it
/// gets on the device.
pub fn header_code(head: &[u8]) -> String {
    let Some(field) = head.get(0xac..0xb0) else {
        return String::new();
    };
    let end = field.iter().position(|b| *b == 0).unwrap_or(field.len());
    std::str::from_utf8(&field[..end])
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// The bytes a rom's CRC32 starts after, by the rule the No-Intro dats are hashed under: a SNES
/// rom drops a 512-byte copier header, which a file has when its size is 512 past a whole KiB.
/// A NES rom is hashed whole, iNES header included, as libretro's NES dat lists it.
pub fn crc_skip(platform: Platform, len: u64) -> u64 {
    if platform == Platform::Snes && len % 1024 == 512 {
        512
    } else {
        0
    }
}

/// A ROM is up to 32 MB and the page reads it as a stream, so the hash is fed in pieces.
#[derive(Default)]
pub struct Crc32(crc32fast::Hasher);

impl Crc32 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, chunk: &[u8]) {
        self.0.update(chunk);
    }

    pub fn finish(self) -> u32 {
        self.0.finalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_check_value_is_the_standard_one() {
        let mut crc = Crc32::new();
        crc.update(b"123456789");
        assert_eq!(crc.finish(), 0xCBF4_3926);
    }

    #[test]
    fn chunks_hash_the_same_as_one_update() {
        let data: Vec<u8> = (0..100_000u32).map(|i| (i * 7 % 251) as u8).collect();
        let mut whole = Crc32::new();
        whole.update(&data);
        let mut parts = Crc32::new();
        for chunk in data.chunks(4113) {
            parts.update(chunk);
        }
        assert_eq!(whole.finish(), parts.finish());
    }

    #[test]
    fn only_a_snes_copier_header_is_skipped() {
        assert_eq!(crc_skip(Platform::Snes, 32768 + 512), 512);
        assert_eq!(crc_skip(Platform::Snes, 32768), 0);
        assert_eq!(crc_skip(Platform::Snes, 32768 + 16), 0);
        assert_eq!(crc_skip(Platform::Nes, 16384 + 16), 0);
        assert_eq!(crc_skip(Platform::Nes, 16384 + 512), 0);
        assert_eq!(crc_skip(Platform::Gba, 1024 + 512), 0);
    }

    #[test]
    fn the_code_stops_at_a_nul_and_is_empty_when_missing() {
        let mut head = vec![0u8; 0xb0];
        head[0xac..0xb0].copy_from_slice(b"AB\0\0");
        assert_eq!(header_code(&head), "AB");
        assert_eq!(header_code(&[0u8; 0xb0]), "");
        assert_eq!(header_code(&[0u8; 0x20]), "");
    }
}
