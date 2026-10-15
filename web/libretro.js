// libretro's databases and thumbnails, both from raw.githubusercontent.com, which answers
// cross-origin requests. thumbnails.libretro.com serves the same images and sends no
// access-control-allow-origin header at all, so a page can fetch it and still not read a byte.

const DAT = 'https://raw.githubusercontent.com/libretro/libretro-database/master/metadat/no-intro/';
const THUMBS = 'https://raw.githubusercontent.com/libretro-thumbnails/';

// One database and one thumbnail repository per platform, under libretro's own name for the
// system: spaces in the database file, underscores in the repository. The platform is the card
// folder a rom sits in, so a .gbc filed under GB is looked up in the Game Boy database — slot
// decides a platform by where a file is, and the studio does not second-guess it by extension.
const NAMES = {
  GBA: 'Nintendo - Game Boy Advance',
  GB: 'Nintendo - Game Boy',
  GBC: 'Nintendo - Game Boy Color',
  // This NES dat lists headered dumps (16400 bytes for a 16 KiB rom), so a .nes is hashed whole.
  NES: 'Nintendo - Nintendo Entertainment System',
  SNES: 'Nintendo - Super Nintendo Entertainment System',
};

export async function fetchDat(platform) {
  const r = await fetch(`${DAT}${encodeURIComponent(NAMES[platform])}.dat`);
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  return r.text();
}

// A card can hold hundreds of carts, each wanting two images, and GitHub is not a CDN.
export function limiter(slots) {
  let running = 0;
  const waiting = [];
  const pump = () => {
    while (running < slots && waiting.length) {
      const { job, resolve, reject } = waiting.shift();
      running++;
      job()
        .then(resolve, reject)
        .finally(() => {
          running--;
          pump();
        });
    }
  };
  return (job) =>
    new Promise((resolve, reject) => {
      waiting.push({ job, resolve, reject });
      pump();
    });
}

// A set of our own, hosted as static files: <platform>/<media>/<CRC32>.png, with an index.json
// mapping a rom's checksum to what it holds. Looked up by checksum rather than by name, so it
// needs no agreement about spelling and reaches roms No-Intro has never heard of. It is a
// separate origin from the page, so whatever serves it has to allow the read.
export async function fetchIndex(base) {
  const r = await fetch(new URL('index.json', base));
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  return r.json();
}

export async function fetchArt(base, path) {
  const r = await fetch(new URL(path, base));
  if (r.status === 404) return null;
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  return new Uint8Array(await r.arrayBuffer());
}

// A thumbnail's bytes, or null when libretro has none. A git symlink comes back as a one-line
// body naming its target in the same folder; `stubTarget`, from the Rust side, tells the two apart.
export async function fetchThumb(platform, folder, name, stubTarget) {
  const repo = NAMES[platform].replaceAll(' ', '_');
  const get = async (file) => {
    const r = await fetch(`${THUMBS}${repo}/master/${folder}/${encodeURIComponent(file)}`);
    if (r.status === 404) return null;
    if (!r.ok) throw new Error(`HTTP ${r.status}`);
    return new Uint8Array(await r.arrayBuffer());
  };
  const body = await get(`${name}.png`);
  if (!body) return null;
  const target = stubTarget(body);
  return target === undefined ? body : get(target);
}

// The platforms whose thumbnail repository has a Named_Logos folder. Game Boy Color's has none
// (tools/harvest.py says so), which is why the art sets exist.
// NES and SNES only: GB and GBA keep the original studio's curated-logos behaviour.
export const LOGOS = ['NES', 'SNES'];

// A cart's logo: its art set's own, else, for a game the set has no art for at all (no label scan
// and no logo), libretro's Named_Logos by name. Only ever a logo: never box art or a screenshot.
// `ours` and `theirs` fetch, or give null for what isn't there; null back means slot's own label.
// The idea is bently0602's (github.com/bently0602/slot-cart-studio, GPL-3.0).
export async function logoFor({ platform, name, hasReal }, ours, theirs, readable) {
  const own = await ours();
  if (own && readable(own)) return own;
  if (hasReal || !name || !LOGOS.includes(platform)) return null;
  const logo = await theirs(name);
  return logo && readable(logo) ? logo : null;
}
