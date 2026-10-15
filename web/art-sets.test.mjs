import { test } from 'node:test';
import assert from 'node:assert/strict';
import { artSets, filedKey, filedUnder, loadSets, setFor } from './art-sets.js';

test('Game Boy and GBA art comes from Brandon’s set, NES and SNES art from slot-nes-snes’s', () => {
  const sets = artSets('');
  assert.equal(sets.main.base, 'https://art.slot-cfw.fyi/');
  assert.equal(sets.retro.base, 'https://felipecpaiva.github.io/slot-nes-snes-art/');
  for (const p of ['GB', 'GBC', 'GBA']) assert.equal(setFor(sets, p), sets.main);
  for (const p of ['NES', 'SNES']) assert.equal(setFor(sets, p), sets.retro);
});

test('each base can be pointed elsewhere, or turned off with an empty value, on its own', () => {
  const sets = artSets('?art=http://127.0.0.1:8766/&art_retro=');
  assert.equal(sets.main.base, 'http://127.0.0.1:8766/');
  assert.equal(sets.retro.base, '');
  assert.equal(artSets('?art_retro=http://x.test/').main.base, 'https://art.slot-cfw.fyi/');
});

const fake = (ok) => async (base) => {
  if (!ok.includes(base)) throw new Error('HTTP 404');
  return { [base]: {} };
};

test('a set that fails to load leaves the other one loaded, and says which failed', async () => {
  const sets = artSets('?art=http://gb.test/&art_retro=http://retro.test/');
  const message = await loadSets(sets, fake(['http://gb.test/']));
  assert.deepEqual(sets.main.index, { 'http://gb.test/': {} });
  assert.equal(sets.retro.index, null);
  assert.match(message, /NES and SNES art set at http:\/\/retro\.test\/ didn’t load \(HTTP 404\)/);
  assert.doesNotMatch(message, /GB, GBC and GBA/);

  const other = artSets('?art=http://gb.test/&art_retro=http://retro.test/');
  assert.match(await loadSets(other, fake(['http://retro.test/'])), /^The GB, GBC and GBA art set/);
  assert.deepEqual(other.retro.index, { 'http://retro.test/': {} });
});

test('both failing makes one message; an empty base is never fetched', async () => {
  const both = await loadSets(artSets('?art=http://a.test/&art_retro=http://b.test/'), fake([]));
  assert.match(both, /GB, GBC and GBA.*\. The NES and SNES/);
  const asked = [];
  const sets = artSets('?art=&art_retro=http://b.test/');
  assert.equal(await loadSets(sets, async (b) => (asked.push(b), {})), null);
  assert.deepEqual(asked, ['http://b.test/']);
  assert.equal(sets.main.index, null);
});

const INDEX = {
  AAAAAAAA: { 'support-texture': 'NES/support-texture/AAAAAAAA.png', wheel: 'NES/wheel/AAAAAAAA.png' },
  BBBBBBBB: { 'support-texture': 'GBA/support-texture/BBBBBBBB.png', look: { ground: '000000' } },
  CCCCCCCC: { wheel: 'SNES/wheel/CCCCCCCC.png' },
};

test('a game with art in the loaded set is found by any of its checksums', () => {
  assert.equal(filedKey(INDEX, 'NES', ['00000000', 'AAAAAAAA']), 'AAAAAAAA');
  assert.equal(filedKey(INDEX, 'SNES', ['CCCCCCCC']), 'CCCCCCCC', 'a logo alone is art');
  assert.equal(filedKey(INDEX, 'GBA', ['BBBBBBBB']), 'BBBBBBBB');
});

test('a game the set has nothing for, or no loaded set, has no art', () => {
  assert.equal(filedKey(INDEX, 'NES', ['00000000']), null);
  assert.equal(filedKey(INDEX, 'NES', []), null);
  assert.equal(filedKey(null, 'NES', ['AAAAAAAA']), null);
});

test('a NES or SNES game has no art from another console’s folder', () => {
  assert.equal(filedKey(INDEX, 'SNES', ['AAAAAAAA']), null);
  assert.equal(filedKey(INDEX, 'NES', ['BBBBBBBB']), null);
  assert.equal(filedUnder(INDEX, 'GB', 'AAAAAAAA'), true, 'Game Boy and GBA art has no folder rule');
});
