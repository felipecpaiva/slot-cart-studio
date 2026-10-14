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

The studio reads art from two sets, each a folder of CRC-named images with an `index.json`:
GB, GBC and GBA art from Brandon's set at `https://art.slot-cfw.fyi/` (`?art=` points elsewhere),
NES and SNES art from this fork's own set at `https://felipecpaiva.github.io/slot-nes-snes-art/`
(`?art_retro=` points elsewhere). An empty value turns that set off. A set that fails to load
leaves only its own platforms on slot's generated labels or a dropped-in logo; the other set
still works. Box art is only ever read for a ground colour, never shown.

## Building against slot-nes-snes

The studio compiles slot's own cart code from a slot checkout beside it, `../slot`. In this fork
`../slot` is slot v1.6.0 with slot-nes-snes's patches applied, which is not a commit anywhere:
`slot-nes-snes.ref` names the
[slot-nes-snes](https://github.com/felipecpaiva/slot-nes-snes) commit that rebuilds it, from
slot's release tarball (sha256 checked against that commit's `SLOT_PIN`) and that commit's
`patches/`. The Pages workflow does exactly this; on a desk:

```sh
mkdir desk && cd desk
git clone https://github.com/felipecpaiva/slot-nes-snes
git clone --branch nes-snes https://github.com/felipecpaiva/slot-cart-studio
git -C slot-nes-snes checkout --detach "$(cat slot-cart-studio/slot-nes-snes.ref)"
sh slot-nes-snes/scripts/fetch-slot.sh fetch "$PWD/slot"       # ../slot for the studio
sh slot-nes-snes/scripts/apply-patches.sh build "$PWD/slot"

cd slot-cart-studio
task test       # cargo test, then node --test web/*.test.mjs
(cd tools && python3 -m unittest test_harvest test_look)   # test_look needs Pillow and numpy
task serve      # wasm-pack build, then http://127.0.0.1:8765
```

To move to a newer slot-nes-snes, write its commit into `slot-nes-snes.ref`, rebuild `../slot`
as above and run `task test`: the face tests hold every face the studio draws to the one slot's
own code draws.

`task build` needs the `wasm32-unknown-unknown` target (named in `rust-toolchain.toml`) and
wasm-pack 0.15.0. `cargo run --example identity -- <dir>` writes every GB, GBC and GBA output
for a fixed synthetic card; run it against the original studio and this one and `diff -r` the
two folders to check those platforms are unchanged.

## Publishing

`.github/workflows/pages.yml` builds on every push to `nes-snes` (or by hand) and publishes
`web/` to GitHub Pages at `https://felipecpaiva.github.io/slot-cart-studio/`. There is no
`CNAME`. The footer links the commit it was built from.

## Hosting the NES/SNES art set

The NES/SNES set is harvested from ScreenScraper with `tools/harvest.py` and published as its
own repository, `slot-nes-snes-art`, on GitHub Pages (which answers cross-origin reads, so the
studio can fetch it). The published set covers the maintainer's library (about 1630 NES and 755
SNES carts, label scans reduced to 256-colour palettes to stay well under the Pages size limit), not
every No-Intro dump. To harvest every dump instead, run one platform per run into the same folder;
the second run adds to the first's `index.json`:

```sh
export SS_DEVID=... SS_DEVPASSWORD=... SS_SOFTNAME=...   # SS_SSID and SS_SSPASSWORD optional
python3 -m pip install Pillow numpy                     # for the measured label look
python3 tools/harvest.py --dat NES  --out ../slot-nes-snes-art
python3 tools/harvest.py --dat SNES --out ../slot-nes-snes-art
```

That writes `NES/<media>/<CRC32>.png`, `SNES/<media>/<CRC32>.png` and `index.json`, the layout
the studio reads. A run stops when ScreenScraper says the day's quota is spent; run it again and
it skips what is already on disk. With a developer login alone (no `SS_SSID`) it never asks for
member limits (`ssuserInfos.php` refuses a developer-only login) and paces itself at one call a
second. The first run checks ScreenScraper's names for systems 3 and 4 and stops if they are not
the ones `SS_EXPECT` states. The set is CC BY-NC-SA 4.0, as ScreenScraper's media are: give its
repository that licence and a line crediting ScreenScraper.

## License

GPL-3.0-or-later, same as slot and the original Cart Studio. See [LICENSE](LICENSE).

Game names and CRCs come from
[libretro-database](https://github.com/libretro/libretro-database) (CC BY-SA 4.0); logos come
from [ScreenScraper](https://www.screenscraper.fr/) (CC BY-NC-SA 4.0), NES and SNES logos and labels
by way of the [slot-nes-snes-art](https://github.com/felipecpaiva/slot-nes-snes-art) set. No game
art or rom is part of this repository.

Game Boy, NES, Super NES, Famicom and Super Famicom are trademarks of Nintendo. Not affiliated
with Nintendo.
