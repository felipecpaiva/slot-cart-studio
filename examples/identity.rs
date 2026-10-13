//! Every GB, GBC and GBA output the studio makes for a fixed set of synthetic carts, written to a
//! folder: labels, cart faces, shell values and the shell file. Run it against two slot trees and
//! `diff -r` the folders to prove a change left those platforms byte for byte as they were.
//!
//! cargo run --example identity -- <out dir>

use std::path::PathBuf;

use slot_cart_studio::{choice, face, label};

fn png(w: u32, h: u32, colour: png::ColorType, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(colour);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(data).unwrap();
    }
    out
}

fn gb_head(title: &[u8], code: &[u8], cgb: u8, japan: bool) -> Vec<u8> {
    let mut h = vec![0u8; 0x150];
    h[0x134..0x134 + title.len()].copy_from_slice(title);
    h[0x13f..0x13f + code.len()].copy_from_slice(code);
    h[0x143] = cgb;
    h[0x14a] = u8::from(!japan);
    h
}

fn main() {
    let out = PathBuf::from(std::env::args().nth(1).expect("usage: identity <out dir>"));
    std::fs::create_dir_all(&out).unwrap();
    let put = |name: &str, bytes: &[u8]| std::fs::write(out.join(name), bytes).unwrap();

    let flat = png(
        64,
        64,
        png::ColorType::Rgb,
        &[0xd0, 0x20, 0xa0].repeat(64 * 64),
    );
    let tall: Vec<u8> = (0..480u32)
        .flat_map(|y| (0..160u32).map(move |x| (x, y)))
        .flat_map(|(x, y)| [x as u8, (y % 256) as u8, 0x40])
        .collect();
    let tall = png(160, 480, png::ColorType::Rgb, &tall);
    let logo: Vec<u8> = (0..60u32)
        .flat_map(|y| (0..120u32).map(move |x| (x, y)))
        .flat_map(|(x, y)| {
            if (x / 10 + y / 10) % 2 == 0 {
                [250, 210, 20, 255]
            } else {
                [0, 0, 0, 0]
            }
        })
        .collect();
    let logo = png(120, 60, png::ColorType::Rgba, &logo);

    let carts: Vec<(&str, &str, Vec<u8>, &str)> = vec![
        (
            "GBA",
            "BPEE",
            vec![],
            "Pokemon - Emerald Version (USA, Europe)",
        ),
        ("GBA", "AMTE", vec![], "Metroid Fusion (USA)"),
        ("GBA", "", vec![], "Homebrew"),
        (
            "GB",
            "",
            gb_head(b"", b"", 0x00, false),
            "Tetris (World) (Rev 1)",
        ),
        ("GB", "", gb_head(b"", b"", 0x80, false), "Dual (USA)"),
        (
            "GB",
            "",
            gb_head(b"", b"", 0xc0, false),
            "Colour Only (USA)",
        ),
        (
            "GB",
            "",
            gb_head(b"POKEMON RED", b"", 0x00, true),
            "Pocket Monsters Aka (Japan)",
        ),
        (
            "GBC",
            "",
            gb_head(b"POKEMON_GLD", b"AAUE", 0x80, false),
            "Pokemon - Gold Version (USA, Europe)",
        ),
        (
            "GBC",
            "",
            gb_head(b"PM_CRYSTAL", b"BYTE", 0xc0, false),
            "Pokemon - Crystal Version (USA, Europe)",
        ),
    ];
    let shells = [
        "",
        "rounded 336699 clear",
        "notched 123456 glitter",
        "auto c0282c solid",
    ];
    let band = label::Band::parse("top", 0.2, &[10, 20, 30]);

    for p in ["GBA", "GB", "GBC"] {
        let size = slot_cart_studio::cart_size(p);
        put(
            &format!("cart_size_{p}.txt"),
            format!("{size:?}").as_bytes(),
        );
    }
    let mut text = String::new();
    for (i, (p, code, head, stem)) in carts.iter().enumerate() {
        let platform = face::platform_of(p);
        put(
            &format!("{i}_auto.txt"),
            choice::auto_shell(platform, code, head, stem).as_bytes(),
        );
        for (j, shell) in shells.iter().enumerate() {
            let tag = format!("{i}_{j}");
            for (k, bytes) in [&flat, &tall, &b"not a png".to_vec()].iter().enumerate() {
                put(
                    &format!("{tag}_existing{k}.rgba"),
                    &face::from_png(bytes, platform, code, head, shell, stem),
                );
            }
            let made = label::Label::from_png(&logo, [101, 68, 21], platform, None).unwrap();
            put(&format!("{tag}_logo.png"), &made.png());
            put(
                &format!("{tag}_logo.rgba"),
                &made.face(platform, code, head, shell, stem),
            );
            let banded = label::Label::from_png(&logo, [60, 20, 90], platform, band).unwrap();
            put(&format!("{tag}_band.png"), &banded.png());
            put(
                &format!("{tag}_band.rgba"),
                &banded.face(platform, code, head, shell, stem),
            );
            let full = label::Label::full(&tall, platform).unwrap();
            put(&format!("{tag}_full.png"), &full.png());
            put(
                &format!("{tag}_full.rgba"),
                &full.face(platform, code, head, shell, stem),
            );
        }
        text = choice::merge_cart_shells(
            &text,
            &[stem.to_string()],
            &[shells[i % shells.len()].to_string()],
        )
        .unwrap();
    }
    put("cart_shell.txt", text.as_bytes());
    put("presets.txt", choice::presets().join("\n").as_bytes());
    // slot hands these back from a map, in no fixed order.
    let layered = choice::layered_cart_shells(&text, "Homebrew = auto 444444 clear\n");
    let mut pairs: Vec<String> = layered.chunks(2).map(|p| p.join(" = ")).collect();
    pairs.sort();
    put("layered.txt", pairs.join("\n").as_bytes());
}
