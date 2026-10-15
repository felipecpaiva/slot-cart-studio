import { test } from 'node:test';
import assert from 'node:assert/strict';
import { fetchArt, fetchThumb, logoFor } from './libretro.js';

const SET = 'https://set.test/';
const LOGOS = 'https://raw.githubusercontent.com/libretro-thumbnails/Nintendo_-_Super_Nintendo_Entertainment_System/master/Named_Logos/';
const png = (n) => new Uint8Array([0x89, 0x50, 0x4e, 0x47, n]);

// fetch over a map of url to bytes; anything else is a 404. Every url asked is kept.
function fakeFetch(files) {
  const asked = [];
  globalThis.fetch = async (url) => {
    asked.push(String(url));
    const body = files[String(url)];
    return body ? new Response(body) : new Response(null, { status: 404 });
  };
  return asked;
}

const readable = (bytes) => bytes[0] === 0x89;
const noStub = () => undefined;
const logo = (cart, path) =>
  logoFor(
    { platform: 'SNES', name: 'Star Fox (USA)', hasReal: false, ...cart },
    () => (path ? fetchArt(SET, path) : Promise.resolve(null)),
    (name) => fetchThumb(cart?.platform ?? 'SNES', 'Named_Logos', name, noStub),
    readable,
  );

test('the set’s own logo wins, and libretro is never asked', async () => {
  const asked = fakeFetch({ [`${SET}SNES/wheel/AAAAAAAA.png`]: png(1), [`${LOGOS}Star%20Fox%20(USA).png`]: png(2) });
  assert.deepEqual(await logo({}, 'SNES/wheel/AAAAAAAA.png'), png(1));
  assert.deepEqual(asked, [`${SET}SNES/wheel/AAAAAAAA.png`]);
});

test('a game the set has no art for takes libretro’s Named_Logos logo', async () => {
  const asked = fakeFetch({ [`${LOGOS}Star%20Fox%20(USA).png`]: png(2) });
  assert.deepEqual(await logo({}, null), png(2));
  assert.deepEqual(asked, [`${LOGOS}Star%20Fox%20(USA).png`]);
  assert.ok(asked.every((u) => !/Named_(Boxarts|Snaps|Titles)/.test(u)), 'only ever a logo');
});

test('an unreadable logo in the set counts as none', async () => {
  fakeFetch({ [`${SET}SNES/wheel/AAAAAAAA.png`]: new Uint8Array([0]), [`${LOGOS}Star%20Fox%20(USA).png`]: png(2) });
  assert.deepEqual(await logo({}, 'SNES/wheel/AAAAAAAA.png'), png(2));
});

test('neither having a logo leaves the cart to slot’s own label', async () => {
  fakeFetch({});
  assert.equal(await logo({}, null), null);
  fakeFetch({ [`${LOGOS}Star%20Fox%20(USA).png`]: new Uint8Array([0]) });
  assert.equal(await logo({}, null), null, 'an unreadable libretro logo is none too');
});

test('libretro is not asked for a cart with a label scan, no name, or no Named_Logos folder', async () => {
  const asked = fakeFetch({});
  assert.equal(await logo({ hasReal: true }, null), null);
  assert.equal(await logo({ name: null }, null), null);
  assert.equal(await logo({ platform: 'GBC' }, null), null);
  assert.equal(await logo({ platform: 'GB' }, null), null);
  assert.equal(await logo({ platform: 'GBA' }, null), null);
  assert.deepEqual(asked, []);
});
