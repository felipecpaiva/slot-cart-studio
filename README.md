# slot cart studio, with NES and SNES

Cart labels for a [slot](https://slot-cfw.fyi) SD card, previewed on the same cart face the
frontend draws.

This is a fork of [Brandon Kowalski's Cart Studio](https://github.com/BrandonKowalski/slot-cart-studio)
(the original runs at [studio.slot-cfw.fyi](https://studio.slot-cfw.fyi) and covers GB, GBC and
GBA). The fork adds NES and SNES carts for
[slot-nes-snes](https://github.com/felipecpaiva/slot-nes-snes), the patch set that brings those
two consoles to slot. Game Boy and GBA carts behave exactly as in the original.

## What the fork adds

- Card folders `Games/NES` (`.nes`) and `Games/SNES` (`.sfc`, `.smc`); labels go to
  `Labels/NES/<stem>.png` and `Labels/SNES/<stem>.png`.
- Each label is made at the shape of the label well slot-nes-snes cuts for the cart's variant:
  `nes` (portrait), `famicom`, `snes` and `sfc` (wide). The variant is slot's own pick: a
  `cart_shell.txt` word for the cart's platform, else the stem's region tags (NES: `(Japan)`
  alone gives Famicom) or the SNES header's region byte (Japan and PAL codes give Super Famicom).
- The editor's Outline row offers NES/Famicom or SNES/Super Famicom, and writes the `nes`,
  `famicom`, `snes` and `sfc` words to `Labels/cart_shell.txt`.
- Identification against libretro-database's No-Intro dats: a NES rom is hashed whole, iNES
  header included (that dat lists headered dumps); a SNES rom drops a 512-byte copier header
  when its size is 512 past a whole KiB.
- `tools/harvest.py` knows NES (ScreenScraper system 3) and SNES (system 4).

NES and SNES carts get a real label or a logo only from an art set that files them under
`NES/` or `SNES/`. Brandon's set at art.slot-cfw.fyi has GB, GBC and GBA art only, so until a
NES/SNES harvest exists (it needs a ScreenScraper developer id) those carts fall back to Logo
Only from a dropped-in logo, or to slot's generated label. Box art is only ever read for a
ground colour, never shown.

## Building against slot-nes-snes

The studio compiles slot's own cart code from a slot checkout beside it, `../slot`, pinned by
`slot.ref`. In this fork `../slot` must be slot v1.6.0 with slot-nes-snes's patches applied,
which is the `work/slot` tree slot-nes-snes builds (`task work:init` then `task work:resume`
there). `slot.ref` names that tree's head commit as it was built here, `4f4c732`, for patches
0001 to 0018. A `work/slot` built elsewhere holds the same files under a different commit, so
write its head into `slot.ref` before `task slot:pin-check`, which says whether the cart face
still matches.

```sh
# the desk: the studio and the patched slot side by side
git clone <slot-nes-snes> && (cd slot-nes-snes && task work:init && task work:resume)
git clone --branch nes-snes <this fork> slot-cart-studio
git clone slot-nes-snes/work/slot slot          # ../slot for the studio

cd slot-cart-studio
task test       # cargo test, then node --test web/*.test.mjs
(cd tools && python3 -m unittest test_harvest test_look)   # test_look needs Pillow and numpy
task serve      # wasm-pack build, then http://127.0.0.1:8765
```

`task build` needs the `wasm32-unknown-unknown` target (named in `rust-toolchain.toml`) and
wasm-pack 0.15.0. `cargo run --example identity -- <dir>` writes every GB, GBC and GBA output
for a fixed synthetic card; run it against the original pin and this one and `diff -r` the two
folders to check those platforms are unchanged.

The Pages workflow in `.github/workflows/pages.yml` and the `CNAME` are still the original's:
they build against upstream slot and publish to the original's domain. Both have to change
before this fork is hosted anywhere.

## License

GPL-3.0-or-later, same as slot and the original Cart Studio. See [LICENSE](LICENSE).

Game names and CRCs come from
[libretro-database](https://github.com/libretro/libretro-database) (CC BY-SA 4.0); logos come
from [ScreenScraper](https://www.screenscraper.fr/) (CC BY-NC-SA 4.0). No game art or rom is
part of this repository.

Game Boy, NES, Super NES, Famicom and Super Famicom are trademarks of Nintendo. Not affiliated
with Nintendo.
