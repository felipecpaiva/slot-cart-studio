"""Which media a cart still needs. Run from tools/: python3 -m unittest test_harvest"""

import tempfile
import unittest
from pathlib import Path

import harvest


class TodoFor(unittest.TestCase):
    def test_look_is_skipped_once_recorded_unless_relooked(self):
        root = Path(tempfile.mkdtemp())
        (root / 'wheel').mkdir()
        (root / 'wheel' / 'AAAA0000.png').write_bytes(b'x')
        index = {'AAAA0000': {'look': {'ground': '000000', 'band': None, 'sure': True, 'by': 'analysis'}}}
        wanted = ['wheel', 'look']
        self.assertEqual(harvest.todo_for('AAAA0000', wanted, index, root, relook=False), [])
        self.assertEqual(harvest.todo_for('AAAA0000', wanted, index, root, relook=True), ['look'])
        self.assertEqual(harvest.todo_for('BBBB1111', wanted, index, root, relook=False), ['wheel', 'look'])



class WriteIndex(unittest.TestCase):
    def test_the_index_is_compact_and_sorted(self):
        path = Path(tempfile.mkdtemp()) / 'index.json'
        harvest.write_index(path, {'BBBB1111': {'wheel': 'GB/wheel/BBBB1111.png'}, 'AAAA0000': {'look': None}})
        self.assertEqual(
            path.read_text(),
            '{"AAAA0000":{"look":null},"BBBB1111":{"wheel":"GB/wheel/BBBB1111.png"}}',
        )


class RegionsFor(unittest.TestCase):
    def test_a_dump_prefers_its_own_region(self):
        self.assertEqual(harvest.regions_for('Metroid Fusion (USA)'), ('us', 'wor', 'eu', 'jp'))
        self.assertEqual(harvest.regions_for('Metroid Fusion (USA, Australia)'), ('us', 'wor', 'eu', 'jp'))
        self.assertEqual(harvest.regions_for('Metroid Fusion (Japan)'), ('jp', 'us', 'wor', 'eu'))
        self.assertEqual(harvest.regions_for('Metroid Fusion (Europe) (En,Fr,De,Es,It)'), ('eu', 'us', 'wor', 'jp'))
        self.assertEqual(harvest.regions_for('Pokemon - Gelbe Edition (Germany)'), ('de', 'eu', 'us', 'wor', 'jp'))
        self.assertEqual(harvest.regions_for('Tetris (World) (Rev 1)'), ('wor', 'us', 'eu', 'jp'))
        self.assertEqual(harvest.regions_for('E60EC183'), ('us', 'wor', 'eu', 'jp'))

    def test_pick_follows_the_order_it_is_given(self):
        medias = [{'type': 'wheel', 'region': 'us', 'url': 'u'}, {'type': 'wheel', 'region': 'jp', 'url': 'j'}]
        self.assertEqual(harvest.pick(medias, 'wheel')['url'], 'u')
        self.assertEqual(harvest.pick(medias, 'wheel', harvest.regions_for('X (Japan)'))['url'], 'j')
        self.assertEqual(harvest.pick(medias, 'wheel', harvest.regions_for('X (France)'))['url'], 'u')


class Redo(unittest.TestCase):
    def test_redo_asks_for_every_media_again(self):
        root = Path(tempfile.mkdtemp())
        (root / 'wheel').mkdir()
        (root / 'wheel' / 'AAAA0000.png').write_bytes(b'x')
        index = {'AAAA0000': {'look': {'ground': '000000', 'band': None, 'sure': True, 'by': 'analysis'}}}
        self.assertEqual(harvest.todo_for('AAAA0000', ['wheel', 'look'], index, root, relook=False, redo=True), ['wheel', 'look'])


class NesSnes(unittest.TestCase):
    def test_a_snes_copier_header_is_left_out_and_a_nes_rom_is_hashed_whole(self):
        import zlib
        d = Path(tempfile.mkdtemp())
        body = bytes(range(256)) * 128
        (d / 'clean.sfc').write_bytes(body)
        (d / 'copier.smc').write_bytes(b'\xaa' * 512 + body)
        want = f'{zlib.crc32(body):08X}'
        self.assertEqual(harvest.crc32(d / 'clean.sfc', 'SNES'), want)
        self.assertEqual(harvest.crc32(d / 'copier.smc', 'SNES'), want)
        nes = b'NES\x1a' + b'\x00' * 12 + body[:16384]
        (d / 'x.nes').write_bytes(nes)
        self.assertEqual(harvest.crc32(d / 'x.nes', 'NES'), f'{zlib.crc32(nes):08X}')
        # A Game Boy rom 512 past a KiB is still hashed whole, as before.
        whole = (d / 'copier.smc').read_bytes()
        self.assertEqual(harvest.crc32(d / 'copier.smc', 'GB'), f'{zlib.crc32(whole):08X}')

    def test_only_the_platforms_a_run_harvests_are_held_to_their_names(self):
        listed = {'response': {'systemes': [
            {'id': 9, 'noms': {'nom_eu': 'Game Boy'}},
            {'id': 3, 'noms': {'nom_eu': 'Something Else'}},
        ]}}
        real = harvest.call
        harvest.call = lambda endpoint, params: listed
        try:
            self.assertEqual(harvest.check_systems({}, ['GB'])['GB'], 9)
            with self.assertRaises(SystemExit):
                harvest.check_systems({}, ['NES'])
        finally:
            harvest.call = real

    def test_nes_and_snes_cards_are_read_from_their_own_folders(self):
        card = Path(tempfile.mkdtemp())
        (card / 'Games' / 'SNES').mkdir(parents=True)
        for name in ('A.sfc', 'B.smc', 'C.nes', '.D.sfc'):
            (card / 'Games' / 'SNES' / name).write_bytes(b'x')
        self.assertEqual([p.name for p in harvest.roms(card, 'SNES')], ['A.sfc', 'B.smc'])
        self.assertEqual(harvest.roms(card, 'NES'), [])

    def test_a_developer_only_login_never_asks_for_member_limits(self):
        asked = []
        real = harvest.call
        harvest.call = lambda endpoint, params: asked.append(endpoint) or {}
        try:
            q = harvest.quota({'devid': 'd', 'devpassword': 'p'})
            self.assertEqual(asked, [])
            self.assertEqual((q['per_min'], q['per_day'], q['threads']), (None, None, 1))
            harvest.quota({'devid': 'd', 'devpassword': 'p', 'ssid': 'u', 'sspassword': 'x'})
            self.assertEqual(asked, ['ssuserInfos.php'])
        finally:
            harvest.call = real


if __name__ == '__main__':
    unittest.main()
