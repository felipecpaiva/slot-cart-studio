// A rom's checksums, kept apart from the WASM so node can test them with any Crc32.

// A SNES rom 512 past a whole KiB is taken to carry a copier header and hashed without it, as the
// No-Intro dat lists it. The whole file is hashed in the same pass, for a dump that only happens
// to be that size: `skip` is 0 for every other rom, and then there is one checksum, not two.
export async function romCrcs(file, skip, Crc32) {
  const bare = new Crc32();
  const raw = skip ? new Crc32() : null;
  const reader = file.stream().getReader();
  let at = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) return { crc: bare.finish(), rawCrc: raw ? raw.finish() : null };
    raw?.update(value);
    if (at + value.length > skip) bare.update(at >= skip ? value : value.subarray(skip - at));
    at += value.length;
  }
}

// The headerless checksum while the dat or the art set knows it, else the whole file's when that
// one is known instead. Neither known keeps the headerless one, the rule the dats are hashed by.
export const knownCrc = (crc, rawCrc, known) =>
  rawCrc !== null && !known(crc) && known(rawCrc) ? rawCrc : crc;
