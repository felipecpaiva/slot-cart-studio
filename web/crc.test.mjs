import { test } from 'node:test';
import assert from 'node:assert/strict';
import { crc32 } from 'node:zlib';
import { knownCrc, romCrcs } from './crc.js';

// The WASM Crc32's shape, over node's own CRC-32.
class Crc32 {
  value = 0;
  update(chunk) {
    this.value = crc32(chunk, this.value);
  }
  finish() {
    return this.value;
  }
}

// A file whose stream comes in uneven chunks, so the header's edge falls inside one.
const fileOf = (bytes, size = 700) => ({
  size: bytes.length,
  stream: () =>
    new ReadableStream({
      start(c) {
        for (let at = 0; at < bytes.length; at += size) c.enqueue(bytes.subarray(at, at + size));
        c.close();
      },
    }),
});

const rom = (n) => Uint8Array.from({ length: n }, (_, i) => (i * 7) % 251);

test('a SNES rom 512 past a whole KiB gives its headerless and its whole checksum', async () => {
  const bytes = rom(32768 + 512);
  const { crc, rawCrc } = await romCrcs(fileOf(bytes), 512, Crc32);
  assert.equal(crc, crc32(bytes.subarray(512)));
  assert.equal(rawCrc, crc32(bytes));
});

test('any other rom is hashed whole, once', async () => {
  const bytes = rom(32768);
  assert.deepEqual(await romCrcs(fileOf(bytes), 0, Crc32), { crc: crc32(bytes), rawCrc: null });
});

test('the whole-file checksum is used only when the headerless one is unknown and it is known', async () => {
  const bytes = rom(32768 + 512);
  const { crc, rawCrc } = await romCrcs(fileOf(bytes), 512, Crc32);
  const knows = (...crcs) => (c) => crcs.includes(c);
  assert.equal(knownCrc(crc, rawCrc, knows(crc, rawCrc)), crc, 'headerless first');
  assert.equal(knownCrc(crc, rawCrc, knows(rawCrc)), rawCrc, 'a dump that only happens to be that size');
  assert.equal(knownCrc(crc, rawCrc, knows()), crc, 'neither known keeps the dat’s rule');
  assert.equal(knownCrc(crc, null, knows(rawCrc)), crc);
});
