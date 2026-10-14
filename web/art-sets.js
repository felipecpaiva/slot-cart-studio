// Two art sets, each a base url holding index.json and the images it names. Game Boy and GBA art
// is Brandon's set; NES and SNES art is slot-nes-snes's own, harvested and hosted apart from it.
// `?art=` and `?art_retro=` point either at another (a local one while it is being harvested), and
// an empty value leaves that set's platforms to libretro and slot's own labels.

// slot-nes-snes's platforms.
export const RETRO = ['NES', 'SNES'];

export function artSets(search) {
  const q = new URLSearchParams(search);
  return {
    main: { base: q.get('art') ?? 'https://art.slot-cfw.fyi/', index: null, platforms: 'GB, GBC and GBA' },
    retro: {
      base: q.get('art_retro') ?? 'https://felipecpaiva.github.io/slot-nes-snes-art/',
      index: null,
      platforms: 'NES and SNES',
    },
  };
}

export const setFor = (sets, platform) => (RETRO.includes(platform) ? sets.retro : sets.main);

// Each set loads on its own: one that fails leaves its platforms to slot's own labels and the
// other set untouched. The failures come back as one message, since the banner holds one.
export async function loadSets(sets, fetchIndex) {
  const on = Object.values(sets).filter((s) => s.base);
  const got = await Promise.allSettled(on.map((s) => fetchIndex(s.base)));
  const failed = [];
  got.forEach((r, i) => {
    if (r.status === 'fulfilled') on[i].index = r.value;
    else failed.push(`The ${on[i].platforms} art set at ${on[i].base} didn’t load (${r.reason?.message ?? r.reason})`);
  });
  return failed.length ? `${failed.join('. ')}, so those carts get slot’s own labels.` : null;
}
