import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

_SPEC = importlib.util.spec_from_file_location(
    "debt_ratchet", Path(__file__).resolve().parent / "debt-ratchet.py"
)
assert _SPEC and _SPEC.loader
dr = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(dr)


class DebtRatchetTest(unittest.TestCase):
    def _make_repo(self, root: Path) -> None:
        (root / "crates/demo/src").mkdir(parents=True)
        (root / "crates/cpl-cli/tests/pending").mkdir(parents=True)
        (root / "spec/std").mkdir(parents=True)
        (root / "stdlib").mkdir(parents=True)

        # Big file (25 lines, using max_rs_lines=20 in tests)
        (root / "crates/demo/src/big.rs").write_text("\n".join(["// line"] * 25) + "\n")
        # Small file with one real #[ignore] and one commented #[ignore]
        (root / "crates/demo/src/small.rs").write_text(
            "//! doc mentioning #[ignore]\n#[ignore = \"slow\"]\nfn t() {}\n"
        )
        # stdlib var named todo is NOT a stub; spec/std `= todo` IS a stub
        (root / "stdlib/dp.cp").write_text("var todo = [1]\ntodo.push(2)\n")
        (root / "spec/std/seq.cp").write_text("pub fn foo() = todo\nfn bar():\n    todo\n")
        (root / "crates/cpl-cli/tests/pending/case1.cp").write_text("print(1)\n")

    def test_measure_counts_accurately(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._make_repo(root)
            m = dr.measure(root, max_rs_lines=20)
            self.assertEqual(m["rs_file_size_allowlist"], {"crates/demo/src/big.rs": 25})
            self.assertEqual(m["stdlib_todo"], 2)
            self.assertEqual(m["rust_ignore"], 1)
            self.assertEqual(m["pending_tests"], 1)

    def test_evaluate_fails_on_growth_and_warns_on_new_big_file(self) -> None:
        baseline = {
            "max_rs_lines": 20,
            "rs_file_size_allowlist": {"crates/demo/src/big.rs": 25},
            "stdlib_todo": 2,
            "rust_ignore": 1,
            "pending_tests": 1,
        }
        # Exact match passes with no warnings/errors
        errs, warns, notes = dr.evaluate(baseline, baseline)
        self.assertEqual((errs, warns, notes), ([], [], []))

        # Allowlisted file growing fails; non-allowlisted file > cap warns; counters growing fail
        worse = {
            "max_rs_lines": 20,
            "rs_file_size_allowlist": {
                "crates/demo/src/big.rs": 26,
                "crates/demo/src/new_big.rs": 22,
            },
            "stdlib_todo": 3,
            "rust_ignore": 2,
            "pending_tests": 2,
        }
        errs, warns, _ = dr.evaluate(worse, baseline)
        self.assertEqual(len(errs), 4)
        self.assertEqual(len(warns), 1)
        self.assertIn("new_big.rs", warns[0])

    def test_evaluate_allows_shrinking(self) -> None:
        baseline = {
            "max_rs_lines": 20,
            "rs_file_size_allowlist": {"crates/demo/src/big.rs": 25},
            "stdlib_todo": 2,
            "rust_ignore": 1,
            "pending_tests": 1,
        }
        better = {
            "max_rs_lines": 20,
            "rs_file_size_allowlist": {},
            "stdlib_todo": 1,
            "rust_ignore": 0,
            "pending_tests": 0,
        }
        errs, warns, notes = dr.evaluate(better, baseline)
        self.assertEqual(errs, [])
        self.assertEqual(warns, [])
        self.assertEqual(len(notes), 4)

    def test_cli_roundtrip_update_and_check(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._make_repo(root)
            ratchet = root / "ratchet.json"
            self.assertEqual(dr.main(["--update", "--root", str(root), "--ratchet", str(ratchet)]), 0)
            self.assertTrue(json.loads(ratchet.read_text())["stdlib_todo"] == 2)
            self.assertEqual(dr.main(["--check", "--root", str(root), "--ratchet", str(ratchet)]), 0)


if __name__ == "__main__":
    unittest.main()
