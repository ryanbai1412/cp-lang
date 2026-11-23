import importlib.util
import json
import subprocess
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
        (root / "std").mkdir(parents=True)
        (root / "stdlib").mkdir(parents=True)

        # Big file (25 lines, using max_rs_lines=20 in tests)
        (root / "crates/demo/src/big.rs").write_text("\n".join(["// line"] * 25) + "\n")
        # Small file with one real #[ignore] and one commented #[ignore]
        (root / "crates/demo/src/small.rs").write_text(
            "//! doc mentioning #[ignore]\n#[ignore = \"slow\"]\nfn t() {}\n"
        )
        # stdlib var named todo is NOT a stub; std `= todo` IS a stub
        (root / "stdlib/dp.cp").write_text("var todo = [1]\ntodo.push(2)\n")
        (root / "std/seq.cp").write_text("pub fn foo() = todo\nfn bar():\n    todo\n")
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
        self.assertEqual(len(errs), 5)
        self.assertEqual(warns, [])
        self.assertTrue(any("new_big.rs" in e for e in errs))

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

    def _write(self, root: Path, rel: str, text: str) -> None:
        (root / rel).parent.mkdir(parents=True, exist_ok=True)
        (root / rel).write_text(text)

    def test_architecture_counters(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._write(root, "crates/cpl-ir/src/lower/call.rs",
                        'match n {\n    "prefix" | "sum" => 1,\n    "sum" => 2,\n    x => 3,\n}\n')
            self._write(root, "crates/cpl-codegen/src/a.rs", 'if t == "Bitset" { Edition::V02 }\n')
            self._write(root, "crates/cpl-codegen/src/tests.rs", 'let _ = "Bitset";\n')
            body = "fn helper() {\n" + "".join(f"    let x{i} = {i};\n" for i in range(6)) + "}\n"
            self._write(root, "crates/cpl-fmt/src/cli.rs", body)
            self._write(root, "crates/cpl-doc/src/cli.rs", "// copy\n" + body)
            self._write(root, "crates/cpl-lint/Cargo.toml",
                        '[package]\nname = "cpl-lint"\n[dependencies]\ncpl-check = "0"\n'
                        '[target.\'cfg(not(target_arch = "wasm32"))\'.dependencies]\ncpl-prover = "0"\n'
                        '[dev-dependencies]\ncpl-cli = "0"\n')
            self._write(root, "plan/ARCHITECTURE.md", "`crates/cpl-lint` and `crates/gone/x.rs`\n")
            self._write(root, "std/zz.cp", "pub fn rotate(self: Int) -> Int = self\n")
            self._write(root, "crates/cpl-lint/src/rules/a.rs", 'if m == "rotate" || m == "tmp" {}\n')
            self._write(root, "crates/cpl-lint/src/rules/tests.rs", 'let _ = "rotate";\n')
            self.assertEqual(dr.lint_std_names(root), ["crates/cpl-lint/src/rules/a.rs:1 rotate"])
            self.assertEqual(dr.ir_builtin_names(root), ["prefix", "sum"])
            self.assertEqual(dr.backend_std_names(root), ["crates/cpl-codegen/src/a.rs:1 Bitset"])
            self.assertEqual(dr.edition_refs(root), ["crates/cpl-codegen/src/a.rs:1"])
            self._write(root, "decisions.md", "## 9. a\n## 10. b\n## 9. c\n### 10. not a decision\n")
            self.assertEqual(dr.decision_dup_numbers(root), ["9"])
            self.assertEqual(len(dr.dup_fn_bodies(root)), 1)
            self.assertEqual(dr.crate_edges(root), ["cpl-lint -> cpl-check", "cpl-lint -> cpl-prover"])
            self.assertEqual(dr.dev_crate_edges(root), ["cpl-lint -> cpl-cli"])
            self.assertEqual(dr.layer_violations(root), ["cpl-lint -> cpl-prover"])
            self.assertEqual(dr.stale_arch_paths(root), ["plan/ARCHITECTURE.md:1 crates/gone/x.rs"])

    def test_prelude_privilege(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "crates/cpl-check/src/prelude.rs",
                        'pub const VALUE_NAMES: &[&str] = &[\n    "sum", "print",\n];\n'
                        'pub const TYPE_NAMES: &[&str] = &["Counter"];\n')
            self._write(root, "std/seq.cp", "pub fn sum(xs: List<Int>) -> Int = todo\nfn print() = todo\n")
            self._write(root, "std/bag.cp", "pub struct Counter {}\n")
            self.assertEqual(dr.prelude_privilege(root), ["Counter", "sum"])

    def test_unrun_script_tests(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "scripts/check.sh", "python3 -m unittest scripts/test_a.py\n")
            self._write(root, "scripts/test_a.py", "")
            self._write(root, "scripts/test_b.py", "")
            self.assertEqual(dr.unrun_script_tests(root), ["scripts/test_b.py"])

    def test_stale_arch_paths_skips_ignored_and_known_debt(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            # A pre-commit hook exports GIT_DIR; `git init` would then rewrite the outer repo.
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, "web/.gitignore", "out/\n")
            self._write(root, "plan/domains/x/ARCHITECTURE.md",
                        "`web/out` and `web/gone.ts`\n## Known debt\n- `scripts/deleted.py` is gone\n")
            self.assertEqual(dr.stale_arch_paths(root), ["plan/domains/x/ARCHITECTURE.md:1 web/gone.ts"])

    def test_evaluate_counters_and_edges(self) -> None:
        baseline = {"edition_refs": 3, "crate_edges": ["a -> b", "a -> c"]}
        actual = {"edition_refs": 4, "dup_fn_bodies": 9, "crate_edges": ["a -> b", "a -> d"]}
        errs, _, notes = dr.evaluate(actual, baseline)
        # dup_fn_bodies has no baseline yet, so it is not gated.
        self.assertEqual(len(errs), 2)
        self.assertIn("edition", errs[0])
        self.assertIn("a -> d", errs[1])
        self.assertEqual(notes, ["crate edge a -> c removed"])

    def test_cli_show_and_list(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._make_repo(root)
            self.assertEqual(dr.main(["--show", "rust_ignore", "--root", str(root)]), 0)
            self.assertEqual(dr.main(["--list", "stdlib_todo", "--root", str(root)]), 0)
            self.assertEqual(dr.main(["--list", "nope", "--root", str(root)]), 2)

    def test_stdlib_todo_marks_stubs_in_test_fns(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._write(root, "std/seq.cp",
                        "pub fn a() = todo\n@test\nfn t():\n    todo\npub fn b() = todo\n")
            self.assertEqual(dr.stdlib_todo(root),
                             ["std/seq.cp:1", "std/seq.cp:4 @test", "std/seq.cp:5"])

    def test_cli_update_by_name_keeps_the_rest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            ratchet = root / "ratchet.json"
            ratchet.write_text(json.dumps({"max_rs_lines": 2000, "rs_file_size_allowlist": {"a.rs": 9000},
                                           "stdlib_todo": 99, "rust_ignore": 7}))
            self.assertEqual(dr.main(["--update", "stdlib_todo", "--root", str(root), "--ratchet", str(ratchet)]), 0)
            data = json.loads(ratchet.read_text())
            self.assertEqual(data["stdlib_todo"], 0)
            self.assertEqual(data["rust_ignore"], 7)
            self.assertEqual(data["rs_file_size_allowlist"], {"a.rs": 9000})
            self.assertEqual(dr.main(["--update", "nope", "--root", str(root), "--ratchet", str(ratchet)]), 2)

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
