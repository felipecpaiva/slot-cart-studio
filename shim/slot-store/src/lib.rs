//! slot-store as slot's cart face sees it, with the Game Boy header reads swapped. Those open the
//! rom, which a browser cannot do, so these answer from header bytes the page read.

pub use ::slot_store_real::*;

pub mod gb {
    use std::cell::RefCell;
    use std::path::Path;

    pub use ::slot_store_real::gb::*;

    const KEY: &str = "stash:rom";

    thread_local! {
        static HEAD: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) };
    }

    /// Runs `f` with a rom key whose header is `head`, for the length of one draw.
    pub fn with_header<R>(head: &[u8], f: impl FnOnce(&Path) -> R) -> R {
        HEAD.with(|s| *s.borrow_mut() = Some(head.to_vec()));
        let out = f(Path::new(KEY));
        HEAD.with(|s| *s.borrow_mut() = None);
        out
    }

    fn stashed(rom: &Path) -> Option<Option<Header>> {
        if rom != Path::new(KEY) {
            return None;
        }
        HEAD.with(|s| s.borrow().as_ref().map(|b| Header::parse(b)))
    }

    /// Slot's signatures; any other path is slot's own read.
    pub fn header(rom: &Path) -> Option<Header> {
        stashed(rom).unwrap_or_else(|| ::slot_store_real::gb::header(rom))
    }

    pub fn class(rom: &Path) -> Class {
        match stashed(rom) {
            Some(h) => h.map_or(Class::Original, |h| h.class()),
            None => ::slot_store_real::gb::class(rom),
        }
    }

    /// The stashed bytes themselves, for the SNES read below.
    pub(crate) fn stashed_bytes<R>(rom: &Path, f: impl FnOnce(&[u8]) -> R) -> Option<R> {
        if rom != Path::new(KEY) {
            return None;
        }
        HEAD.with(|s| s.borrow().as_deref().map(f))
    }
}

/// slot-nes-snes's SNES header read, which picks a cart's outline from its region byte. slot
/// seeks into the rom for it, up to 4 MiB in, so the page hands over only the three 32-byte
/// windows slot probes (`head`), already past any copier header. `valid` and `region_outline`
/// are slot's rules restated, because slot keeps them private; `tests/retro.rs` holds this to
/// slot's own read of the same rom.
pub mod snes {
    use std::path::Path;

    pub use ::slot_store_real::snes::*;

    use crate::Outline;

    const COPIER_LEN: usize = 512;
    const PROBES: [(usize, &[u8]); 3] = [(0x7fc0, &[0, 2, 3]), (0xffc0, &[1, 5]), (0x40ffc0, &[5])];
    const HEADER_LEN: usize = 0x20;
    const MAP_OFF: usize = 0x15;
    const REGION_OFF: usize = 0x19;
    const COMPLEMENT_OFF: usize = 0x1c;

    /// The three header windows of a whole rom, zero where the rom is too short to hold one.
    pub fn head(rom: &[u8]) -> Vec<u8> {
        let skip = if rom.len() % 1024 == COPIER_LEN {
            COPIER_LEN
        } else {
            0
        };
        let mut out = vec![0u8; PROBES.len() * HEADER_LEN];
        for (i, (off, _)) in PROBES.iter().enumerate() {
            if let Some(w) = rom.get(skip + off..skip + off + HEADER_LEN) {
                out[i * HEADER_LEN..(i + 1) * HEADER_LEN].copy_from_slice(w);
            }
        }
        out
    }

    fn valid(h: &[u8], modes: &[u8]) -> bool {
        if h[MAP_OFF] & 0xe0 != 0x20 || !modes.contains(&(h[MAP_OFF] & 0x0f)) {
            return false;
        }
        let complement = u16::from_le_bytes([h[COMPLEMENT_OFF], h[COMPLEMENT_OFF + 1]]);
        let checksum = u16::from_le_bytes([h[COMPLEMENT_OFF + 2], h[COMPLEMENT_OFF + 3]]);
        complement ^ checksum == 0xffff && checksum != 0 && checksum != 0xffff
    }

    fn region_outline(code: u8) -> Outline {
        match code {
            0x00 | 0x02..=0x14 => Outline::Sfc,
            _ => Outline::Snes,
        }
    }

    fn from_head(head: &[u8]) -> Option<Outline> {
        PROBES
            .iter()
            .zip(head.chunks_exact(HEADER_LEN))
            .find(|((_, modes), h)| valid(h, modes))
            .map(|(_, h)| region_outline(h[REGION_OFF]))
    }

    /// Slot's signature; any other path is slot's own read.
    pub fn outline(rom: &Path) -> Option<Outline> {
        super::gb::stashed_bytes(rom, from_head)
            .unwrap_or_else(|| ::slot_store_real::snes::outline(rom))
    }
}
