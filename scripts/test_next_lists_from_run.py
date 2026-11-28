import importlib.util
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("nlfr", Path(__file__).with_name("next-lists-from-run.py"))
nlfr = importlib.util.module_from_spec(spec)
spec.loader.exec_module(nlfr)


def shard(root: Path, i: int, n: int, **lists: str) -> Path:
    d = root / f"next-lists-{i}-{n}" / f"shard-{i}-of-{n}"
    d.mkdir(parents=True)
    for name in nlfr.NAMES:
        (d / name).write_text(lists.get(name.replace(".", "_").replace("-", "_"), ""))
    return d


class Merge(unittest.TestCase):
    def test_rows_from_every_shard_in_id_byte_order(self):
        with tempfile.TemporaryDirectory() as t:
            t = Path(t)
            dirs = [shard(t, 1, 2, passing_txt="next/b.cp\ncore-01/z.cp\n", wrong_tsv="core-02/a.cp\twrong-accept\tx\n"),
                    shard(t, 2, 2, passing_txt="core-01/a_b.cp\ncore-01/a.cp\n")]
            got = nlfr.merge(dirs)
            self.assertEqual(got["passing.txt"], "core-01/a.cp\ncore-01/a_b.cp\ncore-01/z.cp\nnext/b.cp\n")
            self.assertEqual(got["wrong.tsv"], "core-02/a.cp\twrong-accept\tx\n")

    def test_a_missing_shard_fails(self):
        with tempfile.TemporaryDirectory() as t:
            with self.assertRaises(SystemExit):
                nlfr.merge([shard(Path(t), 1, 2)])

    def test_a_case_in_two_shards_fails(self):
        with tempfile.TemporaryDirectory() as t:
            t = Path(t)
            with self.assertRaises(SystemExit):
                nlfr.merge([shard(t, 1, 2, passing_txt="a.cp\n"), shard(t, 2, 2, passing_txt="a.cp\n")])

    def test_a_shard_missing_a_list_fails(self):
        with tempfile.TemporaryDirectory() as t:
            t = Path(t)
            a, b = shard(t, 1, 2), shard(t, 2, 2)
            (b / "masked.txt").unlink()
            with self.assertRaises(SystemExit):
                nlfr.merge([a, b])


if __name__ == "__main__":
    unittest.main()
