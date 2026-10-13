//! T48 (slot-nes-snes issue 40): NES, Famicom, SNES and Super Famicom carts on a synthetic card.
//! Each gets the label well slot cuts for its variant, a label made at that well's shape, a face
//! that is slot's pixel for pixel, and cart_shell.txt lines slot reads back. No real rom or art:
//! the roms are headers and zeros, the logo a checkerboard.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use slot_cart_studio::{choice, face, label, label_well};
use tempfile::TempDir;

const JAPAN: u8 = 0x00;
const USA: u8 = 0x01;

/// A LoROM image whose header slot accepts: map mode 0x20, a checksum and its complement, and
/// the region byte. `copier` puts a 512-byte copier header in front.
fn snes_rom(region: u8, copier: bool) -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    let h = &mut rom[0x7fc0..0x7fe0];
    h[..21].copy_from_slice(b"PROBE CART           ");
    h[0x15] = 0x20;
    h[0x19] = region;
    let sum: u16 = 0x1234;
    h[0x1c..0x1e].copy_from_slice(&(!sum).to_le_bytes());
    h[0x1e..0x20].copy_from_slice(&sum.to_le_bytes());
    if copier {
        let mut out = vec![0u8; 512];
        out.extend_from_slice(&rom);
        out
    } else {
        rom
    }
}

/// An iNES image: the 16-byte header and one 16 KiB bank.
fn nes_rom() -> Vec<u8> {
    let mut rom = vec![0u8; 16 + 16384];
    rom[..4].copy_from_slice(b"NES\x1a");
    rom[4] = 1;
    rom
}

/// What the page hands the studio as a cart's head: nothing for NES, the header windows for SNES.
fn head_of(dir: &str, rom: &[u8]) -> Vec<u8> {
    if dir == "SNES" {
        slot_store::snes::head(rom)
    } else {
        Vec::new()
    }
}

fn png_rgba(w: u32, h: u32, px: impl Fn(u32, u32) -> [u8; 4]) -> Vec<u8> {
    let data: Vec<u8> = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .flat_map(|(x, y)| px(x, y))
        .collect();
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&data).unwrap();
    }
    out
}

fn logo() -> Vec<u8> {
    png_rgba(120, 60, |x, y| {
        if (x / 10 + y / 10) % 2 == 0 {
            [250, 210, 20, 255]
        } else {
            [0, 0, 0, 0]
        }
    })
}

fn png_size(png: &[u8]) -> (u32, u32) {
    let r = png::Decoder::new(std::io::Cursor::new(png))
        .read_info()
        .unwrap();
    (r.info().width, r.info().height)
}

/// A card holding one rom, its label, and a Labels/cart_shell.txt.
fn card(dir: &str, stem: &str, ext: &str, rom: &[u8], label: &[u8], shells: &str) -> TempDir {
    let d = tempfile::tempdir().unwrap();
    for sub in ["Games", "Labels"] {
        std::fs::create_dir_all(d.path().join(sub).join(dir)).unwrap();
    }
    std::fs::write(d.path().join(format!("Games/{dir}/{stem}.{ext}")), rom).unwrap();
    std::fs::write(d.path().join(format!("Labels/{dir}/{stem}.png")), label).unwrap();
    std::fs::write(d.path().join(slot_store::LABELS_SHELL_FILE), shells).unwrap();
    d
}

fn slot_cart(root: &Path) -> slot_store::Cart {
    let mut cart = slot_store::scan(root).unwrap().remove(0);
    cart.read_header();
    cart
}

/// Platform folder, stem, extension, rom, and the variant slot must pick.
type Sample = (Str, Str, Str, Vec<u8>, Str);
type Str = &'static str;

/// The sample card.
fn sample() -> Vec<Sample> {
    let snes =
        |stem, ext, region, copier, variant| ("SNES", stem, ext, snes_rom(region, copier), variant);
    vec![
        ("NES", "Probe Quest (USA)", "nes", nes_rom(), "nes"),
        ("NES", "Probe Quest (Japan)", "nes", nes_rom(), "famicom"),
        snes("Probe World (USA)", "sfc", USA, false, "snes"),
        snes("Probe World (USA) [copier]", "smc", USA, true, "snes"),
        snes("Probe World (Japan)", "sfc", JAPAN, false, "sfc"),
        snes("Probe World (Japan) [copier]", "smc", JAPAN, true, "sfc"),
    ]
}

fn other(variant: &str) -> &'static str {
    match variant {
        "nes" => "famicom",
        "famicom" => "nes",
        "snes" => "sfc",
        _ => "snes",
    }
}

#[test]
fn t48_each_variant_gets_its_own_well_and_slots_face() {
    let logo = logo();
    for (dir, stem, ext, rom, variant) in sample() {
        let platform = face::platform_of(dir);
        let head = head_of(dir, &rom);
        let well = label_well(dir, &head, "", stem);
        assert_eq!(well, variant, "{stem}");

        // The label is made at the well's own shape, which is the box slot cuts for this cart.
        let made = label::Label::from_png(&logo, [40, 60, 120], label::Well::parse(&well), None)
            .expect("logo decodes");
        let png = made.png();
        let (lw, lh) = png_size(&png);
        assert_eq!((lw, lh), label::size(label::Well::parse(&well)), "{stem}");
        assert_eq!(lw.max(lh), label::RETRO_LONG, "{stem}");

        let d = card(dir, stem, ext, &rom, &png, "");
        let cart = slot_cart(d.path());
        let (bw, bh) = slot_ui::label_size(&cart);
        assert_eq!((bw, bh), label::Well::parse(&well).label_box(), "{stem}");
        let (want, got) = (bw as f32 / bh as f32, lw as f32 / lh as f32);
        assert!(
            (want - got).abs() / want < 0.01,
            "{stem}: {got} is not the well's {want}"
        );

        // The preview is slot's own face for the card, label and all, and so is the face of a
        // label that does not decode.
        assert!(
            slot_ui::cart_face(&cart).rgba == made.face(platform, "", &head, "", stem),
            "{stem}: preview differs from slot's cart"
        );
        let bare = card(dir, stem, ext, &rom, b"not a png", "");
        assert!(
            slot_ui::cart_face(&slot_cart(bare.path())).rgba
                == face::from_png(b"not a png", platform, "", &head, "", stem),
            "{stem}: generated label differs from slot's"
        );
    }
}

#[test]
fn t48_cart_shell_lines_carry_the_outline_words_slot_reads() {
    let logo = logo();
    let mut text = String::new();
    for (dir, stem, ext, rom, variant) in sample() {
        let platform = face::platform_of(dir);
        let head = head_of(dir, &rom);

        // Automatic is a valid line slot keeps, in the plastic slot gives this cart.
        let auto = choice::auto_shell(platform, "", &head, stem);
        assert!(auto.starts_with("auto "), "{auto}");
        assert!(slot_store::ShellChoice::parse(&auto).is_some(), "{auto}");

        // Choosing the other variant writes its word, flips the well, and slot draws it so.
        let (_, rest) = auto.split_once(' ').unwrap();
        let line = format!("{} {rest}", other(variant));
        assert_eq!(
            label_well(dir, &head, &line, stem),
            other(variant),
            "{stem}"
        );
        text = choice::merge_cart_shells(&text, &[stem.to_string()], std::slice::from_ref(&line))
            .unwrap();

        let made = label::Label::from_png(
            &logo,
            [40, 60, 120],
            label::Well::parse(other(variant)),
            None,
        )
        .unwrap();
        let shells = format!("{stem} = {line}\n");
        let d = card(dir, stem, ext, &rom, &made.png(), &shells);
        let cart = slot_cart(d.path());
        assert_eq!(cart.shell.map(|c| c.to_value()), Some(line.clone()));
        assert!(
            slot_ui::cart_face(&cart).rgba == made.face(platform, "", &head, &line, stem),
            "{stem}: {line:?} differs from slot's cart"
        );
    }

    // The whole file: every line parses, and survives being laid over Config as slot reads it.
    let read: Vec<String> = choice::cart_shells(&text);
    let layered = choice::layered_cart_shells("", &text);
    for (_, stem, _, _, variant) in sample() {
        let at = read.iter().position(|s| s == stem).expect(stem);
        let word = format!("{} ", other(variant));
        assert!(read[at + 1].starts_with(&word), "{}", read[at + 1]);
        assert!(layered.iter().any(|s| s == stem), "{stem}");
    }
    assert_eq!(read.len(), 2 * sample().len());
}

#[test]
fn a_snes_copier_header_is_skipped_for_the_crc_and_the_header() {
    let plain = snes_rom(JAPAN, false);
    let headered = snes_rom(JAPAN, true);
    let skip = slot_cart_studio::crc_skip("SNES", headered.len() as f64) as usize;
    assert_eq!(skip, 512);
    assert_eq!(slot_cart_studio::crc_skip("SNES", plain.len() as f64), 0);
    assert_eq!(
        crc32fast::hash(&headered[skip..]),
        crc32fast::hash(&plain),
        "a headered dump hashes as the clean one the dat lists"
    );
    assert_eq!(
        slot_store::snes::head(&headered),
        slot_store::snes::head(&plain)
    );
    // NES is hashed whole: the iNES header is part of what libretro's dat lists.
    assert_eq!(slot_cart_studio::crc_skip("NES", nes_rom().len() as f64), 0);
}

/// The shim's outline, read from the header windows the page sends, against slot's own read of
/// the whole file: every probe, region codes on both sides of the map, a bad checksum, a copier.
#[test]
fn the_snes_outline_from_the_page_matches_slots_read_of_the_file() {
    let d = tempfile::tempdir().unwrap();
    let mut roms: Vec<Vec<u8>> = Vec::new();
    for (at, map, len) in [
        (0x7fc0, 0x20, 0x8000),
        (0xffc0, 0x21, 0x10000),
        (0x40ffc0, 0x25, 0x410000),
    ] {
        for region in [0x00, 0x01, 0x02, 0x0d, 0x0f, 0x10, 0x14, 0x15, 0xff] {
            for (sum, copier) in [(0x1234u16, false), (0x1234, true), (0x0000, false)] {
                let mut rom = vec![0u8; len];
                rom[at + 0x15] = map;
                rom[at + 0x19] = region;
                rom[at + 0x1c..at + 0x1e].copy_from_slice(&(!sum).to_le_bytes());
                rom[at + 0x1e..at + 0x20].copy_from_slice(&sum.to_le_bytes());
                if copier {
                    rom.splice(0..0, [0u8; 512]);
                }
                roms.push(rom);
            }
        }
    }
    roms.push(vec![0u8; 100]);
    for (i, rom) in roms.iter().enumerate() {
        let path = d.path().join(format!("{i}.sfc"));
        std::fs::write(&path, rom).unwrap();
        let slot = slot_store::snes::outline(&path);
        let page = slot_store::gb::with_header(&slot_store::snes::head(rom), |key| {
            slot_store::snes::outline(key)
        });
        assert_eq!(page, slot, "rom {i}");
    }
}
