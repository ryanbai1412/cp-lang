import contextlib
import io
import importlib.util
import json
import os
import subprocess
import re
import tempfile
import unittest
from unittest import mock
from pathlib import Path

PKGS = "corpus/problems"

_SPEC = importlib.util.spec_from_file_location(
    "debt_ratchet", Path(__file__).resolve().parent / "debt-ratchet.py"
)
assert _SPEC and _SPEC.loader
dr = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(dr)


CRATE_RULES = """
### Crate rules

| Rule | Crates |
|---|---|
| back end (reads the IR) | `cpl-ir`, `cpl-interp`, `cpl-codegen` |
| consumer | `cpl-lint`, `cpl-lsp`, `cpl-cli`, `cpl-stress`, `cpl-codegen`, `cpl-wasm` |
| forbidden edge | `cpl-codegen -> cpl-check`, `cpl-lint -> cpl-prover` |
| layer order (bottom first) | `cpl-a`, `cpl-b + cpl-c`, `cpl-d` |
"""


class DebtRatchetTest(unittest.TestCase):
    def test_every_measured_by_key_names_a_counter(self) -> None:
        self.assertEqual(sorted(set(dr.MEASURED_BY) - set(dr.COUNTERS)), [])

    def _make_repo(self, root: Path) -> None:
        (root / "crates/demo/src").mkdir(parents=True)
        (root / "crates/cpl-cli/tests/pending").mkdir(parents=True)
        (root / "std").mkdir(parents=True)
        (root / "stdlib").mkdir(parents=True)

        # Big file (25 lines, using max_rs_lines=20 in tests)
        (root / "crates/demo/src/big.rs").write_text("\n".join(["x();"] * 25) + "\n")
        # Small file with one real #[ignore] and one commented #[ignore]
        (root / "crates/demo/src/small.rs").write_text(
            "//! doc mentioning #[ignore]\n#[ignore = \"slow\"]\nfn t() {}\n"
        )
        # stdlib var named todo is NOT a stub; std `= todo` IS a stub
        (root / "stdlib/dp.cp").write_text("var todo = [1]\ntodo.push(2)\n")
        # std's `@role(todo)` filler is the trap, not a stub
        (root / "std/seq.cp").write_text("pub fn foo() = todo\nfn bar():\n    todo\n@role(todo)\npub fn todo() = todo\n")
        (root / "crates/cpl-cli/tests/pending/case1.cp").write_text("print(1)\n")

    def test_corpus_test_files_counts_every_test_file(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            files = [f"{PKGS}/a/tests/sample1.in", f"{PKGS}/a/tests/1.in",
                     f"{PKGS}/a/tests/1.ans", f"{PKGS}/a/gen.py"]
            for f in files:
                (root / f).parent.mkdir(parents=True, exist_ok=True)
                (root / f).write_text("1\n")
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(["git", "-C", str(root), "add", "."], check=True)
            self.assertEqual(dr.corpus_test_files(root), [
                f"{PKGS}/a/tests/1.ans", f"{PKGS}/a/tests/1.in",
                f"{PKGS}/a/tests/sample1.in"])

    def test_corpus_no_tests_counts_packages_without_a_tests_toml(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name, files in (("a", ("meta.json", "tests.toml")), ("b", ("meta.json",)), ("c", ("notes.md",))):
                for f in files:
                    self._write(root, f"corpus/problems/{name}/{f}", "{}")
            self.assertEqual(dr.corpus_no_tests(root), [f"{PKGS}/b"])

    def test_corpus_no_statement_counts_packages_without_an_inline_sample(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            sample = '[[case]]\ninput = "1\\n"\n\n[[case]]\nseed = 1\nsize = 4\n'
            for name, files in (("a", {"meta.json": "{}", "tests.toml": sample}),
                                ("b", {"meta.json": "{}", "tests.toml": "[[case]]\nargs = [\"--max\"]\n"}),
                                ("c", {"meta.json": "{}"}), ("d", {"notes.md": ""}),
                                ("e", {"meta.json": "{}", "tests.toml": '[[case]]\ninput_file = "samples/1.in"\n'})):
                for f, text in files.items():
                    self._write(root, f"corpus/problems/{name}/{f}", text)
            self.assertEqual(dr.corpus_no_statement(root), [f"{PKGS}/b", f"{PKGS}/c"])

    def test_corpus_no_statement_skips_only_a_pinned_statement_with_no_samples(self) -> None:
        pinned = {"url": "https://example.org/t.pdf", "sha256": "ab" * 32, "samples": 0}
        variants = {"ok": pinned, "no_url": {k: v for k, v in pinned.items() if k != "url"},
                    "no_sha": {k: v for k, v in pinned.items() if k != "sha256"},
                    "no_samples": {k: v for k, v in pinned.items() if k != "samples"},
                    "has_samples": {**pinned, "samples": 2}, "empty_url": {**pinned, "url": ""},
                    "bool_samples": {**pinned, "samples": False}, "not_a_dict": "https://example.org/t.pdf"}
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name, st in variants.items():
                self._write(root, f"corpus/problems/{name}/meta.json", json.dumps({"statement": st}))
                self._write(root, f"corpus/problems/{name}/tests.toml", '[[case]]\nseed = 1\nsize = 4\n')
            self.assertEqual(dr.corpus_no_statement(root),
                             [f"corpus/problems/{n}" for n in sorted(variants) if n != "ok"])

    def test_corpus_fmt_dirty_lists_the_files_cpl_fmt_would_change(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            for f in ("a/port.cp", "a/dream.cp", "b/spec.cp", "b/gen.py"):
                (root / "corpus/problems" / f).parent.mkdir(parents=True, exist_ok=True)
                (root / "corpus/problems" / f).write_text("")
            cpl = root / "cpl"
            cpl.write_text("#!/bin/sh\nshift 3\nprintf '{\"files\": ['\nsep=''\nfor f; do\n"
                           "  c=false; case $f in *dream.cp|*spec.cp) c=true;; esac\n"
                           "  printf '%s{\"path\": \"%s\", \"changed\": %s}' \"$sep\" \"$f\" $c; sep=,\n"
                           "done\nprintf ']}'\n")
            cpl.chmod(0o755)
            with mock.patch.dict(os.environ, {"CPL": str(cpl)}):
                self.assertEqual(dr.corpus_fmt_dirty(root),
                                 [f"{PKGS}/a/dream.cp", f"{PKGS}/b/spec.cp"])
            with mock.patch.dict(os.environ, {"CPL": str(root / "missing")}):
                with self.assertRaisesRegex(dr.MeasureError, "no cpl at"):
                    dr.corpus_fmt_dirty(root)

    def test_a_package_without_a_statement_sample_fails_the_ratchet(self) -> None:
        self.assertIn("corpus_no_statement", dr.HARD_ZERO)
        errs, _, _ = dr.evaluate({"corpus_no_statement": 1}, {"corpus_no_statement": 0})
        self.assertEqual(len(errs), 1)
        self.assertIn("must stay at 0", errs[0])

    def test_corpus_no_max_counts_packages_without_a_max_mode(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            for f, text in [("a/port.cp", ""), ("a/gen.py", "if sys.argv[1:] == ['--max']:\n"),
                            ("b/port.cp", ""), ("b/gen.py", "print(1)\n"), ("c/port.cp", ""),
                            ("d/port.cp", ""), ("d/interactor.py", ""), ("e/gen.py", "")]:
                (root / "corpus/problems" / f).parent.mkdir(parents=True, exist_ok=True)
                (root / "corpus/problems" / f).write_text(text)
            self.assertEqual(dr.corpus_no_max(root), [f"{PKGS}/b", f"{PKGS}/c"])

    def test_corpus_no_limits_counts_ports_without_a_limits_line(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            for f, text in [("a/port.cp", "// Source: x\n// limits: 2000 ms, 256 MB\n"),
                            ("b/port.cp", "// Source: x\n"), ("c/port.cp", "// limits: soon\n"),
                            ("d/dream.cp", "")]:
                (root / "corpus/problems" / f).parent.mkdir(parents=True, exist_ok=True)
                (root / "corpus/problems" / f).write_text(text)
            self.assertEqual(dr.corpus_no_limits(root), [f"{PKGS}/b", f"{PKGS}/c"])

    def test_v02_port_files_counts_ports_not_in_the_v03_set(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            for f in [f"{PKGS}/a/port.cp", f"{PKGS}/b/port.cp",
                      f"{PKGS}/c/spec.cp", "corpus/sets/v03.txt"]:
                (root / f).parent.mkdir(parents=True, exist_ok=True)
                (root / f).write_text("b\n")
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(["git", "-C", str(root), "add", "."], check=True)
            self.assertEqual(dr.v02_port_files(root), [f"{PKGS}/a/port.cp"])

    def test_ports_unreported_needs_both_friction_and_review(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            for f in [f"{PKGS}/a/port.cp", f"{PKGS}/b/port.cp", f"{PKGS}/c/port.cp", f"{PKGS}/d/spec.cp",
                      "research/frictions/a.md", "research/reviews/a.md", "research/reviews/b.md"]:
                (root / f).parent.mkdir(parents=True, exist_ok=True)
                (root / f).write_text("b\n")
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(["git", "-C", str(root), "add", "."], check=True)
            self.assertEqual(dr.ports_unreported(root), [f"{PKGS}/b/port.cp", f"{PKGS}/c/port.cp"])

    def test_a_port_without_a_report_fails_the_ratchet(self) -> None:
        self.assertIn("ports_unreported", dr.HARD_ZERO)
        errs, _, _ = dr.evaluate({"ports_unreported": 1}, {"ports_unreported": 0})
        self.assertEqual(len(errs), 1)
        self.assertIn("must stay at 0", errs[0])

    def test_reviews_uncapped_counts_a_high_score_without_a_dream(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            files = {"corpus/problems/a/port.cp": "", "corpus/problems/b/port.cp": "",
                     "corpus/problems/b/dream.cp": "", "corpus/problems/c/port.cp": "",
                     "corpus/problems/d/port.cp": "",
                     "research/frictions/d.md": "# d\n\n## Dream\n\n```cpl\n```\n",
                     "research/reviews/d.md": "# d\n\nScore: 4/5\nDream: 4/5\n",
                     "research/reviews/a.md": "# a\n\nScore: 4/5\nDream: none\n",
                     "research/reviews/b.md": "# b\n\nScore: 5/5\nDream: 5/5\n",
                     "research/reviews/c.md": "# c\n\nScore: 3/5\n\nReads as a 5.\n",
                     "research/reviews/README.md": "Score: 5/5\n"}
            for f, text in files.items():
                (root / f).parent.mkdir(parents=True, exist_ok=True)
                (root / f).write_text(text)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(["git", "-C", str(root), "add", "."], check=True)
            self.assertEqual(dr.reviews_uncapped(root), ["research/reviews/a.md"])

    def test_reviews_pending_counts_only_an_unscored_port_line(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            files = {"research/reviews/a.md": "# a\n\nScore: pending\nDream: 4/5\n",
                     "research/reviews/b.md": "# b\n\nScore: 4/5\nDream: pending\n",
                     "research/reviews/c.md": "# c\n\nScore: 3/5\n\nWas `Score: pending` once.\n"}
            for f, text in files.items():
                (root / f).parent.mkdir(parents=True, exist_ok=True)
                (root / f).write_text(text)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(["git", "-C", str(root), "add", "."], check=True)
            self.assertEqual(dr.reviews_pending(root), ["research/reviews/a.md"])

    def test_dead_code_allows_and_external_crates(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / "crates/demo/src").mkdir(parents=True)
            (root / "crates/demo/src/lib.rs").write_text(
                "#[allow(dead_code)]\nfn a() {}\n#![allow(unused, dead_code)]\n// dead_code\n")
            (root / "Cargo.lock").write_text(
                '[[package]]\nname = "cpl-demo"\nversion = "0.1.0"\n\n'
                '[[package]]\nname = "shlex"\nversion = "1.3.0"\n'
                'source = "registry+https://github.com/rust-lang/crates.io-index"\n')
            self.assertEqual(dr.dead_code_allows(root), ["crates/demo/src/lib.rs:1", "crates/demo/src/lib.rs:3"])
            self.assertIn("dead_code_allows", dr.HARD_ZERO)
            errs, _, _ = dr.evaluate({"dead_code_allows": 1}, {"dead_code_allows": 1})
            self.assertTrue(any("allow(dead_code)" in e for e in errs), errs)
            self.assertEqual(dr.external_crates(root), ["shlex 1.3.0"])

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
            "ir_reinfer": 2,
            "allow_attrs": 1,
            "unsafe_blocks": 1,
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
            "ir_reinfer": 3,
            "allow_attrs": 2,
            "unsafe_blocks": 2,
        }
        errs, warns, _ = dr.evaluate(worse, baseline)
        self.assertEqual(len(errs), 5)
        self.assertEqual(warns, [])
        self.assertTrue(any("new_big.rs" in e for e in errs))

    def test_evaluate_allows_shrinking(self) -> None:
        baseline = {
            "max_rs_lines": 20,
            "rs_file_size_allowlist": {"crates/demo/src/big.rs": 25},
            "ir_reinfer": 2,
            "rust_ignore": 1,
            "pending_tests": 1,
        }
        better = {
            "max_rs_lines": 20,
            "rs_file_size_allowlist": {},
            "ir_reinfer": 1,
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

    def test_next_snapshot_keyed_queries_counts_a_query_running_the_whole_pipeline(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._write(root, "crates/cpl-next/src/facade.rs", "struct PerFile {\n    front: OnceLock<Front>,\n}\n")
            self._write(root, "crates/cpl-next/src/linked.rs",
                        "impl Query for Linked {\n    fn compute() {\n        let b = compile(&std);\n    }\n}\n"
                        "fn other() {\n    compile(&std);\n}\n")
            self.assertEqual(dr.next_snapshot_keyed_queries(root),
                             ["crates/cpl-next/src/facade.rs:2", "crates/cpl-next/src/linked.rs:3"])

    def test_link_reads_check_counts_the_linkers_typed_tree_reads(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._write(root, "crates/cpl-next-link/src/lib.rs",
                        "use cpl_next_check::Typed;\nfn f(p: &Program) {}\nlet g = typed.globals;\n")
            self.assertEqual(dr.link_reads_check(root),
                             ["crates/cpl-next-link/src/lib.rs:1", "crates/cpl-next-link/src/lib.rs:3"])

    def test_link_reads_check_follows_checker_data_through_holders(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._write(root, "crates/cpl-next-decls/src/views.rs",
                        "pub struct Answers<'a> {\n    pub whole: &'a Whole,\n}\n")
            self._write(root, "crates/cpl-next-link/src/lib.rs",
                        "fn a(adds: &SetAdds) {}\nfn b(c: &dyn Checked) {}\nstruct L<'a> {\n    x: &'a views::Answers<'a>,\n}\n"
                        "fn c(d: Deferred) {}\n")
            self.assertEqual(dr.link_reads_check(root), [f"crates/cpl-next-link/src/lib.rs:{i}" for i in (1, 4, 6)])

    def test_frozen_std_drift_is_any_change_to_the_pinned_copy(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            env = dr._git_env()
            git = lambda *a: subprocess.run(["git", *a], cwd=root, check=True, env=env,
                                            capture_output=True, text=True).stdout.strip()
            git("init", "-q")
            self.assertEqual(dr.frozen_std_drift(root, "0" * 40), [])
            self._write(root, f"{dr.FROZEN_STD}/a.cp", "pub fn a() = 1\n")
            self._write(root, f"{dr.FROZEN_STD}/b.cp", "pub fn b() = 2\n")
            git("add", "-A")
            pin = git("rev-parse", f"{git('write-tree')}:{dr.FROZEN_STD}")
            self.assertEqual(dr.frozen_std_drift(root, pin), [])
            self._write(root, f"{dr.FROZEN_STD}/a.cp", "pub fn a() = 3\n")
            self.assertEqual(dr.frozen_std_drift(root, pin), [f"{dr.FROZEN_STD}/a.cp: AM"])
            git("add", "-A")
            self.assertEqual(dr.frozen_std_drift(root, pin), [f"{dr.FROZEN_STD}/a.cp: differs from the pin"])
            self._write(root, f"{dr.FROZEN_STD}/a.cp", "pub fn a() = 1\n")
            self._write(root, f"{dr.FROZEN_STD}/c.cp", "")
            git("add", f"{dr.FROZEN_STD}/a.cp")
            self.assertEqual(dr.frozen_std_drift(root, pin), [f"{dr.FROZEN_STD}/c.cp: ??"])

    def test_name_compares_count_each_form(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._write(root, "crates/cpl-next-check/src/lib.rs",
                        'if n == "todo" {}\n// n == "x"\n"Some" => 1,\n| "a" | "b" => 2,\nmatches!(k, Name(n) if n == "on")\nif s.eq("print") {}\nlet m = "text";\nif a == b {}\n')
            self.assertEqual(dr.next_name_compares(root), [f"crates/cpl-next-check/src/lib.rs:{i}" for i in (1, 3, 4, 5, 6)])

    def test_library_gates_count_each_call_and_its_wrapper(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._write(root, "crates/cpl-next-decls/src/lib.rs",
                        "fn items_only(m: M) -> bool { m.is_library() }\n// is_library\nif items_only(m) {}\nlet x = 1;\n")
            self.assertEqual(dr.next_library_gates(root), ["crates/cpl-next-decls/src/lib.rs:1",
                                                           "crates/cpl-next-decls/src/lib.rs:3"])

    def test_crate_deps_see_renamed_and_table_dependencies(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._write(root, "crates/cpl-next-link/Cargo.toml",
                        "[dependencies]\nchk = { package = \"cpl-next-check\", path = \"x\" }\n"
                        "[dependencies.cpl-next-ir]\npath = \"y\"\n"
                        "[dev-dependencies.cpl-next-own]\npath = \"z\"\n")
            self.assertEqual(dr._crate_deps(root)["cpl-next-link"], ["cpl-next-check", "cpl-next-ir"])

    def test_id_sets_read_a_fresh_run_over_the_committed_list(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            self._write(root, "crates/cpl-next/tests/wrong.tsv", "a.cp\twrong-accept\tx\nb.cp\tsplit\ty\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "x", "--no-verify"],
                           cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.next_wrong_accepts(root), ["a.cp"])
            run = root / "target/cpl-next/wrong.tsv"
            run.parent.mkdir(parents=True)
            run.write_text("c.cp\twrong-accept\tx\n")
            self.assertEqual((dr.next_wrong_accepts(root), dr.next_split(root)), (["c.cp"], []))
            os.utime(run, (0, 0))
            self.assertEqual(dr.next_wrong_accepts(root), ["a.cp"])

    def test_a_case_on_two_lists_is_counted_once_per_extra_list(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._write(root, "crates/cpl-next/tests/passing.txt", "a.cp\nb.cp\n")
            self._write(root, "crates/cpl-next/tests/wrong.tsv", "b.cp\twrong-output\tx\nc.cp\twrong-accept\ty\n")
            self._write(root, "crates/cpl-next/tests/masked.txt", "c.cp\n")
            self.assertEqual(dr.next_listed_twice(root),
                             ["b.cp: passing.txt and wrong.tsv", "c.cp: wrong.tsv and masked.txt"])
            self.assertEqual(dr.next_wrong_outputs(root), ["b.cp"])
            self.assertIn("next_listed_twice", dr.HARD_ZERO)
            self.assertIn("next_wrong_outputs_ids", dr.ID_SETS)

    def test_link_reads_check_follows_the_dependency_graph(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._write(root, "crates/cpl-next-link/Cargo.toml",
                        "[package]\nname = 1\n[dependencies]\ncpl-next-mid = { path = \"x\" }\n"
                        "[dev-dependencies]\ncpl-next-check = { path = \"y\" }\n")
            self._write(root, "crates/cpl-next-mid/Cargo.toml", "[dependencies]\ncpl-next-check.workspace = true\n")
            self.assertEqual(dr.link_reads_check(root), ["crates/cpl-next-link/Cargo.toml: reaches the checker via "
                                                         "cpl-next-link -> cpl-next-mid -> cpl-next-check"])
            self._write(root, "crates/cpl-next-mid/Cargo.toml", "[dev-dependencies]\ncpl-next-check.workspace = true\n")
            self.assertEqual(dr.link_reads_check(root), [])

    def test_crate_rules_come_from_architecture_md(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self.assertEqual(dr.crate_rules(root), {})
            self._write(root, "handbook/design/architecture.md", CRATE_RULES)
            self.assertEqual(dr.crate_rules(root)["back end"], ["cpl-ir", "cpl-interp", "cpl-codegen"])
            self.assertIn(("cpl-lint", "cpl-prover"), dr.forbidden_edges(root))
            layered = {e for e in dr.forbidden_edges(root) if e[0].startswith("cpl-") and len(e[0]) == 5}
            self.assertEqual(layered, {("cpl-a", "cpl-b"), ("cpl-a", "cpl-c"), ("cpl-a", "cpl-d"),
                                       ("cpl-b", "cpl-c"), ("cpl-c", "cpl-b"), ("cpl-b", "cpl-d"),
                                       ("cpl-c", "cpl-d")})
            self._write(root, "handbook/design/architecture.md", "# no rules\n")
            with self.assertRaises(SystemExit):
                dr.crate_rules(root)
        real = dr.crate_rules(dr.ROOT)
        self.assertTrue(real["back end"] and real["consumer"] and real["forbidden edge"])

    def test_stale_arch_paths_reads_the_ratchets_own_crate_paths(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._write(root, "scripts/debt-ratchet.py", 'X = ("crates/a/src/gone.rs", "crates/**/*.rs")\n'
                        '"crates/b/old.txt": ("c", "o", "deleted: a test fails instead"),\n')
            self.assertEqual(dr.stale_arch_paths(root), ["scripts/debt-ratchet.py:1 crates/a/src/gone.rs"])

    def test_a_wrong_accept_swap_fails_though_the_count_holds(self) -> None:
        base = {"next_wrong_accepts": 2, "next_wrong_accepts_ids": ["a.cp", "b.cp"]}
        actual = {"next_wrong_accepts": 2, "next_wrong_accepts_ids": ["a.cp", "c.cp"]}
        errors, _, notes = dr.evaluate(actual, base)
        self.assertEqual([e.split(":")[0] for e in errors], ["c.cp"])
        self.assertIn("b.cp: left next_wrong_accepts_ids", notes)
        self.assertIn("next_wrong_accepts_ids", dr.improved({"next_wrong_accepts_ids": ["a.cp"]}, base))

    def test_next_split_counts_runs_that_disagree(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._write(root, dr.NEXT_TESTS + "/wrong.tsv",
                        "a.cp\tsplit\tC++: exit 1\nb.cp\twrong-reject\tsplit here\nc.cp\tsplit\tx\n")
            self.assertEqual(dr.next_split(root), ["a.cp", "c.cp"])

    def test_frozen_edits_flags_each_commit_adding_old_pipeline_lines(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            env = dr._git_env()
            git = lambda *a: subprocess.run(["git", *a], cwd=root, check=True, env=env, capture_output=True)
            git("init", "-q")
            self._write(root, "crates/cpl-check/src/a.rs", "fn a() {}\nfn b() {}\nfn c() {}\n")
            git("add", "-A"); git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "base")
            base = subprocess.run(["git", "rev-parse", "HEAD"], cwd=root, env=env,
                                  capture_output=True, text=True).stdout.strip()
            # A net-negative fix: the line count falls, the freeze is still broken.
            self._write(root, "crates/cpl-check/src/a.rs", "fn a() { fixed() }\n")
            git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qam", "check: fix a")
            self._write(root, "crates/cpl-check/src/a.rs", "fn a() { fixed(); std() }\n")
            git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qam",
                "check: read std's new marker\n\nFrozen-exception: old checker reads std/\n"
                "Lane: c\nFrozen-exception-signed-by: g")
            self._write(root, "crates/cpl-check/src/a.rs", "fn a() { fixed(); std(); more() }\n")
            git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qam",
                "check: read more\n\nFrozen-exception: old checker reads std/\n"
                "Lane: c\nFrozen-exception-signed-by: c")
            self._write(root, "crates/cpl-check/src/a.rs", "")
            git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qam", "check: delete a")
            # Putting the file back as it was at the pin adds lines and keeps the freeze.
            self._write(root, "crates/cpl-check/src/a.rs", "fn a() {}\nfn b() {}\nfn c() {}\n")
            git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qam", "revert(check): to the pin")
            # Vendoring the pinned std may fit the crates to it, once.
            self._write(root, dr.FROZEN_STD + "/seq.cp", "pub fn f() = 0\n")
            self._write(root, "crates/cpl-check/src/a.rs", "fn a() {}\nfn b() {}\nfn c() {}\nfn pinned() {}\n")
            git("add", "-A"); git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "check: pin std")
            self._write(root, "crates/cpl-check/src/a.rs", "fn a() {}\nfn b() {}\nfn c() {}\nfn pinned() {}\nfn d() {}\n")
            self._write(root, dr.FROZEN_STD + "/seq.cp", "pub fn f() = 0\npub fn g() = 0\n")
            git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qam", "check: after the pin")
            with mock.patch.object(dr, "FROZEN_PIN", base):
                got = dr.frozen_edits(root, base)
            # No trailer excuses a commit, whoever signs it.
            self.assertEqual(len(got), 4)
            self.assertTrue(all("adds 1 lines to the frozen old pipeline" in g for g in got[1:]))

    def test_skipped_ratchet_flags_new_pipeline_commits_that_skip_it(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            env = dr._git_env()
            git = lambda *a: subprocess.run(["git", *a], cwd=root, check=True, env=env, capture_output=True)
            commit = lambda m: git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qam", m)
            git("init", "-q")
            self._write(root, "crates/cpl-next-check/src/a.rs", "fn a() {}\n")
            self._write(root, "docs/a.md", "a\n")
            git("add", "-A"); commit("base")
            base = subprocess.run(["git", "rev-parse", "HEAD"], cwd=root, env=env,
                                  capture_output=True, text=True).stdout.strip()
            self._write(root, "crates/cpl-next-check/src/a.rs", "fn a() { b() }\n")
            commit("check(next): b\n\nSkipped: ratchet, it is slow")
            self._write(root, "crates/cpl-next-check/src/a.rs", "fn a() { c() }\n")
            commit("check(next): c\n\nSkipped: full suite and list regeneration")
            self._write(root, "crates/cpl-next-check/src/a.rs", "fn a() { d() }\n")
            commit("check(next): d\n\nSkipped: handbook edit, none needed")
            self._write(root, "docs/a.md", "b\n")
            commit("docs: b\n\nSkipped: ratchet, docs only")
            got = dr.skipped_ratchet(root, base)
            self.assertEqual(len(got), 2)
            self.assertTrue(all("skips the ratchet" in g for g in got))

    def _repo(self, root: Path):
        env = dr._git_env()
        git = lambda *a: subprocess.run(["git", *a], cwd=root, check=True, env=env, capture_output=True)
        commit = lambda m: git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qam", m)
        head = lambda: subprocess.run(["git", "rev-parse", "HEAD"], cwd=root, env=env, check=True,
                                      capture_output=True, text=True).stdout.strip()
        git("init", "-q")
        return git, commit, head

    def test_unlaned_flags_new_pipeline_and_std_commits_with_no_lane(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            git, commit, head = self._repo(root)
            self._write(root, "crates/cpl-next-check/src/a.rs", "fn a() {}\n")
            self._write(root, "std/a.cp", "fn a() = 1\n")
            self._write(root, "docs/a.md", "a\n")
            git("add", "-A"); commit("base")
            base = head()
            self._write(root, "crates/cpl-next-check/src/a.rs", "fn a() { b() }\n")
            commit("check(next): b\n\nDomain: compiler")
            self._write(root, "std/a.cp", "fn a() = 2\n")
            commit("stdlib: a is 2\n\nDomain: stdlib")
            self._write(root, "std/a.cp", "fn a() = 3\n")
            commit("stdlib: a is 3\n\nDomain: stdlib\nLane: B")
            self._write(root, "docs/a.md", "b\n")
            commit("docs: b\n\nDomain: docs")
            got = dr.unlaned(root, base)
            self.assertEqual(len(got), 2, got)
            self.assertTrue(all("no `Lane:` trailer" in g for g in got))

    def test_lane_for_signed_by_the_coordinator_gives_a_pushed_commit_its_lane(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            git, commit, head = self._repo(root)
            self._write(root, "std/a.cp", "fn a() = 1\n")
            git("add", "-A"); commit("base")
            base = head()
            self._write(root, "std/a.cp", "fn a() = 2\n")
            commit("stdlib: a is 2\n\nDomain: stdlib")
            given = head()
            self._write(root, "std/a.cp", "fn a() = 3\n")
            commit("stdlib: a is 3\n\nDomain: stdlib")
            other = head()
            self._write(root, "std/a.cp", "fn a() = 4\n")
            commit(f"stdlib: a is 4\n\nDomain: stdlib\nLane: B\n"
                   f"Lane-for: {given[:9]} D, signed off by coordinator\n"
                   f"Lane-for: {other[:9]} D, signed off by B")
            got = dr.unlaned(root, base)
            self.assertEqual(len(got), 2, got)
            self.assertTrue(any("signed off by coordinator`" in g and "signed off by B" in g for g in got), got)
            self.assertTrue(any(g.startswith(other[:9]) and "no `Lane:` trailer" in g for g in got), got)
            self.assertFalse(any(g.startswith(given[:9]) for g in got), got)

    def test_a_run_measured_raise_names_a_full_run_and_its_commit(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            git, commit, head = self._repo(root)
            self._write(root, "a.txt", "a\n")
            git("add", "-A"); commit("base")
            base = head()
            self._write(root, "a.txt", "b\n")
            commit("check(next): b\n\nRaise: next_error_typed 10 -> 116: read only next/, "
                   "CPL_NEXT_ONLY=next/ run target/cpl-next/only/error-typed.txt at " + base)
            self.assertIsNotNone(dr.raise_evidence(root, base, "next_error_typed"))
            self._write(root, "a.txt", "c\n")
            commit("check(next): c\n\nRaise: next_error_typed 10 -> 116: the first count read next/")
            self.assertIsNotNone(dr.raise_evidence(root, base, "next_error_typed"))
            self._write(root, "a.txt", "d\n")
            commit("check(next): d\n\nRaise: next_error_typed 10 -> 116: full run "
                   "target/cpl-next/error-typed.txt at " + base[:9])
            self.assertIsNone(dr.raise_evidence(root, base, "next_error_typed"))
            self.assertIsNotNone(dr.raise_evidence(root, base, "next_split"))

    def test_std_fn_copies_count_as_duplicates(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            self._write(root, "std/a.cp", "\n".join([
                "pub fn drop_while<S: Seq>(self: S,",
                "        keep: T -> Bool = x => x == 0) -> S:",
                "    // skip the prefix",
                "    let i = self.first(x => !keep(x)) ?? self.len",
                "    self[i..]",
                "",
                "pub fn skip_while<S: Seq>(self: S, keep: T -> Bool) -> S:",
                "    let i = self.first(x => !keep(x)) ?? self.len",
                "    self[i..]",
                "pub fn rank(self: M) -> Int = self.rows.count(r => r.any(x => x != 0))",
                "pub fn rank_real(self: M) -> Int = self.rows.count(r => r.any(x => x.abs() > eps))",
                "pub fn short(self: M) -> Int = 1",
                "pub fn tiny(self: M) -> Int = 1", ""]))
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual([n for _, _, n, _ in dr._cp_fn_bodies(root)],
                             ["drop_while", "skip_while", "rank", "rank_real", "short", "tiny"])
            self.assertEqual(dr._cp_dup_bodies(root), ["std/a.cp:7 skip_while = std/a.cp:1 drop_while"])
            self.assertEqual(dr._cp_near_dups(root), ["std/a.cp:11 rank_real ~ std/a.cp:10 rank"])

    def test_rust_idiom_lints_reads_the_record_diff(self) -> None:
        one_span = ("let a = [1]\nlet n = a.len\nprint(n)\n", "let a = [1]\nprint(a.len)\n")
        two_findings = ("let a = (x ?? 0) > 1\nprint(1)\nlet b = (y ?? 0) > 2\n",
                        "let a = x.any(v => v > 1)\nprint(1)\nlet b = y.any(v => v > 2)\n")
        two_places = ("fn f(u: Int) -> Int:\n    u\nlet _ = f(2)\n", "fn f(u: Int):\n    u\nf(2)\n")
        leaves_block = ("for x in a:\n    h.push(-x)\nprint(-h.pop())\n", "for x in a:\n    h.push(x)\nprint(h.pop())\n")
        shared_binding = ("let a: Int = 1\nprint(1)\nlet b: Int = 2\n",
                          "type Z = Mod\nlet a: Z = 1\nprint(1)\nlet b: Z = 2\n")
        shared_const = ("print(1)\nprint(x % 7_000)\n", "const P = 7_000\nprint(1)\nprint(x % P)\n")
        self.assertTrue(dr.idiom_edit_stateable(*one_span))
        self.assertTrue(dr.idiom_edit_stateable(*two_findings))
        self.assertFalse(dr.idiom_edit_stateable(*leaves_block))
        self.assertFalse(dr.idiom_edit_stateable(*shared_binding))
        self.assertFalse(dr.idiom_edit_stateable(*shared_const))
        self.assertTrue(dr.idiom_edit_stateable(*two_places))
        self.assertFalse(dr.idiom_edit_stateable("x\n", "x\n"))
        adds_directive = ("let t: Int = read()\nfor _ in 0..t:\n    print(1)\n", "@!multitest\nprint(1)\n")
        self.assertFalse(dr.idiom_edit_stateable(*adds_directive))
        core_only = ("if k in m:\n    m[k] += v\nelse:\n    m[k] = v\nif x >= 0 and x < n:\n    y = 1\n",
                     "m[k] += v\nif 0 <= x < n:\n    y = 1\n")
        self.assertFalse(dr.idiom_edit_stateable(*core_only))
        self.assertTrue(dr.idiom_edit_stateable("print(sum(xs))\n", "print(xs.sum())\n"))
        self.assertFalse(dr.idiom_edit_stateable("for i in 0..n:\n    s += a[i]\n", "for x in 0..n:\n    s += x\n"))
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            for name, group, (before, after) in [("in_rust", "idiom", one_span), ("in_std", "idiom", one_span),
                                                 ("style", "style", one_span), ("hard", "idiom", leaves_block),
                                                 ("core", "idiom", core_only)]:
                self._write(root, f"spec/lints/{name}/lint.toml", f'group = "{group}"\n')
                self._write(root, f"spec/lints/{name}/before.cp", before)
                self._write(root, f"spec/lints/{name}/after.cp", after)
            self._write(root, "std/seq.cp", '@lint(in_std, from: "x.len", to: "n")\npub fn n(x: Int) -> Int = x\n')
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.rust_idiom_lints(root), ["spec/lints/in_rust"])

    def test_direct_python_reads_every_script_shell(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            self._write(root, "justfile", "a:\n    python3 scripts/a.py\n    uv run python scripts/b.py\n")
            self._write(root, "scripts/release/b.sh", "# python3 x.py\nx=$(cat f | python3 -c 1)\n")
            self._write(root, "web/c.sh", "python3 c.py\n")
            self._write(root, ".github/workflows/w.yml", "    steps:\n      - run: python3 d.py\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.direct_python(root), ["justfile:2", ".github/workflows/w.yml:2",
                                                      "scripts/release/b.sh:2"])

    def test_architecture_counters(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._write(root, "crates/cpl-ir/src/lower/call.rs",
                        'match n {\n    "prefix" | "sum" => 1,\n    "sum" => 2,\n    x => 3,\n}\n')
            self._write(root, "crates/cpl-ir/src/lower/call/fold.rs", 'match n {\n    "min" => 1,\n}\n')
            self._write(root, "crates/cpl-codegen/src/a.rs", 'if t == "Bitset" { Edition::V02 }\n')
            self._write(root, "crates/cpl-codegen/src/tests.rs", 'let _ = "Bitset";\n')
            body = "fn helper() {\n" + "".join(f"    let x{i} = {i};\n" for i in range(6)) + "}\n"
            self._write(root, "crates/cpl-fmt/src/cli.rs", body)
            self._write(root, "crates/cpl-doc/src/cli.rs", "// copy\n" + body)
            self._write(root, "crates/cpl-lint/Cargo.toml",
                        '[package]\nname = "cpl-lint"\n[dependencies]\ncpl-check = "0"\n'
                        '[target.\'cfg(not(target_arch = "wasm32"))\'.dependencies]\ncpl-prover = "0"\n'
                        '[dev-dependencies]\ncpl-cli = "0"\n')
            self._write(root, "handbook/design/architecture.md", "`crates/cpl-lint` and `crates/gone/x.rs`\n" + CRATE_RULES)
            self._write(root, "std/zz.cp", "pub fn rotate(self: Int) -> Int = self\n")
            self._write(root, "crates/cpl-lint/src/rules/a.rs", 'if m == "rotate" || m == "tmp" {}\n')
            self._write(root, "crates/cpl-lint/src/rules/tests.rs", 'let _ = "rotate";\n')
            self.assertEqual(dr.lint_std_names(root), ["crates/cpl-lint/src/rules/a.rs:1 rotate"])
            self.assertEqual(dr.ir_builtin_names(root), ["min", "prefix", "sum"])
            self.assertEqual(dr.backend_std_names(root), ["crates/cpl-codegen/src/a.rs:1 Bitset"])
            self.assertEqual(dr.edition_refs(root), ["crates/cpl-codegen/src/a.rs:1"])
            self._write(root, "decisions.md", "## 9. a\n## 10. b\n## 9. c\n### 10. not a decision\n")
            self.assertEqual(dr.decision_dup_numbers(root), ["9"])
            self.assertEqual(len(dr.dup_fn_bodies(root)), 1)
            self.assertEqual(dr.crate_edges(root), ["cpl-lint -> cpl-check", "cpl-lint -> cpl-prover"])
            self.assertEqual(dr.dev_crate_edges(root), ["cpl-lint -> cpl-cli"])
            self.assertEqual(dr.layer_violations(root), ["cpl-lint -> cpl-prover"])
            self.assertEqual(dr.stale_arch_paths(root), ["handbook/design/architecture.md:1 crates/gone/x.rs"])

    def test_prelude_privilege(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "crates/cpl-check/src/prelude.rs",
                        'pub const VALUE_NAMES: &[&str] = &[\n    "sum", "print",\n];\n'
                        'pub const TYPE_NAMES: &[&str] = &["Counter"];\n')
            self._write(root, "std/seq.cp", "pub fn sum(xs: List<Int>) -> Int = todo\nfn print() = todo\n")
            self._write(root, "std/bag.cp", "pub struct Counter {}\n")
            self.assertEqual(dr.prelude_privilege(root), ["Counter", "sum"])

    def test_old_crate_counters_read_the_std_they_read(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "crates/cpl-check/src/prelude.rs", 'pub const VALUE_NAMES: &[&str] = &["sum", "fold"];\n')
            self._write(root, "std/seq.cp", "pub fn sum(xs: List<Int>) -> Int = 0\npub fn fold() = 0\n")
            self.assertEqual(dr.prelude_privilege(root), ["fold", "sum"])
            self._write(root, dr.FROZEN_STD + "/seq.cp", "pub fn sum(xs: List<Int>) -> Int = 0\n")
            self.assertEqual(dr.prelude_privilege(root), ["sum"])

    def test_domain_member_names(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "std/subset.cp", "@member(subsets)\npub struct Subset<B>(bits: U64):\n")
            self._write(root, "crates/cpl-ir/src/a.rs",
                        '// "subsets" in a comment\nif matches!(m, "cuts" | "subsets") {}\n')
            self.assertEqual(dr.domain_member_names(root), ["crates/cpl-ir/src/a.rs:2 subsets"])

    def test_cfg_test_skips_only_the_item_it_gates(self) -> None:
        src = ["a.unwrap();", "#[cfg(test)]", "#[allow(x)]", "fn t() {", '    let s = "}";', "    t.unwrap();",
               "}", "b.unwrap();", "#[cfg(test)]", "mod helpers;", "c.unwrap();", "#[cfg(test)]",
               "mod tests {", "    fn x() { y.unwrap(); }", "}"]
        self.assertEqual([i for i, _ in dr._ungated_lines(src)], [1, 8, 11])

    def test_timeout_passes_finds_a_timeout_that_ends_in_success(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            self._write(root, "scripts/land.sh", 'case "$rc" in\n    124) echo slow ;;\n    124) exit 1 ;;\nesac\n')
            self._write(root, "scripts/run.py", "try:\n    f()\nexcept subprocess.TimeoutExpired:\n    return None\n"
                        "try:\n    f()\nexcept TimeoutExpired:\n    return 'timed out'\n"
                        "try:\n    f()\nexcept (subprocess.TimeoutExpired, OSError):\n    pass\n"
                        "try:\n    f()\nexcept subprocess.TimeoutExpired:\n    return None  # timeout fails: g reports it\n")
            self._write(root, "scripts/test_run.py", "try:\n    f()\nexcept subprocess.TimeoutExpired:\n    pass\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.timeout_passes(root), ["scripts/land.sh:2", "scripts/run.py:12", "scripts/run.py:4"])

    def test_ci_retries_spares_only_the_helper(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            self._write(root, "scripts/retry.sh", "retry() {\n")
            self._write(root, "scripts/r.py", "for _ in range(tries):\n")
            self._write(root, ".github/workflows/x.yml", "nextest run --retries 1 --flaky-result fail\n"
                        "for try in 1 2 3; do\n  try() { x; }\nnextest run --retries 2\n"
                        "for wait in 0 30 90; do\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.ci_retries(root), [".github/workflows/x.yml:1", ".github/workflows/x.yml:2", ".github/workflows/x.yml:3",
                                                   ".github/workflows/x.yml:4", ".github/workflows/x.yml:5", "scripts/r.py:1"])

    def test_ci_jobs_without_recipe_sees_gradle(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            self._write(root, ".github/workflows/ci.yml", "jobs:\n  smoke:\n    steps:\n      - run: ./gradlew --no-daemon test\n"
                        "  ok:\n    steps:\n      - run: just jetbrains-gradle test\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.ci_jobs_without_recipe(root), [".github/workflows/ci.yml:4 (smoke)"])

    def test_upstream_is_measured_with_this_checkouts_cpl(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            env = dr._git_env()
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=env)
            self._write(root, "target/debug/cpl", "#!/bin/sh\n")
            subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q",
                            "--allow-empty", "-m", "x"], cwd=root, check=True, env=env)
            seen = []
            with mock.patch.object(dr, "measure", lambda wt, *_: seen.append(os.environ.get("CPL")) or {}):
                dr._measure_at(root, "HEAD", 0)
            self.assertEqual(seen, [str((root / "target/debug/cpl").resolve())])

    def test_a_counter_without_its_tool_raises(self) -> None:
        with tempfile.TemporaryDirectory() as tmp, mock.patch.dict(os.environ, {}, clear=False):
            os.environ.pop("CPL", None)
            root = Path(tmp)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            self._write(root, f"{PKGS}/p/port.cp", "print(1)\n")
            self._write(root, "crates/x/src/lib.rs", "\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            with self.assertRaisesRegex(dr.MeasureError, "no cpl at target/debug/cpl; build it with `cargo build -p cpl-cli`"):
                dr.corpus_fmt_dirty(root)
            self._write(root, "target/debug/cpl", "#!/bin/sh\n")
            self._write(root, "target/debug/cpl.d", f"{root}/target/debug/cpl: crates/x/src/lib.rs\n")
            os.utime(root / "target/debug/cpl", (1, 1))
            with self.assertRaisesRegex(dr.MeasureError, "target/debug/cpl is stale: run cargo build -p cpl-cli"):
                dr.corpus_fmt_dirty(root)
            os.environ["CPL"] = str(root / "missing")
            with self.assertRaisesRegex(dr.MeasureError, "no cpl at"):
                dr.corpus_fmt_dirty(root)

    def test_a_stale_newer_cpl_gives_way_to_a_current_one(self) -> None:
        # A release cpl built before a rebase is newer than the debug cpl the tier just built.
        with tempfile.TemporaryDirectory() as tmp, mock.patch.dict(os.environ, {}, clear=False):
            os.environ.pop("CPL", None)
            root = Path(tmp)
            for f, t in (("crates/x/src/lib.rs", 1), ("crates/x/src/new.rs", 3000)):
                self._write(root, f, "\n")
                os.utime(root / f, (t, t))
            for build, dep, t in (("debug", "lib.rs", 1000), ("release", "new.rs", 2000)):
                self._write(root, f"target/{build}/cpl", "#!/bin/sh\n")
                self._write(root, f"target/{build}/cpl.d", f"{root}/target/{build}/cpl: crates/x/src/{dep}\n")
                os.utime(root / f"target/{build}/cpl", (t, t))
            self.assertEqual(dr._built_cpl(root), root / "target/debug/cpl")
            os.utime(root / "target/debug/cpl", (2, 2))
            os.utime(root / "crates/x/src/lib.rs", (5, 5))
            with self.assertRaisesRegex(dr.MeasureError, "target/release/cpl is stale: run cargo build --release -p cpl-cli"):
                dr._built_cpl(root)

    def test_uncached_installs_counts_installs_outside_install_tools(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, ".github/workflows/w.yml", "jobs:\n  a:\n    steps:\n"
                        "      - uses: taiki-e/install-action@v2\n"
                        "      - run: cargo binstall just\n"
                        "      - run: bash <(curl -sSfL https://example.com/download-actionlint.bash)\n"
                        "      - uses: ./.github/actions/install-tools\n"
                        "      - run: cargo build -p cpl-cli\n")
            self._write(root, dr.INSTALL_ACTION, "runs:\n  steps:\n    - run: retry cargo install --locked --root d x\n")
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.uncached_installs(root), [".github/workflows/w.yml:4", ".github/workflows/w.yml:5",
                                                          ".github/workflows/w.yml:6"])
        self.assertIn("uncached_installs", dr.HARD_ZERO)
        self.assertEqual(dr.uncached_installs(dr.ROOT if hasattr(dr, "ROOT") else Path(dr.__file__).resolve().parents[1]), [])

    def test_web_skip_only_reads_only_js_and_ts(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            self._write(root, "web/tests/a.test.ts", "it.only('x', () => {})\n")
            self._write(root, "web/lib/b.mjs", "describe.skip('y')\n")
            self._write(root, "web/content/c.cp", "let g = G.only(v => v > 0)\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(sorted(dr.web_skip_only(root)), ["web/lib/b.mjs:1", "web/tests/a.test.ts:1"])

    def test_foreign_outside_modules_counts_habit_docs_off_foreign_modules(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            doc = "/// Python's `a.index(x)`: `a.find(x)!`.\n"
            self._write(root, "crates/cpl-check/src/check/advice.rs",
                        doc + "pub(super) fn index_of() {}\n/// A C-style loop counter.\nfn counter() {}\n"
                        "#[cfg(test)]\nmod t {\n" + doc + "fn gated() {}\n}\n")
            self._write(root, "crates/cpl-ir/src/lower/typo.rs", "/// The C++/Rust spelling.\n#[inline]\nfn by() {}\n")
            self._write(root, "crates/cpl-check/src/check/foreign/python.rs", doc + "fn index_of() {}\n")
            self._write(root, "crates/cpl-syntax/src/lexer/foreign.rs", doc + "fn floor() {}\n")
            self._write(root, "crates/cpl-codegen/src/rt.rs", "/// As C's `printf` writes it.\nfn show() {}\n")
            self.assertEqual(sorted(dr.foreign_outside_modules(root)),
                             ["crates/cpl-check/src/check/advice.rs:2", "crates/cpl-ir/src/lower/typo.rs:3"])
        self.assertIn("foreign_outside_modules", dr.HARD_ZERO)

    def test_unrun_script_tests(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            self._write(root, "scripts/ci.py", 'SCRIPT_TESTS = ("scripts/test_*.py",)\n')
            self._write(root, "scripts/check.sh", "uv run python -m unittest tools/x/test_named.py\n")
            for rel in ("scripts/test_a.py", "tools/x/test_named.py", "tools/y/test_b.py", "corpus/test_c.py"):
                self._write(root, rel, "")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.unrun_script_tests(root), ["corpus/test_c.py", "tools/y/test_b.py"])

    def test_tests_unrun(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            self._write(root, "scripts/ci.py", 'SLOW_TESTS = ("named", "by_binary", "lost")\n'
                        'WALL_CLOCK_TESTS = ("fast_enough",)\n'
                        'STEPS = (("cargo", "test", "--exact", "named"), ("cargo", "test", "--test", "whole"))\n')
            self._write(root, ".github/workflows/ci-slow.yml", "run: just check-full\n")
            self._write(root, ".github/workflows/ci.yml", 'part: ["hash:1/2", "hash:2/2"]\n')
            self._write(root, "crates/a/tests/t.rs", "#[test]\nfn plain() {}\n#[test]\n#[ignore]\nfn off() {}\n"
                        "#[test]\nfn named() {}\n#[test]\nfn lost() {}\n#[test]\nfn fast_enough() {}\n")
            self._write(root, "crates/a/tests/whole.rs", "#[test]\nfn by_binary() {}\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.tests_unrun(root), [
                "crates/a/tests/t.rs::fast_enough (wall-clock, no timing job)",
                "crates/a/tests/t.rs::lost (SLOW_TESTS, no --full step)",
                "crates/a/tests/t.rs::off (ignored)",
            ])
            self._write(root, ".github/workflows/ci.yml", 'part: ["hash:1/2"]\n')
            self.assertIn("crates/a/tests/t.rs::plain (ci.yml's hash partitions miss a part)", dr.tests_unrun(root))
            self._write(root, ".github/workflows/ci.yml", 'part: ["time:1/2", "time:2/2"]\n')
            self.assertNotIn("crates/a/tests/t.rs::plain (ci.yml's hash partitions miss a part)", dr.tests_unrun(root))
        self.assertIn("tests_unrun", dr.HARD_ZERO)

    def test_stale_arch_paths_skips_ignored_and_known_debt(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            # A pre-commit hook exports GIT_DIR; `git init` would then rewrite the outer repo.
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, "web/.gitignore", "out/\n")
            self._write(root, "plan/domains/x/ARCHITECTURE.md",
                        "`web/out` and `web/gone.ts`\n## Known debt\n- `scripts/deleted.py` is gone\n")
            self.assertEqual(dr.stale_arch_paths(root), ["plan/domains/x/ARCHITECTURE.md:1 web/gone.ts"])

    def test_bench_lists_count_entries_of_tracked_bench_lists(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, "bench/oversize.txt", "# why\nbench/a.cp 40000\n\nbench/b.cp\n")
            self._write(root, "bench/dp/notes.txt", "x\n")
            self._write(root, "bench/a.cp", "print(1)\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self._write(root, "bench/untracked.txt", "y\n")
            self.assertEqual(sorted(dr.bench_lists(root)), [
                "bench/dp/notes.txt", "bench/oversize.txt", "bench/oversize.txt"])

    def test_a_skip_list_entry_fails_the_ratchet(self) -> None:
        self.assertIn("skip_lists", dr.HARD_ZERO)
        errs, _, _ = dr.evaluate({"skip_lists": 1}, {"skip_lists": 0})
        self.assertEqual(len(errs), 1)

    def test_skip_lists_count_entries_of_exemption_lists(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, "crates/a/tests/t.rs", "\n".join([
                "const NOT_BUILT: &[&str] = &[", '    "x.cp", // why', '    "y,z.cp",', "];",
                'const KNOWN_WORSE: &[(&str, &str)] = &[("a", "b"), ("c", "d")];',
                'const KNOWN_DIFFERS: [(&str, &str); 1] = [("e", "*")];',
                'const CASES: &[&str] = &["kept"];', ""]))
            self._write(root, "spec/rule-gaps.txt", "# header\nR1\n\nR2\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(sorted(dr.skip_lists(root)), [
                "crates/a/tests/t.rs KNOWN_DIFFERS",
                "crates/a/tests/t.rs KNOWN_WORSE", "crates/a/tests/t.rs KNOWN_WORSE",
                "crates/a/tests/t.rs NOT_BUILT", "crates/a/tests/t.rs NOT_BUILT",
                "spec/rule-gaps.txt", "spec/rule-gaps.txt",
            ])

    def test_escapes_open_needs_a_real_row_and_a_suite_fix(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "handbook/domains/x.md",
                        "## Quality register\n| Attribute | Check |\n|---|---|\n| Agree | `missing` |\n")
            entry = '[[escape]]\nbug = "{}"\ndomain = "x"\nrow = "{}"\nsuite_fix = "{}"\n'
            self._write(root, "plan/escapes.toml", entry.format("a", "Agree", "abc1234")
                        + entry.format("b", "Agree", "") + entry.format("c", "Nope", "abc1234"))
            self.assertEqual(dr.escapes_open(root), [
                "#2 x: b (suite not fixed)",
                "#3 x: c (no such register row: 'Nope')",
            ])

    def test_unwrap_input_counts_renamed_wrappers(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "crates/a/src/invariant.rs", "\n".join([
                "pub(crate) fn broken_invariant(invariant: &'static str) -> ! {",
                '    panic!("broken invariant: {invariant}")', "}", ""]))
            self._write(root, "crates/a/src/lib.rs", "\n".join([
                'fn ok() { let Some(x) = y else { broken_invariant("y is set") }; }',
                "#[track_caller]",
                "fn stop() -> ! { loop {} }",
                "fn forward(why: &'static str) { broken_invariant(why) }",
                "fn get<T>(o: Option<T>) -> T {",
                '    o.unwrap_or_else(|| broken_invariant("present"))',
                "}",
                "fn take(o: Option<u8>) -> u8 {",
                "    match o {",
                "        Some(v) => v,",
                "        None => broken_invariant(",
                '            "set",',
                "        ),",
                "    }",
                "}",
                "fn long(o: Option<u8>) -> u8 {", *(["    step();"] * 8),
                '    o.unwrap_or_else(|| broken_invariant("set"))', "}",
                "fn ends() -> u8 { loop {} }", ""]))
            self._write(root, "crates/b/src/invariant.rs", "fn a() { panic!(); }\nfn b() { x.unwrap(); }\n")
            self.assertEqual(dr.unwrap_input(root), [
                "crates/a/src/lib.rs:2", "crates/a/src/lib.rs:3", "crates/a/src/lib.rs:4",
                "crates/a/src/lib.rs:5", "crates/a/src/lib.rs:8", "crates/b/src/invariant.rs:2"])
            self.assertEqual(dr.invariant_sites(root), [f"crates/a/src/lib.rs:{i}" for i in (1, 4, 6, 11, 25)])

    def test_new_debt_counters(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, "crates/a/src/lib.rs", "\n".join([
                "fn f() { x.unwrap(); y.expect(\"e\"); panic!(\"p\") }",
                "// z.unwrap()", "#[cfg(test)]", "mod tests { fn t() { q.unwrap(); } }", ""]))
            self._write(root, "crates/a/src/x_tests.rs", "fn t() { q.unwrap(); }\n")
            self._write(root, "crates/a/build.rs", "fn main() { q.unwrap(); }\n")
            self._write(root, "crates/a/src/spec/build.rs", "fn f() { q.unwrap(); }\n")
            self._write(root, "crates/cpl-ir/src/lower/m.rs",
                        'self.unsupported(span, "x");\nlet c = "E0501";\n')
            self._write(root, "crates/cpl-lsp/src/hover.rs",
                        'if t.starts_with("fn ") {}\nl.find(|l| l.starts_with("cost: O("));\nl.starts_with("use ");\n')
            self._write(root, "crates/cpl-lint/src/typed.rs",
                        "const TYPED: &[(&Lint, &Lint)] = &[\n    (&A, &B),\n    (&C, &D),\n];\n")
            self._write(root, ".github/workflows/ci.yml", "\n".join([
                "jobs:", "  a:", "    continue-on-error: true", "    steps:",
                "      - run: make || true", "        continue-on-error: true",
                "      - run: x", "        continue-on-error: false", "      # a || true comment", ""]))
            self._write(root, f"{PKGS}/p/dream.cp", "print(1)\n")
            self._write(root, f"{PKGS}/p/meta.json", "{}")
            self._write(root, f"{PKGS}/q/dream.cp", "print(2)\n")
            self._write(root, "plan/domains/stdlib/measure.py", "")
            self._write(root, "plan/domains/tools/measure.py", "")
            self._write(root, "scripts/test_stdlib_measure.py", "")
            self._write(root, "scripts/walk.py", "for p in glob.glob('corpus/problems/*'):\n    pass\n")
            self._write(root, "scripts/changed.sh", "git diff --cached --name-only -- 'corpus/problems/*/tests/*'\n")
            self._write(root, "scripts/changed.py", '_git(root, "diff", "--cached", "--", "corpus/problems/*/tests/*")\n')
            self._write(root, "scripts/one.py", f"paths = glob.glob('docs/*') + ['!{PKGS}/cf1a/dream.cp']\n")
            self._write(root, "scripts/corpus.py", "os.listdir('corpus/problems')\n")
            self._write(root, "scripts/key.py", "x = glob.glob('a/*')\nclimb = json.load(f)[\"problems\"]\n")
            self._write(root, "crates/a/src/note.rs", "// reads corpus/problems/<id>/spec.cp\nfn f() { std::fs::read_dir(x); }\n")
            self._write(root, "tools/new-allow.txt", "x\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            blob = subprocess.run(["git", "rev-parse", f":{PKGS}/q/dream.cp"], cwd=root, check=True,
                                  capture_output=True, text=True, env=dr._git_env()).stdout.strip()
            self._write(root, f"{PKGS}/q/meta.json", json.dumps({"dream_stress": {"ok": True, "sha": blob}}))
            self.assertEqual(dr.unwrap_input(root), ["crates/a/src/lib.rs:1"] * 3 + ["crates/a/src/spec/build.rs:1"])
            self.assertEqual(len(dr.unsupported_sites(root)), 2)
            self.assertEqual(dr.lsp_string_types(root), ["crates/cpl-lsp/src/hover.rs:1", "crates/cpl-lsp/src/hover.rs:2"])
            self.assertEqual(dr.lint_post_filters(root), ["crates/cpl-lint/src/typed.rs TYPED"] * 2)
            self.assertEqual(dr.or_true(root), [".github/workflows/ci.yml:5", ".github/workflows/ci.yml:9"])
            self.assertEqual(dr.continue_on_error(root), [".github/workflows/ci.yml:3", ".github/workflows/ci.yml:6"])
            self.assertEqual(dr.dream_unverified(root), [f"{PKGS}/p/dream.cp"])
            self.assertEqual(dr.measure_broken(root), ["plan/domains/tools/measure.py"])
            self.assertEqual(dr.mono_unguarded(root), ["crates/cpl-check, crates/cpl-ir: no GROWING_INSTANTIATION use"])
            self.assertEqual(dr.corpus_walkers(root), ["scripts/walk.py"])
            self.assertEqual(dr.unregistered_lists(root), ["tools/new-allow.txt"])

    def test_one_mechanism_surface_counters(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, "crates/cpl-json/Cargo.toml", "[dependencies]\nserde_json = '1'\n")
            self._write(root, "crates/cpl-json/src/lib.rs", 'let j = format!("{{\\"a\\":{}}}", 1);\n')
            self._write(root, "crates/a/Cargo.toml", "[dependencies]\nserde_json = '1'\n")
            self._write(root, "crates/a/src/lib.rs", "\n".join([
                'let j = format!("{{\\"a\\":{}}}", 1);', 'let k = format!("x\\": {}", 1);',
                'let c = root.join("cp.toml");', 'if l.starts_with("///") {}', "s.replace_range(0..1, \"\");",
                "let db = Database::new();", '    "{{\\"b\\":{}}}",', 's.push_str("],\\"c\\":[");',
                'const USAGE: &str = "print `{\\"d\\": [...]}`";', ""]))
            self._write(root, "crates/a/src/tests_x.rs", "s.replace_range(0..1, \"\");\n")
            self._write(root, "crates/cpl-diag/src/lib.rs", "fn apply_fixes() { s.replace_range(0..1, \"\"); }\n")
            self._write(root, "crates/cpl-doc/src/lib.rs", 'l.strip_prefix("///");\n')
            self._write(root, "crates/cpl-config/src/lib.rs", 'pub const FILE: &str = "cp.toml";\n')
            self._write(root, "crates/cpl-check/src/lib.rs", "Database::new();\n")
            self._write(root, "scripts/mod.py", "for p in root.glob('**/*.cp'):\n    p.write_text(fix(p))\n")
            self._write(root, "scripts/read.py", "for p in root.glob('**/*.cp'):\n    out.write_text(p.name)\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.json_outside_cpl_json(root),
                             ["crates/a/Cargo.toml", "crates/a/src/lib.rs:1", "crates/a/src/lib.rs:2",
                              "crates/a/src/lib.rs:7", "crates/a/src/lib.rs:8"])
            self.assertEqual(dr.config_schemas(root), ["crates/a/src/lib.rs"])
            self.assertEqual(dr.std_doc_parsers(root), ["crates/a/src/lib.rs:4"])
            self.assertEqual(dr.edit_engines(root), ["crates/a/src/lib.rs:5"])
            self.assertEqual(dr.codemods_outside_fix(root), ["scripts/mod.py p"])
            self.assertEqual(dr.module_loaders(root), ["crates/a/src/lib.rs:6"])

    def test_ci_jobs_without_recipe(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, ".github/workflows/ci.yml", "\n".join([
                "on: push", "jobs:", "  good:", "    steps:", "      - run: just check", "      - run: cargo build -p cpl-cli",
                "  raw:", "    name: scripts/x.sh in a name is fine", "    steps:", "      - name: a", "        run: |",
                "          echo hi", "          scripts/check.sh --fast", "      - run: cargo test -p a",
                "  block:", "    steps:", "      - run: |", "          # scripts/commented.sh", "          . scripts/retry.sh", "          just site",
                "      - uses: x", "        with:", "          path: scripts/y",
            ]) + "\n")
            self._write(root, ".github/workflows/release.yml", "jobs:\n  b:\n    steps:\n      - run: scripts/release/build.sh\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.ci_jobs_without_recipe(root), [".github/workflows/ci.yml:13 (raw)"])

    def test_editor_logic_lines_counts_tracked_sources_only(self) -> None:
        """A clean checkout and a worktree full of build output count the same."""
        with tempfile.TemporaryDirectory() as tmp:
            root, env = Path(tmp) / "clean", dr._git_env()
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=env)
            self._write(root, "editors/vscode/src/runner.ts", "run();\nstop();\n")
            self._write(root, ".gitignore", "node_modules/\nout/\ndist/\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=env)
            subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@example.com", "commit", "-q", "-m", "x"],
                           cwd=root, check=True, env=env)
            clean = dr.editor_logic_lines(root)
            built = Path(tmp) / "built"
            subprocess.run(["git", "worktree", "add", "-q", str(built)], cwd=root, check=True, env=env)
            for rel in ("editors/vscode/node_modules/x/index.js", "editors/vscode/out/runner.js",
                        "editors/vscode/dist/extension.js", "editors/vscode/src/scratch.ts"):
                self._write(built, rel, "a();\nb();\n")
            self.assertEqual(clean, ["editors/vscode/src/runner.ts:1", "editors/vscode/src/runner.ts:2"])
            self.assertEqual(dr.editor_logic_lines(built), clean)

    def test_editor_logic_lines_read_a_case_key_as_data_and_a_case_arm_as_a_branch(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, "editors/vscode/src/extension.ts", "\n".join([
                "exec('run', { case: name });", "const v = { case : name };", "send(args.case);",
                "switch (kind) {", "    case 'live':", "    case Kind.Error: return;", "}", ""]))
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.editor_logic_lines(root), [f"editors/vscode/src/extension.ts:{i}" for i in (4, 5, 6)])

    def test_editor_logic_lines_read_a_restart_on_launch_settings_as_wiring(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            sections = [{"id": "cp.server", "properties": {"cp.server.path": {}}},
                        {"id": "cp", "properties": {"cp.run.timeoutMs": {}}}]
            self._write(root, "editors/vscode/package.json", json.dumps({"contributes": {"configuration": sections}}))
            self._write(root, "editors/vscode/src/client.ts", "new LanguageClient(get('server.path'), get('run.timeoutMs'));\n")
            self._write(root, "editors/vscode/src/extension.ts", "\n".join([
                "if (event.affectsConfiguration('cp.server')) {", "    void client.restart();", "}",
                "if (event.affectsConfiguration('cp.server')) client.restart();",
                "if (event.affectsConfiguration('cp')) {", "    void client.restart();", "}",
                "if (event.affectsConfiguration('cp.server')) {", "    show();", "}", "get('run.timeoutMs');", ""]))
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.editor_logic_lines(root), [f"editors/vscode/src/extension.ts:{i}" for i in (5, 8)])
            self._write(root, "editors/vscode/src/extension.ts",
                        "if (event.affectsConfiguration('cp.server')) client.restart();\nget('server.path');\n")
            self.assertEqual(dr.editor_logic_lines(root), ["editors/vscode/src/extension.ts:1"])
            self.assertIn("editor_logic_lines", dr.HARD_ZERO)

    def test_surface_counters_find_a_second_mechanism(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, "std/core.cp", "pub fn foo(x: Int) -> Int:\n    x\n")
            self._write(root, "editors/vscode/src/languageData.ts",
                        "export const KEYWORDS = ['fn', 'let'];\nexport const CONTEXTUAL_KEYWORDS = ['by'];\n")
            self._write(root, "editors/vscode/src/extension.ts",
                        "activate();\nif (ready) start();\nshow('wait for it && go');\nconst a = b ?? c;\n")
            self._write(root, "editors/vscode/src/runner.ts", "// runs\nconst a = 1;\n\nrun();\n")
            self._write(root, "editors/jetbrains/src/main/kotlin/dev/cplang/ide/CpLsp.kt",
                        "start()\nval w = find() ?: make()\nshow(\"a ?: b\")\n")
            self._write(root, "editors/vscode/test/unit/x.ts", "check();\n")
            self._write(root, "web/lib/kinds.ts", "const k = ['fn', 'let', 'E0301', 'other'];\n"
                        "const plain = ['a', 'b', 'c', 'd'];\nconst few = ['fn', 'let'];\n")
            self._write(root, "web/lib/kinds.test.ts", "const k = ['fn', 'let', 'foo', 'by'];\n")
            self._write(root, "crates/a/src/lib.rs", "\n".join([
                'let d = Diagnostic::error(code, span, "bad");', '"U8" => 1,', '"U64" => Ty::U64,',
                "Judge::Codeforces => 1,", '"int32_t" | "int" => 4,', ""]))
            self._write(root, "crates/cpl-diag/src/lib.rs", 'Diagnostic::error(code, span, "x");\n')
            self._write(root, "crates/cpl-judge/src/lib.rs", '"codeforces",\n')
            self._write(root, "crates/cpl-ir/src/lib.rs", 'IntWidth::U8 => "uint8_t",\n"U8" => 1,\n')
            self._write(root, "crates/cpl-interp/src/lib.rs", "\n".join([
                "struct Interp<O> {", "    tracer: Option<Box<O>>,", "    profiler: Option<Box<profile::Profiler>>,",
                "}", "fn f() { self.tracer.take(); }", "fn g() { self.profiler.take(); }", ""]))
            self._write(root, "crates/cpl-interp/src/observe.rs", "\n".join([
                "pub(crate) trait Observer {", "    fn on(&mut self, _event: Event) -> bool {", "    }",
                "    fn keeps_step(&self) -> bool {", "    }", "    fn step(&mut self, _stmt: &Stmt) {}", "}",
                "fn helper(&mut self) {}", ""]))
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.web_hand_copied(root), ["web/lib/kinds.ts:1"])
            self.assertEqual(dr.diag_text_outside_registry(root), ["crates/a/src/lib.rs:1"])
            self.assertEqual(dr.width_tables(root), ["crates/a/src/lib.rs:2", "crates/a/src/lib.rs:5"])
            self.assertEqual(dr.judge_tables(root), ["crates/a/src/lib.rs:4"])
            self.assertEqual(dr.event_models(root), ["crates/cpl-interp/src/observe.rs:6",
                                                    "crates/cpl-interp/src/lib.rs:6"])
            self.assertEqual(dr.editor_logic_lines(root),
                             ["editors/jetbrains/src/main/kotlin/dev/cplang/ide/CpLsp.kt:2",
                              "editors/vscode/src/extension.ts:2", "editors/vscode/src/extension.ts:4",
                              "editors/vscode/src/runner.ts:2", "editors/vscode/src/runner.ts:4"])

    def test_raw_repo_wide_counters(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, "web/install.sh", "rm x || true\n")
            self._write(root, "crates/a/src/lib.rs", 'env::var("CPL_TRACE_ENGINE");\n')
            self._write(root, "corpus/generators-known.txt", "# why\ncf1\ncf2\n")
            self._write(root, "crates/a/src/big.rs", "fn a() {}\n\n// note\n    // more\nfn b() {}\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.or_true(root), ["web/install.sh:1"])
            self.assertEqual(dr.trace_engine_refs(root), ["crates/a/src/lib.rs:1"])
            with mock.patch.dict(dr.LISTS, {"corpus/generators-known.txt": ("corpus_known", "idioms", "fix each")}):
                self.assertEqual(dr.corpus_known(root), ["corpus/generators-known.txt"] * 2)
            self.assertEqual(dr.rs_over_cap(root, 1), {"crates/a/src/big.rs": 2})

    def test_std_types_shown_empty(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, "std/a.cp", "pub struct Hid(m: Int):\n    t: List<Int>\n\n"
                        "pub struct Shown:\n    t: Int\n\npub struct Open:\n    pub x: Int\n    y: Int\n\n"
                        "impl Show for Shown:\n    fn show(s) = \"\"\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.std_types_shown_empty(root), ["std/a.cp:1"])

    def test_zero_fails_while_any_gated_counter_is_above_zero(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._make_repo(root)
            self.assertEqual(dr.main(["--zero", "--root", str(root)]), 1)
            with mock.patch.dict(dr.COUNTERS, {n: (l, lambda r: [], g) for n, (l, _, g) in dr.COUNTERS.items()}), \
                    mock.patch.object(dr, "rs_over_cap", lambda r, m: {}):
                self.assertEqual(dr.main(["--zero", "--root", str(root)]), 0)

    def test_the_evadable_counters_see_the_spellings_the_audit_found(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            src = root / "crates/cpl-next/src"
            src.mkdir(parents=True)
            (root / "std").mkdir()
            (root / "std/prelude.cp").write_text("@derive\nstruct List {}\n")
            (src / "lib.rs").write_text(
                'Some("x") => 1,\n("dom_at", Some(k)) => 2,\nm.get("new");\nn.get(k);\n'
                'pub const CORE: ModId = ModId(0);\nif m == decls::CORE {}\nCORE.0;\n'
                'const H: &str = r"int f();\nint g();";\nconst I: &str = include_str!("rt.hpp");\n'
                'let m = "std.prelude";\nmarked(s, "derive");\n'
            )
            rel = "crates/cpl-next/src/lib.rs"
            self.assertEqual(dr.next_name_compares(root), [f"{rel}:{i}" for i in (1, 2, 3)])
            self.assertEqual(dr.next_core_checks(root), [f"{rel}:{i}" for i in (6, 7)])
            self.assertEqual(dr.next_cpp_runtime_lines(root),
                             [f"{rel}:8", f"{rel}:9", f"{rel}:10: include_str"])
            names = dr.next_std_names(root)
            self.assertIn(f"{rel}:11: std.prelude", names)
            self.assertIn(f"{rel}:12: derive", names)

    def test_a_harness_refusal_site_is_counted_where_the_pipeline_s_is_not(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            t = root / dr.NEXT_TESTS
            t.mkdir(parents=True)
            (t / "programs.rs").write_text(
                'return ("refused", "a lint".into());\n'
                'Unbuilt(e) => (if x { "refused" } else { "wrong" }, e),\n'
            )
            self.assertEqual(dr.next_harness_refusals(root), [f"{dr.NEXT_TESTS}/programs.rs:1"])

    def test_zero_with_a_prefix_counts_only_the_counters_it_names(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._make_repo(root)
            fake = {"next_a": ("a", lambda r: [], True), "next_b": ("b", lambda r: ["x"], True),
                    "other": ("o", lambda r: ["y"], True)}
            with mock.patch.dict(dr.COUNTERS, fake, clear=True), \
                    mock.patch.object(dr, "rs_over_cap", lambda r, m: {"big.rs": 3000}):
                self.assertEqual(dr.main(["--zero", "--prefix", "next_", "--root", str(root)]), 1)
                dr.COUNTERS["next_b"] = ("b", lambda r: [], True)
                self.assertEqual(dr.main(["--zero", "--prefix", "next_", "--root", str(root)]), 0)
                self.assertEqual(dr.main(["--zero", "--root", str(root)]), 1)

    def test_retry_calls_and_poll_sleeps(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, "scripts/a.sh", ". retry.sh\nx=$(retry make y)\n# retry here\nretrying=1\nx=$(retry gh api y)\n")
            self._write(root, "scripts/b.py", "from retry import retry\nreturn retry(f, tries=3)\nreturn retry(post_chat, transient=t)\n")
            self._write(root, "crates/a/src/lib.rs", "std::thread::sleep(d);\n// thread::sleep\n")
            self._write(root, "crates/a/src/tests.rs", "std::thread::sleep(d);\n")
            subprocess.run(["git", "-C", str(root), "add", "-A"], check=True, env=dr._git_env())
            self.assertEqual(dr.retry_calls(root), ["scripts/a.sh:2", "scripts/b.py:2"])
            self.assertEqual(dr.poll_sleeps(root), ["crates/a/src/lib.rs:1"])

    def test_a_retry_wraps_only_a_named_fetch(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            named = ["retry timeout 180 sudo apt-get install -y emacs-nox", "retry sh -c 'curl -sSfL $url | tar x'",
                     "retry npm install --prefix t x", "x=$(retry gh api repos/a)", "retry gh release view edge",
                     "retry gh run view 1 --log-failed", 'retry "$mm/bin/micromamba" create -y',
                     "RETRY_TRIES=5 retry sh -c 'git pull -q --rebase origin main && git push -q origin HEAD:main'",
                     "RETRY_WAIT=0 retry push || exit 1"]
            self._write(root, ".github/workflows/w.yml", "\n".join(named) + "\n")
            self._write(root, "scripts/c.sh", "retry cargo test\nretry sh -c 'make all'\nretry pushd x\n")
            subprocess.run(["git", "-C", str(root), "add", "-A"], check=True, env=dr._git_env())
            self.assertEqual(dr.retry_calls(root), ["scripts/c.sh:1", "scripts/c.sh:2", "scripts/c.sh:3"])
            self.assertIn("retry_calls", dr.HARD_ZERO)

    def test_only_the_file_watch_and_the_rate_limiter_may_sleep(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            for owner in dr.SLEEP_OWNERS:
                self._write(root, owner, "std::thread::sleep(self.every);\n")
            self._write(root, "crates/cpl-contest/src/third.rs", "std::thread::sleep(tick);\n")
            subprocess.run(["git", "-C", str(root), "add", "-A"], check=True, env=dr._git_env())
            self.assertEqual(dr.poll_sleeps(root), ["crates/cpl-contest/src/third.rs:1"])
            self.assertIn("poll_sleeps", dr.HARD_ZERO)

    def test_cpl_env_reads_live_in_cpl_env(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, "crates/cpl-env/src/lib.rs", 'std::env::var_os("CPL_X")\n')
            self._write(root, "crates/a/src/lib.rs", 'cpl_env::X.text()\nstd::env::set_var("CPL_X", "1");\n'
                        'env::remove_var(\n"CPL_X")\nvar_os( "CPL_Y")\nenv::var("HOME")\n')
            subprocess.run(["git", "-C", str(root), "add", "-A"], check=True, env=dr._git_env())
            self.assertEqual(dr.cpl_env_reads(root), ["crates/a/src/lib.rs:2", "crates/a/src/lib.rs:5"])
            self.assertIn("cpl_env_reads", dr.HARD_ZERO)

    def test_addendum_five_counters(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, "crates/a/src/lib.rs", "#[allow(dead_code)]\nunsafe { x() }\n"
                        'let v = env::var("CPL_X");\nlet c = "see E0123";\n')
            self._write(root, "crates/a/tests/t.rs", "std::thread::sleep(d);\n")
            self._write(root, "scripts/ci.sh", "curl --retry 3 x\n")
            self._write(root, "web/tests/a.test.ts", "it.skip(x)\n// @ts-ignore\n")
            self._write(root, "std/a.cp", "// TODO x\n")
            self._write(root, "spec/conformance/c.cp", "// gap: core 2026-11-24\n")
            self._write(root, "crates/cpl-check/src/check/advice.rs",
                        'd.with_help("x");\nd.with_help(habits::A.say(&[]));\n// d.fix("y", s, "");\n')
            self._write(root, "crates/cpl-diag/src/habits.rs", 'shape("a", c, "x", "");\nshape("gone", c, "y", "");\n')
            self._write(root, "crates/cpl-cli/tests/habits/a.cp", "let x = 1  // <- here\n")
            self._write(root, "crates/cpl-stress/src/engine.rs", "fn run_case() {}\n")
            self._write(root, "crates/cpl-stress/src/cli.rs", "Verdict::Ok\nstress_with(&s, &r, &o)\n")
            self._write(root, "crates/cpl-stress/src/stress.rs", "stress_with(&s, &r, &o)\n")
            self._write(root, "crates/cpl-stress/src/judge.rs", "Verdict::Ok\n")
            self._write(root, "crates/b/src/lib.rs", "cpl_diag::fix::apply_fixes(x);\n// XFAIL\n")
            self._write(root, "decisions.md", "the known-fail list is gone (xfail_markers)\n")
            self._write(root, "scripts/debt-ratchet.json", '{"xfail_markers": 1}\n')
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.allow_attrs(root), ["crates/a/src/lib.rs:1"])
            self.assertEqual(dr.unsafe_blocks(root), ["crates/a/src/lib.rs:2"])
            self.assertEqual(dr.cpl_env_reads(root), ["crates/a/src/lib.rs:3"])
            self.assertEqual(dr.hardcoded_codes(root), ["crates/a/src/lib.rs:4"])
            self.assertEqual(dr.test_sleeps(root), ["crates/a/tests/t.rs:1"])
            self.assertEqual(dr.ci_retries(root), ["scripts/ci.sh:1"])
            self.assertEqual(dr.web_skip_only(root), ["web/tests/a.test.ts:1"])
            self.assertEqual(dr.ts_ignore(root), ["web/tests/a.test.ts:2"])
            self.assertEqual(dr.todo_fixme(root), ["std/a.cp:1"])
            self.assertEqual(dr.xfail_markers(root), ["crates/b/src/lib.rs:2", "spec/conformance/c.cp:1"])
            self.assertEqual(len(dr.habit_matchers(root)), 2)
            self.assertEqual(dr.stress_engines(root), ["crates/cpl-stress/src/engine.rs:1", "crates/cpl-stress/src/cli.rs:2", "crates/cpl-stress/src/cli.rs"])
            self.assertEqual(dr.unchecked_fix_paths(root), ["crates/b/src/lib.rs:1"])

    def test_a_counter_at_zero_is_never_raised(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, "scripts/debt-ratchet.json", json.dumps({"lint_std_names": 0}))
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "x"],
                           cwd=root, check=True, env=dr._git_env())
            self._write(root, "decisions.md", "lint_std_names may rise\n")
            self.assertEqual(dr.unexplained_bumps(root, {"lint_std_names": 1}, ref="HEAD"),
                             ["lint_std_names: reached 0 on HEAD; a counter at 0 is never raised"])

    def test_only_the_coordinator_signs_a_raise_and_it_gives_its_reason(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            git = lambda *a: subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *a], cwd=root,
                                            check=True, capture_output=True, env=dr._git_env())
            git("init", "-q", "-b", "main")
            self._write(root, "scripts/debt-ratchet.json", json.dumps({"external_crates": 5}))
            self._write(root, "decisions.md", "# Decisions\n")
            git("add", "-A")
            git("commit", "-qm", "base")
            git("checkout", "-q", "-b", "lane")
            bump = lambda raise_: mock.patch.dict(os.environ, {"CPL_RAISE": raise_} if raise_ else {})
            said = lambda: dr.unexplained_bumps(root, {"external_crates": 6}, ref="main")
            with bump(""):
                self.assertIn("signed off by coordinator", said()[0])
            with bump("external_crates 5 -> 6, signed off by E"):
                self.assertIn("only the coordinator signs a raise", said()[0])
            with bump("external_crates 5 -> 6, signed off by coordinator"):
                self.assertIn("gives no reason", said()[0])
            self._write(root, "decisions.md", "# Decisions\nexternal_crates goes 5 -> 6: why.\n")
            with bump("external_crates 5 -> 7, signed off by coordinator"):
                self.assertIn("does not match the baseline's 5 -> 6", said()[0])
            with bump("external_crates 5 -> 6, signed off by coordinator"):
                self.assertEqual(said(), [])
            self._write(root, "decisions.md", "# Decisions\n")
            git("commit", "-q", "--allow-empty", "-m", "x: y\n\nexternal_crates grows: why.\n\nRaise: external_crates 5 -> 6, signed off by coordinator")
            self.assertEqual(said(), [])
            self.assertEqual(dr.raises(root, "main"), [(git("rev-parse", "--short=9", "HEAD").stdout.decode().strip(),
                                                        "external_crates", "5 -> 6, signed off by coordinator")])

    def test_a_hard_zero_counter_is_never_raised_even_signed(self) -> None:
        with mock.patch.dict(os.environ, {"CPL_RAISE": "frozen_std_drift 0 -> 1, signed off by coordinator"}):
            self.assertIn("hard-zero", dr.signed_raise(Path("."), "HEAD", "frozen_std_drift", 0, 1))

    def test_main_lowering_a_baseline_mid_land_is_not_this_changes_raise(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            git = lambda *a: subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *a], cwd=root,
                                            check=True, capture_output=True, env=dr._git_env())
            git("init", "-q", "-b", "main")
            self._write(root, "scripts/debt-ratchet.json", json.dumps({"lint_std_names": 5, "skip_lists": 2}))
            git("add", "-A")
            git("commit", "-qm", "base")
            git("checkout", "-q", "-b", "lane")
            git("checkout", "-q", "main")
            self._write(root, "scripts/debt-ratchet.json", json.dumps({"lint_std_names": 3, "skip_lists": 0}))
            git("commit", "-qam", "main lowers both")
            git("checkout", "-q", "lane")
            self.assertEqual(dr.unexplained_bumps(root, {"lint_std_names": 5, "skip_lists": 2}, ref="main"), [])
            self.assertEqual([b.split(":")[0] for b in dr.unexplained_bumps(root, {"lint_std_names": 6}, ref="main")],
                             ["lint_std_names"])
            self.assertEqual(dr.unexplained_bumps(root, {"skip_lists": 3}, ref="main"),
                             ["skip_lists: reached 0 on main; a counter at 0 is never raised"])

    def test_a_number_lowered_by_hand_is_refused_and_restoring_it_is_not_a_raise(self) -> None:
        # f1b1dd166 set vm_fallbacks 62 -> 1 with 62 still in the source.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            git = lambda *a: subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *a], cwd=root,
                                            check=True, capture_output=True, env=dr._git_env())
            git("init", "-q", "-b", "main")
            self._write(root, "scripts/debt-ratchet.json", json.dumps({"vm_fallbacks": 62}))
            git("add", "-A")
            git("commit", "-qm", "base")
            self.assertEqual([e.split(":")[0] for e in dr.hand_lowered(root, {"vm_fallbacks": 1}, {"vm_fallbacks": 62}, ref="main")],
                             ["vm_fallbacks"])
            self.assertEqual(dr.hand_lowered(root, {"vm_fallbacks": 59}, {"vm_fallbacks": 59}, ref="main"), [])
            self._write(root, "scripts/debt-ratchet.json", json.dumps({"vm_fallbacks": 1}))
            git("commit", "-qam", "lowered by hand")
            self.assertEqual(dr.unexplained_bumps(root, {"vm_fallbacks": 62}, ref="main", upstream={"vm_fallbacks": 62}), [])
            self.assertEqual([b.split(":")[0] for b in dr.unexplained_bumps(root, {"vm_fallbacks": 63}, ref="main",
                                                                              upstream={"vm_fallbacks": 62})],
                             ["vm_fallbacks"])
            self.assertEqual(len(dr.unexplained_bumps(root, {"vm_fallbacks": 62}, ref="main")), 1)
            bumps = dr.unexplained_bumps(root, {"vm_fallbacks": 61}, ref="main")
            self.assertIn("vm_fallbacks", dr.upstream_counters({"vm_fallbacks": 61}, {"vm_fallbacks": 61}, bumps))

    def test_a_file_entering_the_near_cap_list_is_a_raise(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, "scripts/debt-ratchet.json", json.dumps({"rs_near_cap": ["a.rs"]}))
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "x"],
                           cwd=root, check=True, env=dr._git_env())
            bumps = dr.unexplained_bumps(root, {"rs_near_cap": ["a.rs", "b.rs"]}, ref="HEAD")
            self.assertEqual([b.split(":")[0] for b in bumps], ["rs_near_cap"])

    def test_site_rederived_and_browser_lsp_gaps(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, "web/lib/a.ts", "\n".join([
                "if (t === 'Int') {}", "const e = 'E0308';", "  hoverProvider: { uses: [] },",
                "readFileSync('spec/lints/x/lint.toml');", "// spec/lints is read by cpl", "if (k === 'Foo') {}", ""]))
            self._write(root, "web/lib/a.test.ts", "if (t === 'Int') {}\n")
            self._write(root, "web/content/generated/b.js", "const e = 'E0308';\n")
            self._write(root, "crates/cpl-lsp/src/server.rs", "\n".join(["use a;",
                "fn capabilities() -> Value {", '    ("hoverProvider", true.into()),',
                '    ("renameProvider", obj([("prepareProvider", true.into())])),',
                '    ("workspaceSymbolProvider", true.into()),', "}", ""]))
            self._write(root, "web/components/editor/intelligence.ts", 'ask("textDocument/hover");\n')
            self._write(root, "web/components/editor/more.ts", 'ask("workspace/symbol");\n')
            self._write(root, "web/components/editor/x.test.ts", 'ask("textDocument/rename");\n')
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.site_rederived(root), [
                "web/lib/a.ts:1 type Int", "web/lib/a.ts:2 code E0308", "web/lib/a.ts:3 capability hoverProvider",
                "web/lib/a.ts:4 lint source"])
            self.assertEqual(dr.browser_lsp_gaps(root), ["renameProvider"])
        self.assertNotIn("hoverProvider", dr.browser_lsp_gaps(dr.ROOT))

    def test_site_spec_snippets(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, "web/content/landing/a.spec.cp", "let x = 1\n")
            self._write(root, "web/content/landing/b.cp", "let x = 1\n")
            self._write(root, "web/content/docs/c.mdx", "```cp run\nx\n```\n\n```cp status=spec\ny\n```\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.site_spec_snippets(root), ["web/content/docs/c.mdx:5", "web/content/landing/a.spec.cp"])

    def test_every_list_row_names_a_counter(self) -> None:
        for rel, (counter, owner, plan) in dr.LISTS.items():
            self.assertTrue(counter is None and plan.startswith("data:") or counter in dr.COUNTERS, rel)
            self.assertTrue(owner and plan, rel)

    def test_register_rows_never_miss_ignored_or_build_output_paths(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, ".gitignore", "/out/\n")
            self._write(root, "handbook/domains/x.md", "\n".join([
                "## Quality register", "| Attribute | Metric | Check |", "|---|---|---|",
                "| a | m | `target/release/cpl` |",
                "| b | m | `out/report.json` |",
                "| c | m | `scripts/gone.py` |", ""]))
            self.assertEqual(dr.register_rows(root), [
                "handbook/domains/x.md:4 missing: a",
                "handbook/domains/x.md:5 missing: b",
                "handbook/domains/x.md:6 no such check scripts/gone.py: c",
            ])

    def test_hard_zero_counters_fail_on_any_instance_and_never_rise(self) -> None:
        name = sorted(dr.HARD_ZERO)[0]
        errs, _, _ = dr.evaluate({name: 1}, {name: 5})
        self.assertEqual(len(errs), 1)
        self.assertIn("must stay at 0", errs[0])
        errs, _, _ = dr.evaluate({name: 1}, {})
        self.assertEqual(len(errs), 1)
        self.assertEqual(dr.inherited_baseline({name: 0}, {name: 3})[name], 0)

    def test_an_unverified_dream_fails_the_ratchet(self) -> None:
        self.assertIn("dream_unverified", dr.HARD_ZERO)
        errs, _, _ = dr.evaluate({"dream_unverified": 1}, {"dream_unverified": 0})
        self.assertEqual(len(errs), 1)
        self.assertIn("must stay at 0", errs[0])

    def test_an_ignored_rust_test_fails_the_ratchet(self) -> None:
        self.assertIn("rust_ignore", dr.HARD_ZERO)
        errs, _, _ = dr.evaluate({"rust_ignore": 1}, {"rust_ignore": 0})
        self.assertEqual(len(errs), 1)
        self.assertIn("must stay at 0", errs[0])

    def test_an_unscored_review_fails_the_ratchet(self) -> None:
        self.assertIn("reviews_pending", dr.HARD_ZERO)
        errs, _, _ = dr.evaluate({"reviews_pending": 1}, {"reviews_pending": 0})
        self.assertEqual(len(errs), 1)
        self.assertIn("must stay at 0", errs[0])

    def test_a_review_without_a_port_or_dream_is_an_orphan(self) -> None:
        self.assertIn("reviews_orphan", dr.HARD_ZERO)
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            for f in ["corpus/problems/a/port.cp", "corpus/problems/b/dream.cp", "corpus/problems/c/meta.json",
                      "research/reviews/README.md", "research/reviews/a.md", "research/reviews/b.md",
                      "research/reviews/c.md", "research/reviews/gone.md"]:
                (root / f).parent.mkdir(parents=True, exist_ok=True)
                (root / f).write_text("x\n")
            subprocess.run(["git", "init", "-q", d], check=True)
            subprocess.run(["git", "-C", d, "add", "."], check=True)
            self.assertEqual(dr.reviews_orphan(root), ["research/reviews/c.md", "research/reviews/gone.md"])

    def test_a_package_without_tests_toml_fails_the_ratchet(self) -> None:
        self.assertIn("corpus_no_tests", dr.HARD_ZERO)
        errs, _, _ = dr.evaluate({"corpus_no_tests": 1}, {"corpus_no_tests": 0})
        self.assertEqual(len(errs), 1)
        self.assertIn("must stay at 0", errs[0])

    def test_a_port_without_a_limits_line_fails_the_ratchet(self) -> None:
        self.assertIn("corpus_no_limits", dr.HARD_ZERO)
        errs, _, _ = dr.evaluate({"corpus_no_limits": 1}, {"corpus_no_limits": 0})
        self.assertEqual(len(errs), 1)
        self.assertIn("must stay at 0", errs[0])

    def test_an_optional_copies_header_fails_the_ratchet(self) -> None:
        self.assertIn("cpp_optional_copies", dr.HARD_ZERO)
        errs, _, _ = dr.evaluate({"cpp_optional_copies": 1}, {"cpp_optional_copies": 0})
        self.assertEqual(len(errs), 1)
        self.assertIn("must stay at 0", errs[0])

    def test_hard_zero_is_assigned_once(self) -> None:
        src = Path(dr.__file__).read_text(encoding="utf-8")
        self.assertEqual(len(re.findall(r"^HARD_ZERO\s*[|:]?=", src, re.M)), 1,
                         "add zero counters to the one HARD_ZERO frozenset, never rebind it")
        self.assertIsInstance(dr.HARD_ZERO, frozenset)

    def test_mains_paid_debt_is_mains_unless_this_change_adds_more(self) -> None:
        n = "lint_name_matchers"
        self.assertIn(n, dr.HARD_ZERO)
        self.assertEqual(dr.mains_paid_debt({n: 3}, {n: 3}), {n})
        self.assertEqual(dr.mains_paid_debt({n: 4}, {n: 3}), set())
        self.assertEqual(dr.mains_paid_debt({n: 1}, {}), set())
        self.assertIn(n, dr.upstream_counters({n: 1}, {n: 0}, []))

    def test_every_zero_baseline_is_hard_zero(self) -> None:
        baseline = json.loads(dr.DEFAULT_RATCHET.read_text(encoding="utf-8"))
        gated = {n for n, (_, _, g) in dr.COUNTERS.items() if g}
        self.assertEqual({n for n in gated if baseline.get(n) == 0} - dr.HARD_ZERO, set())
        self.assertEqual(dr.HARD_ZERO - set(dr.COUNTERS), set())
        self.assertEqual({n for n in gated if n not in baseline}, set())

    def test_a_gated_counter_without_a_baseline_fails(self) -> None:
        errs, _, _ = dr.evaluate({"ir_reinfer": 1, "rust_ignore": 0}, {"rust_ignore": 0})
        self.assertEqual(len(errs), 1)
        self.assertIn("no baseline", errs[0])

    def test_one_unwrap_in_shipped_code_fails_even_against_a_looser_baseline(self) -> None:
        self.assertIn("unwrap_input", dr.HARD_ZERO)
        errs, _, _ = dr.evaluate({"unwrap_input": 1}, {"unwrap_input": 5})
        self.assertEqual(len(errs), 1)
        self.assertIn("must stay at 0", errs[0])

    def test_register_rows_counts_missing_and_unknown_checks(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "scripts/ok.py", "")
            self._write(root, "handbook/guidelines/t-checklist.md", "# T\n\n- **R8** A rule.\n")
            self._write(root, "crates/x/Cargo.toml", "")
            self._write(root, "crates/cpl-cli/src/lib.rs", 'Some("doc") => doc(),\n"fmt" => fmt,\n')
            self._write(root, "crates/cpl-conform/src/lib.rs", '"corpus" => Some(Suite::Corpus),\n')
            self._write(root, "handbook/domains/x.md", "\n".join([
                "| Attribute | Check |", "|---|---|",
                "| outside | `missing` |",
                "## Quality register",
                "| Attribute | Metric | Check |", "|---|---|---|",
                "| a | m | `scripts/ok.py` (`name`) |",
                "| f | m | `cargo nextest run -p x`, `gaps.txt` |",
                "| b | m | `missing` |",
                "| c | m | `scripts/gone.py` |",
                "| d | m | process |",
                "### Sub", "| A | M | C |", "|---|---|---|", "| g | m | `missing` |",
                "| h | m | `scripts/ok.py` | enforced |",
                "| k | m | a generator | missing |",
                "| l | m | `vm_fallbacks` ratchet | enforced |",
                "| m | m | `handbook/guidelines/t-checklist.md#r8`, `scripts/ok.py` |",
                "| n | m | `handbook/guidelines/t-checklist.md#r8` |",
                "| o | m | `handbook/guidelines/t-checklist.md#r9`, `scripts/ok.py` |",
                "| p | m | `cpl doc` |",
                "| q | m | `cpl fmt --check` |",
                "| r | m | `cpl nosuch --check` |",
                "| s | m | `cpl-conform corpus`, `cpl-conform bogus` |",
                "| t | m | `cargo nextest run -p y` |",
                "| u | m | `cargo build -p x` |",
                "## Suites", "| e | `missing` |", ""]))
            self.assertEqual(dr.register_rows(root), [
                "handbook/domains/x.md:9 missing: b",
                "handbook/domains/x.md:10 no such check scripts/gone.py: c",
                "handbook/domains/x.md:15 missing: g",
                "handbook/domains/x.md:17 missing: k",
                "handbook/domains/x.md:20 missing: n",
                "handbook/domains/x.md:21 no such check handbook/guidelines/t-checklist.md#r9: o",
                "handbook/domains/x.md:22 missing: p",
                "handbook/domains/x.md:24 no such check cpl nosuch --check: r",
                "handbook/domains/x.md:25 no such check cpl-conform bogus: s",
                "handbook/domains/x.md:26 no such check cargo nextest run -p y: t",
                "handbook/domains/x.md:27 missing: u",
            ])

    def test_written_twice_counts_hand_kept_copies_of_a_sourced_list(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            names = [f"m{i}" for i in range(10)]
            for n in names:
                self._write(root, f"std/{n}.cp", "")
            self._write(root, "gen.toml", '[[gen]]\nfiles = ["docs/gen.md"]\n')
            mods = " ".join(f"std.{n}" for n in names)
            self._write(root, "docs/copy.md", mods)
            self._write(root, "docs/gen.md", mods)
            self._write(root, "plan/notes.md", mods)
            self._write(root, "docs/few.md", " ".join(f"std.{n}" for n in names[:4]))
            self._write(root, "docs/words.md", " ".join(names))
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(["git", "-C", str(root), "add", "."], check=True)
            self.assertEqual(dr.written_twice(root), ["docs/copy.md: 10 of 10 std modules"])

    def test_register_rows_by_domain_only_go_down(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            row = ["## Quality register", "| A | M | C |", "|---|---|---|"]
            self._write(root, "handbook/domains/a.md", "\n".join(row + ["| x | m | `missing` |", "| y | m | `missing` |", ""]))
            self._write(root, "handbook/domains/b.md", "\n".join(row + [""]))
            by = dr.register_rows_by_domain(root)
            self.assertEqual(by, {"a": 2, "b": 0})
            grew = dr.evaluate({"register_rows_by_domain": by}, {"register_rows_by_domain": {"a": 1}})[0]
            self.assertEqual(len(grew), 1)
            self.assertIn("a.md", grew[0])
            self.assertEqual(dr.evaluate({"register_rows_by_domain": by}, {"register_rows_by_domain": {"a": 3, "b": 0}})[0], [])
            self.assertEqual(dr.improved({"register_rows_by_domain": by}, {"register_rows_by_domain": {"a": 3}}),
                             ["register_rows_by_domain"])

    def test_authority_drift_wants_the_sentence_and_no_contrary_order(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for f in dr.AUTHORITY_HOMES:
                self._write(root, f, "# X\n\nThe rule: handbook leads design; `spec/` is its\ndownstream exhaustive form.\n")
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            subprocess.run(["git", "add", "."], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.authority_drift(root), [])
            self._write(root, "spec/README.md", "spec/ is the normative language.\n")
            self._write(root, "handbook/language/a.md", "The spec states every rule and\nwins wherever the two differ.\n")
            subprocess.run(["git", "add", "."], cwd=root, check=True, env=dr._git_env())
            found = dr.authority_drift(root)
            self.assertEqual(len(found), 3, found)
            self.assertIn("authority_drift", dr.HARD_ZERO)

    def test_conformance_counters_read_the_scoreboard(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.assertEqual(len(dr.conformance_failing(root)), 1)
            self.assertEqual(len(dr.conformance_wrong_output(root)), 1)
            board = {"schema_version": 2, "suites": {"conformance": {
                "total": 9, "passed": 1,
                "cases": {"a.cp": "pass", "b.cp": "fail", "c.cp": "bad", "d.cp": "timeout"}
                | {f"w{i}.cp": "fail" for i in range(5)},
                "causes": [{"cause": "wrong output", "count": 6, "examples": [f"w{i}.cp" for i in range(5)]},
                           {"cause": "E0302: x", "count": 1, "examples": ["b.cp"]}]}}}
            self._write(root, dr.SCOREBOARD, json.dumps(board))
            failing = dr.conformance_failing(root)
            self.assertEqual(len(failing), 8, failing)
            self.assertIn("c.cp: bad", failing)
            wrong = dr.conformance_wrong_output(root)
            self.assertEqual(len(wrong), 6, wrong)
            self.assertEqual(wrong[0], "w0.cp")
            board["suites"]["conformance"]["cases"] = {"a.cp": "pass"}
            board["suites"]["conformance"]["causes"] = []
            self._write(root, dr.SCOREBOARD, json.dumps(board))
            self.assertEqual(dr.conformance_failing(root), [])
            self.assertEqual(dr.conformance_wrong_output(root), [])

    def test_tools_tests_over_30s_reads_the_test_times_record(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.assertEqual(len(dr.tools_tests_over_30s(root)), 1)
            self._write(root, dr.TOOLS_TEST_TIMES, json.dumps({"over_limit": {"cpl-cli::a slow": 31.0}}))
            self.assertEqual(dr.tools_tests_over_30s(root), ["cpl-cli::a slow: 31.0 s"])

    def test_corpus_max_short_and_unread_read_the_max_short_record(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.assertEqual(len(dr.corpus_max_short(root)), 1)
            self.assertEqual(len(dr.corpus_max_unread(root)), 1)
            self._write(root, dr.CORPUS_MAX_SHORT, json.dumps({"max_short": ["max-short a m"], "max_unread": []}))
            self.assertEqual(dr.corpus_max_short(root), ["max-short a m"])
            self.assertEqual(dr.corpus_max_unread(root), [])

    def test_dream_over_32kb_reads_the_dream_sizes_record(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.assertEqual(len(dr.dream_over_32kb(root)), 1)
            self._write(root, dr.DREAM_OVER_32KB, json.dumps({"dream_over_32kb": ["a"]}))
            self.assertEqual(dr.dream_over_32kb(root), ["a"])

    def test_interp_max_over_20s_reads_the_corpus_board(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.assertEqual(len(dr.interp_max_over_20s(root)), 1)
            self._write(root, dr.SCOREBOARD, json.dumps({"suites": {"corpus": {"cases": {}}}}))
            self.assertEqual(len(dr.interp_max_over_20s(root)), 1)
            board = {"suites": {"corpus": {"cases": {}, "interp_max_over_20s": ["a", "b"]}}}
            self._write(root, dr.SCOREBOARD, json.dumps(board))
            self.assertEqual(dr.interp_max_over_20s(root), ["a", "b"])

    def test_why_written_twice_counts_copied_reasoning_and_spec_rationale(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            why = "A lend lasts until its last use, so moving a print can break a write that the line after it makes legal again."
            self._write(root, "handbook/language/a.md", f"# A\n\n{why}\n")
            self._write(root, "spec/core/01-a.md", f"# A\n\n> {why}\n\n```\n{why}\n```\n")
            self._write(root, "decisions.md", "# D\n\nShort reason.\n")
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            subprocess.run(["git", "add", "."], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.why_written_twice(root), [])
            self._write(root, "spec/lib/b.md", f"{why}\n")
            self._write(root, "spec/lib/rationale/c.md", "x\n")
            subprocess.run(["git", "add", "."], cwd=root, check=True, env=dr._git_env())
            found = dr.why_written_twice(root)
            self.assertEqual(len(found), 2, found)
            self.assertIn("handbook/language/a.md = spec/lib/b.md", found[0])
            self.assertIn("why_written_twice", dr.HARD_ZERO)

    def test_uncited_rules_needs_an_existing_heading_anchor(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "handbook/guidelines/lints.md", "# Lints\n## The rules\n")
            self._write(root, "spec/lints/good/lint.toml", 'rule = "handbook/guidelines/lints.md#the-rules"\n')
            self._write(root, "spec/lints/bad/explain.md", "see handbook/guidelines/lints.md#gone\n")
            self._write(root, "spec/lints/none/lint.toml", "code = 1\n")
            self._write(root, "handbook/guidelines/std-api-checklist.md", "# x\n- **std-api/R12** Searches return ?T.\n")
            self._write(root, "spec/lints/byid/lint.toml", 'rule = "std-api/R12"\n')
            self._write(root, "spec/lints/byanchor/explain.md", "handbook/guidelines/std-api-checklist.md#r12\n")
            self._write(root, "spec/lints/noid/lint.toml", 'rule = "std-api/R13"\n')
            self._write(root, "scripts/check.sh", "python3 scripts/a.py\npython3 scripts/b.py scripts/test_x.py\n")
            self._write(root, "scripts/a.py", '"""Gate. Rule: handbook/guidelines/lints.md#the-rules."""\n')
            self._write(root, "scripts/b.py", '"""Gate."""\n')
            # The runners name gates and are not one.
            self._write(root, "scripts/ci.py", '"""Runner."""\nSTEP = "scripts/ci.py steps checks"\n')
            self.assertEqual(dr.uncited_rules(root), ["lint bad", "lint noid", "lint none", "gate scripts/b.py"])

    def test_unreferenced_scripts_are_the_files_nothing_else_names(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            env = dr._git_env()
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=env)
            for rel, text in {
                "justfile": "run:\n    scripts/run.py\n    scripts/used.sh\n",
                "scripts/run.py": 'import gates.gate_one\nGATES = ["two-words"]\n',
                "scripts/used.sh": "",
                "scripts/dead.sh": "",
                "scripts/test_dead.py": "# runs scripts/dead.sh\n",
                "scripts/old_used.sh": "",
                "scripts/gates/__init__.py": "",
                "scripts/gates/gate_one.py": "",
                "scripts/gates/two_words.py": "",
                "scripts/gates/gone.py": "",
                "scripts/orphans/__init__.py": "",
                "scripts/orphans/lost.py": "",
            }.items():
                self._write(root, rel, text)
            subprocess.run(["git", "-C", str(root), "add", "-A"], check=True, env=env)
            self.assertEqual(dr.unreferenced_scripts(root), [
                "scripts/dead.sh", "scripts/gates/gone.py", "scripts/old_used.sh",
                "scripts/orphans/__init__.py", "scripts/orphans/lost.py"])

    def test_upstream_skips_a_counter_its_baseline_does_not_have_yet(self) -> None:
        def needs_a_record(_root):
            raise dr.MeasureError("research/new.txt is missing")
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            env = dr._git_env()
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=env)
            (cpl := root.parent / "cpl").write_text("")
            self.enterContext(mock.patch.dict(os.environ, {"CPL": str(cpl)}))
            self.enterContext(mock.patch.dict(dr.COUNTERS, {"brand_new": ("new", needs_a_record, True)}))
            self._write(root, "scripts/debt-ratchet.json", json.dumps({"rust_ignore": 0}))
            for args in (["add", "-A"], ["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "x"]):
                subprocess.run(["git", "-C", str(root), *args], check=True, env=env)
            up = dr._measure_at(root, "HEAD", 20)
            self.assertIn("rust_ignore", up)
            self.assertNotIn("brand_new", up)

    def test_upstream_is_measured_once_per_tree_across_worktrees_and_only_what_is_asked(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "main"
            env = dr._git_env()
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=env)
            self._write(root, "a.txt", "a\n")
            for args in (["add", "-A"], ["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "x"]):
                subprocess.run(["git", "-C", str(root), *args], check=True, env=env)
            other = Path(tmp) / "other"
            subprocess.run(["git", "-C", str(root), "worktree", "add", "-q", "--detach", str(other)],
                           check=True, env=env)
            asked = []

            def measured(_root, _rev, _max, only):
                asked.append(sorted(only))
                return {n: 7 for n in only}
            self.enterContext(mock.patch.object(dr, "_measure_at", side_effect=measured))
            self.enterContext(mock.patch.dict(os.environ, {"CI": ""}))
            self.assertEqual(dr.measure_upstream(root, 20, ref="HEAD", only={"skip_lists"}), {"skip_lists": 7})
            # Another worktree's land on the same tree reads it; a counter no land
            # has asked for yet is the only one measured.
            self.assertEqual(dr.measure_upstream(other, 20, ref="HEAD", only={"skip_lists"}), {"skip_lists": 7})
            self.assertEqual(dr.measure_upstream(other, 20, ref="HEAD", only={"skip_lists", "rust_ignore"}),
                             {"skip_lists": 7, "rust_ignore": 7})
            self.assertEqual(asked, [["skip_lists"], ["rust_ignore"]])

    def test_upstream_on_a_fresh_tip_is_measured_except_in_ci(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            env = dr._git_env()
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=env)
            (cpl := root.parent / "cpl").write_text("")
            self.enterContext(mock.patch.dict(os.environ, {"CPL": str(cpl)}))
            self._write(root, "crates/demo/src/a.rs", "fn a() {}\n")
            for args in (["add", "-A"], ["-c", "user.name=t", "-c", "user.email=t@t",
                                         "commit", "-qm", "x"]):
                subprocess.run(["git", "-C", str(root), *args], check=True, env=env)
            old = os.environ.pop("CI", None)
            try:
                first = dr.measure_upstream(root, 20, ref="HEAD")
                self.assertIsNotNone(first)
                with mock.patch.object(dr, "_measure_at", side_effect=AssertionError("measured twice")):
                    self.assertEqual(dr.measure_upstream(root, 20, ref="HEAD"), json.loads(json.dumps(first)))
                os.environ["CI"] = "true"
                self.assertIsNone(dr.measure_upstream(root, 20, ref="HEAD"))
            finally:
                os.environ.pop("CI", None)
                if old is not None:
                    os.environ["CI"] = old

    def test_a4_documented_edge_is_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "handbook/design/architecture.md",
                        "## Crates\n| Crate | Job |\n|---|---|\n"
                        "| `cpl-diag` | renderers; JSON through `cpl-json` |\n"
                        "| `cpl-fmt`, `cpl-lint` | tools over `cpl-check` queries |\n"
                        "## A5. Other\n| `cpl-x` | uses cpl-y |\n")
            self.assertEqual(dr.a4_edges(root), [
                "cpl-diag -> cpl-json", "cpl-fmt -> cpl-check", "cpl-lint -> cpl-check"])
        baseline = {"crate_edges": ["cpl-a -> cpl-b"]}
        actual = {"crate_edges": ["cpl-a -> cpl-b", "cpl-diag -> cpl-json", "cpl-a -> cpl-c"],
                  "a4_edges": ["cpl-diag -> cpl-json"]}
        errs, _, notes = dr.evaluate(actual, baseline)
        self.assertEqual(len(errs), 1)
        self.assertIn("cpl-a -> cpl-c", errs[0])
        self.assertTrue(any("cpl-diag -> cpl-json" in n for n in notes))

    def test_evaluate_counters_and_edges(self) -> None:
        baseline = {"near_dup_fns": 3, "allow_attrs": 9, "crate_edges": ["a -> b", "a -> c"]}
        actual = {"near_dup_fns": 4, "allow_attrs": 9, "crate_edges": ["a -> b", "a -> d"]}
        errs, _, notes = dr.evaluate(actual, baseline)
        self.assertEqual(len(errs), 2)
        self.assertIn("near-identical", errs[0])
        self.assertIn("a -> d", errs[1])
        self.assertEqual(notes, ["crate edge a -> c removed"])

    def test_the_perf_counters_read_the_bar_record_and_an_edit_alone_clears_nothing(self) -> None:
        import hashlib
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self.assertEqual(dr.ports_over_bar(root), [], "no corpus, nothing to time")
            (root / "corpus/problems/p").mkdir(parents=True)
            (root / "corpus/problems/p/orig.cpp").write_text("")
            with self.assertRaisesRegex(dr.MeasureError, "--record"):
                dr.ports_over_bar(root)
            sha = {}
            for pid in ("slow", "fat", "fine", "tuned", "edited", "retuned"):
                (root / "corpus/problems" / pid).mkdir(parents=True)
                (root / "corpus/problems" / pid / "port.cp").write_text(pid)
                sha[pid] = hashlib.sha256(pid.encode()).hexdigest()[:12]
            for pid in ("fine", "tuned", "slow", "retuned"):
                (root / "corpus/problems" / pid / "perf.cp").write_text("")
            (root / "research/perf").mkdir(parents=True)
            (root / "research/perf/v03-bar.txt").write_text(
                f"# header\nslow {sha['slow']} 1.4 1.0\nfat {sha['fat']} 1.0 1.2\nfine {sha['fine']} 1.1 -\n"
                f"tuned {sha['tuned']} 0.9 1.1\nedited {sha['fat']} 2.0 2.0\nretuned {sha['fat']} 0.5 0.5\n")
            self.assertEqual(dr.ports_over_bar(root), [
                "slow time 1.4x rss 1.0x", "fat time 1.0x rss 1.2x",
                f"edited time 2.0x rss 2.0x (port.cp edited since: run {dr.BAR_CMD})"])
            self.assertEqual(dr.perf_twins_stale(root), ["corpus/problems/fine/perf.cp: port.cp time 1.1x rss 0.0x",
                                                         "corpus/problems/tuned/perf.cp: port.cp time 0.9x rss 1.1x"])

    def test_cli_show_and_list(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._make_repo(root)
            self.assertEqual(dr.main(["--show", "rust_ignore", "--root", str(root)]), 0)
            self.assertEqual(dr.main(["--list", "stdlib_todo", "--root", str(root)]), 0)
            self.assertEqual(dr.main(["--list", "nope", "--root", str(root)]), 2)

    def test_a_counter_that_cannot_measure_is_one_error_line_and_the_rest_still_print(self) -> None:
        def stale(_root):
            raise dr.MeasureError("cpl: target/debug/cpl is stale: run cargo build -p cpl-cli")
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._make_repo(root)
            label, _, gated = dr.COUNTERS["corpus_fmt_dirty"]
            with mock.patch.dict(dr.COUNTERS, {"corpus_fmt_dirty": (label, stale, gated)}):
                for flags in (["--show", "corpus_fmt_dirty", "rust_ignore"], ["--zero"]):
                    out = io.StringIO()
                    with contextlib.redirect_stdout(out):
                        self.assertEqual(dr.main([*flags, "--root", str(root)]), 1)
                    self.assertIn("corpus_fmt_dirty ERROR cpl: target/debug/cpl is stale: run cargo build -p cpl-cli",
                                  out.getvalue())
                    self.assertNotIn("Traceback", out.getvalue())
                    if flags[0] == "--show":
                        self.assertRegex(out.getvalue(), r"\nrust_ignore \d+\n")

    def test_std_gaps_counts_each_open_line(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self.assertEqual(dr.std_gaps(root), [])
            self._write(root, "std/GAPS.md", "# Gaps\n\nWhy.\n\n- seq: a\n- tree: b\n")
            self.assertEqual(dr.std_gaps(root), ["std/GAPS.md:5", "std/GAPS.md:6"])

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
                                           "stdlib_todo": 99, "allow_attrs": 7}))
            self.assertEqual(dr.main(["--update", "stdlib_todo", "--root", str(root), "--ratchet", str(ratchet)]), 0)
            data = json.loads(ratchet.read_text())
            self.assertEqual(data["stdlib_todo"], 0)
            self.assertEqual(data["allow_attrs"], 7)
            self.assertEqual(data["rs_file_size_allowlist"], {"a.rs": 9000})
            self.assertEqual(dr.main(["--update", "nope", "--root", str(root), "--ratchet", str(ratchet)]), 2)

    def test_cli_roundtrip_update_and_check(self) -> None:
        # _make_repo writes 2 stdlib TODOs: main's stdlib_todo is a hard zero, this one isn't.
        with tempfile.TemporaryDirectory() as d, mock.patch.object(dr, "HARD_ZERO", dr.HARD_ZERO - {"stdlib_todo"}):
            root = Path(d)
            self._make_repo(root)
            ratchet = root / "ratchet.json"
            # A hard-zero counter above 0 is never written as a baseline.
            self.assertEqual(dr.main(["--update", "--root", str(root), "--ratchet", str(ratchet)]), 1)
            (root / "crates/cpl-cli/tests/pending/case1.cp").unlink()
            (root / "crates/demo/src/small.rs").write_text("fn t() {}\n")
            (root / "crates/cpl-ir/src").mkdir(parents=True)
            (root / "crates/cpl-ir/src/instance.rs").write_text("use codes::GROWING_INSTANTIATION;\n")
            (root / dr.SCALING_BOARD).parent.mkdir(parents=True)
            (root / dr.SCALING_BOARD).write_text('{"cases": {}, "unscaled": []}')
            (root / dr.CORPUS_MAX_SHORT).parent.mkdir(parents=True, exist_ok=True)
            (root / dr.CORPUS_MAX_SHORT).write_text('{"max_short": [], "max_unread": []}')
            board = root / dr.SCOREBOARD
            board.parent.mkdir(parents=True, exist_ok=True)
            board.write_text('{"suites": {"corpus": {"interp_max_over_20s": []}, "conformance": {"cases": {}, "causes": []}}}')
            self.assertEqual(dr.main(["--update", "--root", str(root), "--ratchet", str(ratchet)]), 0)
            self.assertTrue(json.loads(ratchet.read_text())["stdlib_todo"] == 2)
            self.assertEqual(dr.main(["--check", "--root", str(root), "--ratchet", str(ratchet)]), 0)
            # An improved counter is lowered to its measured value by `--tighten`, not left as headroom.
            data = json.loads(ratchet.read_text())
            ratchet.write_text(json.dumps({**data, "stdlib_todo": 5}))
            self.assertEqual(dr.main(["--tighten", "--root", str(root), "--ratchet", str(ratchet)]), 0)
            self.assertEqual(json.loads(ratchet.read_text())["stdlib_todo"], 2)


    def test_update_refuses_to_raise_a_baseline_above_main(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            git = lambda *a: subprocess.run(["git", *a], cwd=root, check=True, capture_output=True,
                                            env=dr._git_env())
            git("init", "-q")
            (root / "scripts").mkdir()
            (root / "scripts/debt-ratchet.py").write_text(Path(dr.__file__).read_text(encoding="utf-8"))
            ratchet = root / "scripts/debt-ratchet.json"
            ratchet.write_text(json.dumps({"allow_attrs": 5, "rs_file_size_allowlist": {}}))
            git("add", ".")
            git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "base")
            args = ["--update", "allow_attrs", "--root", str(root), "--ratchet", str(ratchet)]
            for count, code in ((7, 1), (4, 0)):
                with mock.patch.object(dr, "measure", return_value={"allow_attrs": count,
                                                                      "rs_file_size_allowlist": {}}), \
                        mock.patch.dict(os.environ, {"CPL_RATCHET_BASE": "HEAD"}):
                    self.assertEqual(dr.main(args), code)
                self.assertEqual(json.loads(ratchet.read_text())["allow_attrs"], 5 if code else 4)

    def test_debt_already_on_main_is_not_this_changes_error(self):
        base = {"ir_reinfer": 10, "rs_file_size_allowlist": {"a.rs": 2100}}
        main = {"ir_reinfer": 12, "rs_file_size_allowlist": {"a.rs": 2200}}
        eff = dr.inherited_baseline(base, main)
        same = {"ir_reinfer": 12, "rs_file_size_allowlist": {"a.rs": 2200}}
        self.assertEqual(dr.evaluate(same, eff)[0], [])
        more = {"ir_reinfer": 13, "rs_file_size_allowlist": {"a.rs": 2201}}
        self.assertEqual(len(dr.evaluate(more, eff)[0]), 2)
        self.assertEqual(base["ir_reinfer"], 10)

    def test_check_fails_debt_already_on_main_unless_lowered(self):
        with tempfile.TemporaryDirectory() as d:
            ratchet = Path(d, "r.json")
            ratchet.write_text(json.dumps({"ir_reinfer": 10, "rs_file_size_allowlist": {}}))
            now = {"ir_reinfer": 12, "rs_file_size_allowlist": {}}
            for name, fake in (("measure", lambda *a, **k: dict(now)), ("measure_upstream", lambda *a, **k: dict(now)),
                               ("unexplained_bumps", lambda *a, **k: []), ("hand_lowered", lambda *a, **k: [])):
                self.enterContext(mock.patch.object(dr, name, fake))
            argv = ["--check", "--root", d, "--ratchet", str(ratchet)]
            # Main's debt fails a push that doesn't lower it, strict or not.
            for strict in ("", "1"):
                with mock.patch.dict(os.environ, {"CPL_RATCHET_STRICT": strict}):
                    self.assertEqual(dr.main(argv), 1)
            # A push that lowers it goes through off main.
            self.enterContext(mock.patch.object(dr, "measure", lambda *a, **k: {**now, "ir_reinfer": 11}))
            with mock.patch.dict(os.environ, {"CPL_RATCHET_STRICT": ""}):
                self.assertEqual(dr.main(argv), 0)

    def test_last_touched_never_blames_a_shallow_clones_boundary(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            src, clone = Path(d, "src"), Path(d, "clone")
            src.mkdir()
            git = lambda root, *a: subprocess.run(["git", "-C", str(root), *a], check=True, capture_output=True,
                                                  text=True, env=dr._git_env()).stdout
            git(src, "init", "-q")
            for msg, name in (("grow a.rs", "a.rs"), ("touch b.rs", "b.rs")):
                Path(src, name).write_text(msg + "\n")
                git(src, "add", name)
                git(src, "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", msg)
            self.assertEqual([l.split(" ", 1)[1] for l in dr.last_touched(src, "a.rs")], ["grow a.rs"])
            git(Path(d), "clone", "-q", "--depth", "1", f"file://{src}", str(clone))
            self.assertEqual(dr.last_touched(clone, "a.rs"), [])

    def test_lint_dup_helpers_and_vm_fallbacks(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            rules = root / "crates/cpl-lint/src/rules"
            rules.mkdir(parents=True)
            (rules / "a.rs").write_text("fn check() {}\nfn squash() {}\nimpl X {\n    fn inner() {}\n}\n")
            (rules / "b.rs").write_text("pub fn check() {}\npub(crate) fn squash() {}\nfn inner() {}\n")
            self.assertEqual(dr.lint_dup_helpers(root),
                             ["squash: crates/cpl-lint/src/rules/a.rs, crates/cpl-lint/src/rules/b.rs"])
            src = root / "crates/cpl-interp/src"
            (src / "vm/compile").mkdir(parents=True)
            (src / "vm/compile/a.rs").write_text(
                "    fn emit_fallback_stmt(&mut self) {}\n        self.emit_fallback_expr_v(expr)\n"
                "        // self.emit_fallback_stmt(stmt)\n")
            (src / "vm/run.rs").write_text("        let v = self.eval(expr);\n")
            (src / "call.rs").write_text("            _ => self.block(&f.body),\n        self.eval(e)\n")
            self.assertEqual(dr.vm_fallbacks(root), [
                "crates/cpl-interp/src/call.rs:1",
                "crates/cpl-interp/src/vm/run.rs:1",
                "crates/cpl-interp/src/vm/compile/a.rs:2",
            ])

    def test_codegen_effect_guesses(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            src = root / "crates/cpl-codegen/src"
            (src / "uses").mkdir(parents=True)
            (src / "uses/effects.rs").write_text("use cpl_ir::effects::pure_path;\nfn pure_fns() {}\n")
            (src / "a_tests.rs").write_text("use cpl_ir::effects::pure_path;\n")
            (src / "lend.rs").write_text(
                "use cpl_ir::effects::pure_path;\n"
                "// cpl_ir::effects in a comment\n"
                "fn writes_nothing(e: &Expr) -> bool {\n"
                "    walk::deep_expr(e, &mut |x| {\n"
                "        hit |= matches!(x.kind, ExprKind::CallExpr(..))\n"
                "    });\n"
                "    walk::deep_exprs(b, &mut |x| match &x.kind {\n"
                "        ExprKind::Closure(c) => found |= c == 1,\n"
                "        _ => {}\n"
                "    });\n"
                "}\n")
            self.assertEqual(dr.codegen_effect_guesses(root),
                             ["crates/cpl-codegen/src/lend.rs:1", "crates/cpl-codegen/src/lend.rs:3",
                              "crates/cpl-codegen/src/lend.rs:4"])

    def test_opt_without_uses(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            src = root / "crates/cpl-codegen/src"
            (src / "uses").mkdir(parents=True)
            (src / "uses/mod.rs").write_text("fn a() { walk::deep_exprs(b, &mut |e| {}); }\n")
            (src / "walk.rs").write_text("pub fn deep_exprs() {}\n")
            (src / "fuse.rs").write_text("use crate::uses::Touch;\nfn a() { walk::deep_exprs(b, &mut |e| {}); }\n")
            (src / "presize.rs").write_text("fn a() { walk::deep_exprs(b, &mut |e| {}); }\n")
            (src / "stray.rs").write_text("fn a() { walk::deep_stmts(b, &mut |s| {}); }\n")
            (src / "a_tests.rs").write_text("fn a() { walk::deep_exprs(b, &mut |e| {}); }\n")
            (src / "passes.rs").write_text(
                'pass("loop-fusion", "fuse.rs", Uses(&["Touch"]), "t"),\n'
                'pass("presize", "presize.rs", Unasked, "t"),\n')
            walk = "walks IR itself, not through a named uses/ query"
            self.assertEqual(dr.opt_without_uses(root),
                             [f"crates/cpl-codegen/src/fuse.rs:2: {walk}",
                              "crates/cpl-codegen/src/presize.rs: presize asks no Uses query",
                              f"crates/cpl-codegen/src/presize.rs:1: {walk}",
                              "crates/cpl-codegen/src/stray.rs: walks IR with no PASSES row",
                              f"crates/cpl-codegen/src/stray.rs:1: {walk}"])

    def test_stress_hand_json_and_helper_env_vars(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            files = {
                "crates/cpl-stress/src/cli.rs": 'let j = format!("{{\\"case\\":{}}}", n);\nlet s = cpl_json::escape_string(x);\n',
                "crates/cpl-wasm/src/lib.rs": 'let ok = cpl_json::to_string(&v);\n',
                "crates/cpl-cli/src/dap.rs": 'format!("{{\\"seq\\":1}}")\n',
                "crates/cpl-stress/src/choices.rs": 'run(g, &[("CPL_SEED", s)]);\np.env.push(("CPL_CHOICES".to_string(), x));\n',
                "crates/cpl-codegen/src/runtime.rs": 'std::getenv(\\"CPL_CHOICES_OUT\\")\n',
                "crates/cpl-cli/src/toolchain.rs": 'command.env("CPL_TOOLCHAIN", &id);\n',
            }
            for rel, text in files.items():
                (root / rel).parent.mkdir(parents=True, exist_ok=True)
                (root / rel).write_text(text)
            self.assertEqual(dr.stress_hand_json(root), ["crates/cpl-stress/src/cli.rs:1", "crates/cpl-stress/src/cli.rs:2"])
            self.assertEqual(dr.helper_env_vars(root), [
                "CPL_CHOICES: crates/cpl-stress/src/choices.rs:2",
                "CPL_CHOICES_OUT: crates/cpl-codegen/src/runtime.rs:1",
            ])

    def test_ir_reinfer_counts_lowering_probes(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            src = root / "crates/cpl-ir/src/lower/a.rs"
            src.parent.mkdir(parents=True)
            src.write_text("let t = self.probe_expr(e, None).ty;\nfn probe_expr(&mut self) {}\n"
                           "let u = self.probe_expr_ty(e);\n")
            tests = root / "crates/cpl-ir/tests/a.rs"
            tests.parent.mkdir(parents=True)
            tests.write_text("x.probe_expr(e, None);\n")
            self.assertEqual(dr.ir_reinfer(root),
                             ["crates/cpl-ir/src/lower/a.rs:1", "crates/cpl-ir/src/lower/a.rs:3"])

    def test_next_string_keyed_maps_counts_text_keyed_tables_in_cpl_next(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            src = root / "crates/cpl-next-check/src/a.rs"
            src.parent.mkdir(parents=True)
            src.write_text("x: HashMap<String, FnId>,\ny: HashSet<&'a str>,\nz: BTreeMap<FnId, String>,\n"
                           "w: Vec<(String, Ty)>,\n// v: HashMap<String, u32>,\n")
            (root / "crates/cpl-next-decls/src").mkdir(parents=True)
            (root / "crates/cpl-next-decls/src/sym.rs").write_text("ids: HashMap<String, Sym>,\n")
            (root / "crates/cpl-check/src").mkdir(parents=True)
            (root / "crates/cpl-check/src/b.rs").write_text("m: HashMap<String, Ty>,\n")
            self.assertEqual(dr.next_string_keyed_maps(root),
                             ["crates/cpl-next-check/src/a.rs:1", "crates/cpl-next-check/src/a.rs:2",
                              "crates/cpl-next-check/src/a.rs:4"])

    def test_string_keyed_maps_counts_name_keyed_hash_maps_in_cpl_check(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            src = root / "crates/cpl-check/src/a.rs"
            src.parent.mkdir(parents=True)
            src.write_text("x: HashMap<String, Ty>,\ny: HashMap<&'a str, usize>,\nz: HashMap<u32, String>,\n"
                           "w: NameMap<Ty>,\n")
            (root / "crates/cpl-check/src/names.rs").write_text("index: HashMap<String, u32>,\n")
            self.assertEqual(dr.string_keyed_maps(root),
                             ["crates/cpl-check/src/a.rs:1", "crates/cpl-check/src/a.rs:2"])

    def test_own_databases_and_copy_baselines(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            for rel in ("crates/cpl-lsp/src/server.rs", "crates/cpl-lsp/src/fix_tests.rs",
                        "crates/cpl-lsp/tests/a.rs", "crates/cpl-check/src/db.rs"):
                (root / rel).parent.mkdir(parents=True, exist_ok=True)
                (root / rel).write_text("let db = Database::new();\nlet s = Session::new();\n")
            (root / "crates/cpl-ir/src/a.rs").parent.mkdir(parents=True)
            (root / "crates/cpl-ir/src/a.rs").write_text("Database::with_loader(l)\n")
            self.assertEqual(sorted(dr.own_databases(root)),
                             ["crates/cpl-ir/src/a.rs:1", "crates/cpl-lsp/src/fix_tests.rs:1",
                              "crates/cpl-lsp/src/server.rs:1", "crates/cpl-lsp/tests/a.rs:1"])
            errors, _, _ = dr.evaluate({"own_databases": 1}, {"own_databases": 5})
            self.assertTrue(any(re.search(r"must (stay at|be) 0", e) for e in errors), errors)
            case = root / "spec/conformance/core-05/a.cp"
            case.parent.mkdir(parents=True)
            case.write_text("// expect: ok\n// copies: 2\n// std_functions: 1\n// sunk_copies: 2\nprint(1)\n// copies: 9\n")
            port = root / f"{PKGS}/cf1/port.cp"
            port.parent.mkdir(parents=True)
            port.write_text("// copies: 1\nprint(1)\n")
            (root / f"{PKGS}/cf1/notes.md").write_text("// copies: 5\n")
            self.assertEqual(dr.cpp_deep_copies(root), [f"{PKGS}/cf1/port.cp 0"])
            self.assertEqual(len(dr.spec_required_copies(root)), 2)
            self.assertEqual(len(dr.cpp_std_functions(root)), 1)
            self.assertEqual(len(dr.cpp_sunk_copies(root)), 2)

    def test_explain_copy_mismatches_lists_cases_whose_explain_and_cpp_differ(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self.assertEqual(dr.explain_copy_mismatches(root), [])
            for name, cpp in (("same", "auto b = std::vector<ll>(a);"), ("hides", "auto b = a;")):
                case = root / f"spec/conformance/core-05/{name}.cp"
                case.parent.mkdir(parents=True, exist_ok=True)
                case.write_text(f"// expect: ok\n// {cpp}\n")
            cpl = root / "cpl"
            cpl.write_text("#!/usr/bin/env python3\nimport sys\nsrc = open(sys.argv[2]).read().splitlines()[1][3:]\n"
                           "if sys.argv[1] == 'explain': print('[copy] b')\n"
                           "else: open(sys.argv[sys.argv.index('-o') + 1], 'w').write(src + '\\n')\n")
            cpl.chmod(0o755)
            with mock.patch.object(dr, "_built_cpl", return_value=cpl):
                self.assertEqual(dr.explain_copy_mismatches(root),
                                 ["spec/conformance/core-05/hides.cp: explain lists 1, the C++ makes 0"])

    def test_std_shadowed_counts_receivers_a_builtin_arm_catches(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "crates/cpl-ir/src/lower/call.rs",
                        '            "runs" | "take" | "pop" | "len" => {}\n')
            self._write(root, "crates/cpl-check/src/prelude.rs",
                        'fn ty(n: &str) { match n { "Subset" => {}, _ => {} } }\n'
                        '#[cfg(test)]\nmod tests { const X: &str = "Heap"; }\n')
            self._write(root, "crates/cpl-check/src/lib.rs", "#[cfg(test)]\nmod tests_messages;\n")
            self._write(root, "crates/cpl-check/src/tests_messages.rs", 'const V: &str = "Vec";\n')
            self._write(root, "plan/domains/tech-debt/architecture/builtin-floor.md",
                        "| Storage | intrinsic | `len` |\n")
            self._write(root, "std/seq.cp",
                        "pub fn runs<T>(self: List<T>) -> Int = 0\n"
                        "pub fn take<S>(self: S, k: Int) -> S = self\n"
                        "@get pub fn len<T>(self: List<T>) -> Int = 0\n"
                        "pub struct Subset<B>(bits: U64):\n"
                        "    pub fn take(self, k: Int) -> Int = k\n")
            self._write(root, "std/heap.cp",
                        "pub struct Heap<T>():\n    pub fn pop(mut self) -> ?T = None\n"
                        "pub fn pop(self: Heap<Int>) -> Int = 0\n")
            self._write(root, "std/vec.cp", "pub struct Vec<T>():\n")
            self.assertEqual(dr.std_shadowed(root), ["Subset", "runs", "take"])

    def test_std_names_outside_rt_skips_flag_values_not_lowering(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            self._write(root, "std/seq.cp", "pub fn all<S>(self: S) -> Bool = true\npub fn any<S>(self: S) -> Bool = true\n")
            self._write(root, "crates/cpl-ir/src/checks.rs", 'match name { "all" => Checks::All, _ => None }\n')
            self._write(root, "crates/cpl-ir/src/lower/call.rs", 'match name { "any" => {}, _ => {} }\n')
            subprocess.run(["git", "-C", str(root), "add", "-A"], check=True)
            self.assertEqual(dr.std_names_outside_rt(root), ["any"])

    def test_next_std_names_skips_a_variants_own_label(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "std/seq.cp", "pub fn check(self) -> Bool = true\npub fn link() = ()\n")
            self._write(root, "crates/cpl-next/src/lib.rs",
                        'Stage::Check => "check",\nStage::Own => "link",\nif name == "check" {}\n')
            self.assertEqual(dr.next_std_names(root),
                             ["crates/cpl-next/src/lib.rs:2: link", "crates/cpl-next/src/lib.rs:3: check"])

    def test_next_std_names_skips_a_querys_name(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "std/seq.cp", "pub fn parse(self) -> Int = 0\n")
            self._write(root, "crates/cpl-next/src/lib.rs",
                        '    const NAME: &\'static str = "parse";\nif n == "parse" {}\n')
            self.assertEqual(dr.next_std_names(root), ["crates/cpl-next/src/lib.rs:2: parse"])

    def test_next_std_names_counts_without_the_core_stand_in(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "std/seq.cp", "pub fn all<S>(self: S) -> Bool = true\n")
            self._write(root, "crates/cpl-next-check/src/expr.rs", 'if name == "all" {}\n')
            self.assertTrue(dr.next_std_names(root))

    def test_next_std_names_exempts_only_the_grammar_table(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "std/seq.cp", "pub fn max(a: Int) -> Int = a\n")
            self._write(root, dr.NEXT_GRAMMAR_TABLE, '("max", Contextual::Max),\n')
            self._write(root, "crates/cpl-next-syntax/src/parser.rs", 'if text == "max" {}\n')
            self.assertEqual(dr.next_std_names(root), ["crates/cpl-next-syntax/src/parser.rs:1: max"])

    def test_next_std_names_skips_only_the_ids_std_binds_in_an_id_table(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "std/raw.cp", 'pub fn raw_get() = @intrinsic("raw_get")\npub fn ctz() = 0\n'
                        '@fast(clz)\npub fn clz() = 0\n')
            self._write(root, "crates/cpl-next-ir/src/ops.rs",
                        'pub const INTRINSICS: &[(&str, I)] = &[\n    ("raw_get", I::Get),\n    ("ctz", I::Ctz),\n'
                        '    ("clz", I::Clz),\n];\nif n == "raw_get" {}\n')
            self.assertEqual(dr.next_std_names(root),
                             ["crates/cpl-next-ir/src/ops.rs:3: ctz", "crates/cpl-next-ir/src/ops.rs:6: raw_get"])

    def test_next_std_names_skips_only_the_check_names_the_spec_lists(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "std/ratio.cp", "pub fn div(a, b) = a\npub fn odd(a) = a\n")
            self._write(root, "spec/core/09-contexts-checks.md",
                        "Each row has one check name, used by `with checks(...)`: `bounds`, `div`.\n"
                        "This is the one list.\n")
            self._write(root, "crates/cpl-next-ir/src/build.rs",
                        'pub const ALL: [(&str, C); 2] = [\n    ("div", C::Div),\n    ("odd", C::Odd),\n];\n'
                        'if n == "div" {}\n')
            self.assertEqual(dr.next_std_names(root),
                             ["crates/cpl-next-ir/src/build.rs:3: odd", "crates/cpl-next-ir/src/build.rs:5: div"])

    def test_next_std_names_skips_test_code(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "std/seq.cp", "pub fn total(self) -> Int = 0\n")
            self._write(root, "crates/cpl-next-db/src/db.rs",
                        'if n == "total" {}\n#[cfg(test)]\nmod tests {\n    fn t() { q("total"); }\n}\n')
            self.assertEqual(dr.next_std_names(root), ["crates/cpl-next-db/src/db.rs:1: total"])

    def test_ir_builtin_names_reads_tuple_arms(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            call = root / "crates/cpl-ir/src/lower/call"
            call.mkdir(parents=True)
            (call.parent / "call.rs").write_text('    "sum" | "len" => {}\n')
            (call / "value_method.rs").write_text(
                '            (Ty::Range(inner), "nth" | "at") if args.len() == 1 => {\n'
                '            (ty, "find")\n'
                '                if ok =>\n'
                '        let x = ("not", "arm");\n')
            self.assertEqual(dr.ir_builtin_names(root), ["at", "find", "len", "nth", "sum"])
            self._write(root, "plan/domains/tech-debt/architecture/builtin-floor.md",
                        "| Storage | intrinsic | `len` `at` `nth` |\n| Derived | move | `sum` `find` |\n")
            self.assertEqual(dr.ir_builtin_derived(root), ["find", "sum"])

    def test_a_raise_the_old_crates_restore_brings_is_measured_away(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            git = lambda *a: subprocess.run(["git", *a], cwd=root, check=True, capture_output=True,
                                            env=dr._git_env())
            git("init", "-q")
            self._write(root, "std/subset.cp", "@member(subsets)\npub struct Subset<B>(bits: U64):\n")
            self._write(root, "crates/cpl-ir/src/a.rs", "fn a() {}\n")
            git("add", ".")
            git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "base")
            base = git("rev-parse", "HEAD").stdout.decode().strip()
            self.assertIsNone(dr.measured_without_restore(root, base, "domain_member_names"))
            self._write(root, "crates/cpl-ir/src/a.rs", 'fn a() {}\nif matches!(m, "subsets") {}\n')
            self._write(root, "crates/cpl-ir/src/b.rs", 'if matches!(m, "subsets") {}\n')
            git("add", ".")
            git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "restore")
            self.assertEqual(len(dr.domain_member_names(root)), 2)
            self.assertEqual(dr.measured_without_restore(root, base, "domain_member_names"), 0)

    def test_a_raised_baseline_fails_unless_its_measure_changed(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            git = lambda *a: subprocess.run(["git", *a], cwd=root, check=True, capture_output=True,
                                            env=dr._git_env())
            git("init", "-q")
            (root / "scripts").mkdir()
            script = Path(dr.__file__).read_text(encoding="utf-8")
            (root / "scripts/debt-ratchet.py").write_text(script)
            (root / "scripts/debt-ratchet.json").write_text(
                json.dumps({"lint_std_names": 5, "max_rs_lines": 2000, "rs_file_size_allowlist": {"a.rs": 2100}}))
            (root / "decisions.md").write_text("# Decisions\n")
            git("add", ".")
            git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "base")
            self.assertEqual(dr.unexplained_bumps(root, {"lint_std_names": 4}, ref="HEAD"), [])
            with open(root / "decisions.md", "a") as f:
                f.write("## 9. lint_std_names grows by one for X (2026-11-23)\n")
            for raised in ({"lint_std_names": 6}, {"max_rs_lines": 2400},
                           {"rs_file_size_allowlist": {"a.rs": 2101}}):
                self.assertEqual(len(dr.unexplained_bumps(root, raised, ref="HEAD")), 1, raised)
            fn = dr.COUNTERS["lint_std_names"][1].__name__
            at = script.index(f"def {fn}(")
            (root / "scripts/debt-ratchet.py").write_text(script[:at] + script[at:].replace("\n", "\n    # counts more\n", 1))
            self.assertEqual(dr.unexplained_bumps(root, {"lint_std_names": 6}, ref="HEAD"), [])
            self.assertEqual(len(dr.unexplained_bumps(root, {"max_rs_lines": 2400}, ref="HEAD")), 1)

    def test_a_copy_bound_rises_only_with_its_counter(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            git = lambda *a: subprocess.run(["git", *a], cwd=root, check=True, capture_output=True,
                                            env=dr._git_env())
            git("init", "-q")
            counts = root / "crates/cpl-codegen/src/cpp_counts.rs"
            counts.parent.mkdir(parents=True)
            counts.write_text("fn count() {}\n")
            (root / "scripts").mkdir()
            (root / "scripts/debt-ratchet.py").write_text(Path(dr.__file__).read_text(encoding="utf-8"))
            (root / "scripts/debt-ratchet.json").write_text(json.dumps({"cpp_std_functions": 70}))
            git("add", ".")
            git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "base")
            self.assertEqual(len(dr.unexplained_bumps(root, {"cpp_std_functions": 84}, ref="HEAD")), 1)
            counts.write_text("fn count() { /* a new pattern */ }\n")
            self.assertEqual(dr.unexplained_bumps(root, {"cpp_std_functions": 84}, ref="HEAD"), [])

    def test_near_dup_fns_pairs_near_copies_across_crates(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            body = "".join(f"    let x{i} = y.get({i}).unwrap_or(z{i}) + w{i};\n" for i in range(8))
            self._write(root, "crates/cpl-a/src/a.rs", "fn one() {\n" + body + "}\n")
            self._write(root, "crates/cpl-b/src/b.rs", "fn two() {\n" + body.replace("(3)", "(4)") + "}\n")
            self._write(root, "crates/cpl-b/src/c.rs", "fn same_crate() {\n" + body + "}\n")
            self._write(root, "crates/cpl-c/src/c.rs", "fn exact() {\n" + body + "}\n")
            other = "".join(f"    if q{i} > {i} {{ return; }}\n" for i in range(8))
            self._write(root, "crates/cpl-d/src/d.rs", "fn other() {\n" + other + "}\n")
            got = [x.split(" (")[0] for x in dr.near_dup_fns(root)]
            # Numbers fold to `0`, so `(3)` vs `(4)` is an exact token match but not an exact body.
            self.assertEqual(got, ["crates/cpl-b/src/b.rs:1 two ~ crates/cpl-a/src/a.rs:1 one",
                                   "crates/cpl-c/src/c.rs:1 exact ~ crates/cpl-b/src/b.rs:1 two"])

    def test_near_dup_fns_pairs_renamed_std_copies_across_modules(self) -> None:
        def rmq(fn: str, mask: str, at: str, pub: str = "") -> str:
            return (f"    {pub}fn {fn}(self, l: Int, r: Int) -> Int:\n"
                    f"        let base = l - l % 64\n"
                    f"        let {at} = {mask}[r] & ~((U64(1) << (l - base)) - U64(1))\n"
                    f"        base + bit_width({at} & (~{at} + U64(1))) - 1\n")
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            self._write(root, "std/sparse.cp", "struct Rmq:\n" + rmq("in_block", "mask", "m", "pub "))
            # The same body under other names, in another module: one pair.
            self._write(root, "std/strings.cp", "struct Sa:\n" + rmq("lowest_in", "bits", "at")
                        + rmq("again_here", "bits2", "w"))
            # Too short to be evidence, and a different shape of the same length.
            self._write(root, "std/tiny.cp", "fn one(x: Int) -> Int = x + 1\n"
                        "fn two(y: Int) -> Int:\n" + "".join(f"    if y > {i}: print(y * {i}, \"{i}\")\n"
                                                             for i in range(8)))
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            got = [x.split(" (")[0] for x in dr._cp_renamed_dups(root)]
            self.assertEqual(got, ["std/strings.cp:2 lowest_in ~ std/sparse.cp:2 in_block",
                                   "std/strings.cp:6 again_here ~ std/sparse.cp:2 in_block"])
            self.assertEqual(dr._cp_shape(["let a = b + 1"])[1], dr._cp_shape(["let x = y + 2"])[1])
            self.assertNotEqual(dr._cp_shape(["let a = b + 1"])[1], dr._cp_shape(["let a = b - 1"])[1])

    def test_tool_compiler_deps(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._write(root, "handbook/design/architecture.md", CRATE_RULES
                        + "| tool (facade only) | `cpl-lint`, `cpl-cli` |\n| compiler (behind it) | `cpl-check`, `cpl-ir` |\n")
            self._write(root, "crates/cpl-lint/Cargo.toml",
                        '[package]\nname = "cpl-lint"\n[dependencies]\ncpl-check = { path = "../cpl-check" }\n'
                        'cpl-syntax = { path = "../cpl-syntax" }\n[dev-dependencies]\ncpl-ir = { path = "../cpl-ir" }\n')
            self._write(root, "crates/cpl-conform/Cargo.toml",
                        '[package]\nname = "cpl-conform"\n[dependencies]\ncpl-ir = { path = "../cpl-ir" }\n')
            self._write(root, "crates/cpl-lint/src/a.rs",
                        "use cpl_check::Session;\n// cpl_ir::lower in a comment\nuse cpl_syntax::ast;\n"
                        "let p = cpl_ir::lower(&a);\n")
            self._write(root, "crates/cpl-cli/tests/t.rs", "let s = cpl_check::Session::new();\n")
            self._write(root, "crates/cpl-cli/src/abi.rs",
                        'pub extern "C" fn cpl_check() {}\nuse cpl_ir as ir;\nextern crate cpl_check;\n')
            self._write(root, "crates/cpl-conform/src/a.rs", "cpl_ir::lower(&a);\n")
            self.assertEqual(dr.tool_compiler_deps(root), [
                "cpl-lint -> cpl-check", "cpl-lint -> cpl-ir",
                "crates/cpl-lint/src/a.rs:1", "crates/cpl-lint/src/a.rs:4",
                "crates/cpl-cli/src/abi.rs:2", "crates/cpl-cli/src/abi.rs:3", "crates/cpl-cli/tests/t.rs:1"])

    def test_facade_today_uses(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._write(root, "handbook/design/architecture.md", CRATE_RULES
                        + "| tool (facade only) | `cpl-lint`, `cpl-cli` |\n| compiler (behind it) | `cpl-check` |\n")
            self._write(root, "crates/cpl-lint/src/a.rs",
                        "use cpl_facade::today::Analysis;\n// today::Ty in a comment\nlet today = 1;\n"
                        "use cpl_facade::{today, Session};\nlet p = today::program(&a, &job);\n"
                        "use cpl_facade::today;\n")
            self._write(root, "crates/cpl-cli/tests/t.rs", "let s: cpl_facade::today::Ty;\n")
            self._write(root, "crates/cpl-check/src/a.rs", "today::x();\n")
            self.assertEqual(dr.facade_today_uses(root), [
                "crates/cpl-lint/src/a.rs:1", "crates/cpl-lint/src/a.rs:5", "crates/cpl-lint/src/a.rs:6",
                "crates/cpl-cli/tests/t.rs:1"])
            # What the facade re-exports from a compiler crate at its root is today's form too.
            self._write(root, "crates/cpl-facade/src/run.rs",
                        "pub use cpl_check::trace::{Recording, Trap as Fault};\n// pub use cpl_check::Hidden;\n"
                        "pub type Checks = cpl_check::checks::Checks;\npub use cpl_diag::Diagnostic;\n"
                        "pub struct Outcome;\n")
            self._write(root, "crates/cpl-facade/src/today.rs", "pub use cpl_check::Ty;\n")
            self.assertEqual(dr.facade_reexports(root), {"Recording", "Fault", "Checks"})
            self._write(root, "crates/cpl-cli/src/b.rs",
                        "use cpl_facade::{\n    Outcome, Recording,\n};\nfn f(r: &Recording, o: Outcome) {}\n"
                        "// a Recording in a comment\nlet c: cpl_facade::Checks;\nstruct Fault;\nlet t: Ty;\n")
            self.assertEqual([u for u in dr.facade_today_uses(root) if "/b.rs:" in u], [
                "crates/cpl-cli/src/b.rs:2", "crates/cpl-cli/src/b.rs:4", "crates/cpl-cli/src/b.rs:6"])

    def test_consumer_rederived(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self._write(root, "handbook/design/architecture.md", CRATE_RULES)
            self._write(root, "std/heap.cp", "pub struct Heap<T> {}\n")
            self._write(root, "crates/cpl-lsp/src/a.rs",
                        'let db = Database::new();\n// Database::new()\nif t == "Heap" || t == "Foo" {}\n'
                        'match n { "Int" | "Str" => 1, "ok" => 2 }\nlet c = "E0302";\nprint("List");\n'
                        '#[cfg(test)]\nmod tests { let db = Database::new(); }\n')
            self._write(root, "crates/cpl-lsp/src/b_tests.rs", "let db = Database::new();\n")
            self._write(root, "crates/cpl-check/src/c.rs", 'let db = Database::new(); t == "Int";\n')
            self.assertEqual(dr.consumer_rederived(root), [
                "crates/cpl-lsp/src/a.rs:3 type Heap",
                "crates/cpl-lsp/src/a.rs:4 type Int", "crates/cpl-lsp/src/a.rs:4 type Str",
                "crates/cpl-lsp/src/a.rs:5 code E0302"])

    def test_a_documented_new_crate_edge_is_offered_for_tightening(self) -> None:
        baseline = {"crate_edges": ["a -> b"]}
        documented = {"crate_edges": ["a -> b", "c -> a"], "a4_edges": ["a -> b", "c -> a"]}
        self.assertEqual(dr.evaluate(documented, baseline)[0], [])
        self.assertIn("crate_edges", dr.improved(documented, baseline))
        undocumented = {"crate_edges": ["a -> b", "c -> a"], "a4_edges": ["a -> b"]}
        self.assertNotIn("crate_edges", dr.improved(undocumented, baseline))

    def test_a_file_entering_the_near_cap_band_fails(self) -> None:
        baseline = {"rs_near_cap": ["a.rs"], "rs_file_size_allowlist": {"big.rs": 2400}}
        actual = {"rs_near_cap": ["big.rs", "new.rs"], "rs_file_size_allowlist": {}}
        errs, _, notes = dr.evaluate(actual, baseline)
        self.assertEqual(len(errs), 1)
        self.assertIn("new.rs", errs[0])
        self.assertIn("a.rs: left the near-cap band", notes)
        self.assertIn("rs_near_cap", dr.improved(actual, baseline))
        eff = dr.inherited_baseline(baseline, {"rs_near_cap": ["new.rs"]})
        self.assertEqual(dr.evaluate(actual, eff)[0], [])

    def test_not_passing_may_rise_by_the_cases_a_change_adds(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            git = lambda *a: subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *a], cwd=root,
                                            check=True, env=dr._git_env(), capture_output=True)
            git("init", "-q")
            (root / "scripts").mkdir()
            (root / "scripts" / "debt-ratchet.json").write_text('{"next_not_passing": 10}')
            git("add", "-A")
            git("commit", "-qm", "base")
            git("branch", "base")
            case = root / "spec" / "conformance" / "core-01" / "a.cp"
            case.parent.mkdir(parents=True)
            case.write_text("// expect: ok\n")
            git("add", "-A")
            self.assertEqual(dr.unexplained_bumps(root, {"next_not_passing": 11}, ref="base"), [])
            self.assertEqual(len(dr.unexplained_bumps(root, {"next_not_passing": 12}, ref="base")), 1)

    def test_whole_program_inputs_count_layer_fns_taking_the_whole_program(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            src = root / "crates/cpl-next-check/src"
            src.mkdir(parents=True)
            (src / "lib.rs").write_text(
                "pub fn check(d: &Decls) -> Typed {}\n"
                "pub fn body(\n    sigs: &dyn Signatures,\n    t: &check::Typed,\n) -> X {}\n"
                "pub fn item(sigs: &dyn Signatures) -> Typed {}\n"
                "fn private(d: &Decls) {}\n"
                'const RT: &str = r#"int a;\n\nint b;"#;\n'
                "struct Cx<'a> {\n    d: &'a Decls,\n}\n"
                "// a comment naming &Whole\n"
                "let f = |g: &mut cpl_next_lower::Generic| ();\n"
            )
            self.assertEqual(dr.next_whole_program_inputs(root),
                             [f"crates/cpl-next-check/src/lib.rs:{i}" for i in (1, 4, 7, 12, 15)])
            self.assertEqual(len(dr.next_cpp_runtime_lines(root)), 2)

    def test_whole_program_inputs_count_holders_their_fields_and_their_methods(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            src = root / "crates/cpl-next-check/src"
            src.mkdir(parents=True)
            (src / "lib.rs").write_text(
                "struct Front {\n    d: Decls,\n    items: Vec<Item>,\n}\n"
                "struct Cx<'a> {\n    f: &'a Front,\n}\n"
                "impl<'a> Cx<'a> {\n    fn a(&self) {}\n    fn new(f: &'a Front) -> Self {}\n"
                "    fn b(\n        &mut self,\n    ) {}\n}\n"
                "impl<'a> Signatures for Cx<'a> {\n    fn c(&self) {}\n}\n"
                "fn walk(cx: &mut Cx<'_>) {}\n"
                "impl Decls {\n    fn sig(&self) {}\n}\n"
                "struct Tidy {\n    ty: TypedFn,\n}\n"
                "fn make(n: usize) -> Front {}\n"
                "use cpl_next_decls::decls::Decls;\n"
                "struct Pair(Generic);\n"
                "struct Sig {\n    ret: ast::Front,\n}\n"
                "impl Front {\n    fn with_checks(&self, c: Checks) -> Front {}\n    fn cx(&self) -> Cx<'_> {}\n"
                "    fn take(&mut self, d: Decls) {}\n}\n"
            )
            self.assertEqual(dr.next_whole_program_inputs(root),
                             [f"crates/cpl-next-check/src/lib.rs:{i}" for i in (2, 6, 9, 10, 11, 16, 18, 27, 33, 34)])

    def test_whole_program_inputs_count_a_bare_holder_name_only_where_it_resolves(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            files = {
                "cpl-next/src/lib.rs": "pub struct Type<'s> {\n    d: Decls,\n}\npub use cpl_next_decls::modules::Std;\n",
                "cpl-next-decls/src/modules.rs": "pub struct Std {\n    d: Decls,\n}\n",
                # Imported through a multi-line `use crate::{..}` of this crate's re-export.
                "cpl-next/src/facade.rs": "use crate::{\n    Std,\n};\nstruct Held {\n    s: Std,\n}\n",
                # Another crate's `Type`.
                "cpl-next-decls/src/cycles.rs": "use cpl_next_syntax::ast::{GenericArg, Type};\nfn parts(t: &Type) {}\n",
                "cpl-next-check/src/a.rs": "use cpl_next::{\n    Type,\n};\nfn f(t: &Type) {}\n",
                "cpl-next-link/src/lib.rs": "struct Env<'e> {\n    d: Decls,\n}\nfn k(env: &Env) {}\n",
                # A fn-local struct of the same name.
                "cpl-next-decls/src/decls.rs": "fn g() {\n    struct Env {\n        n: usize,\n    }\n    fn h(env: &Env) {}\n}\n",
            }
            for rel, text in files.items():
                (root / "crates" / rel).parent.mkdir(parents=True, exist_ok=True)
                (root / "crates" / rel).write_text(text)
            self.assertEqual(dr.next_whole_program_inputs(root), sorted([
                "crates/cpl-next/src/lib.rs:2", "crates/cpl-next-decls/src/modules.rs:2",
                "crates/cpl-next/src/facade.rs:5", "crates/cpl-next-check/src/a.rs:4",
                "crates/cpl-next-link/src/lib.rs:2", "crates/cpl-next-link/src/lib.rs:4"]))

    def test_debt_already_on_main_fails_a_push_that_does_not_lower_it(self) -> None:
        label = lambda n: f"{dr.COUNTERS[n][0]}: increased"
        hard = sorted(dr.HARD_ZERO)[0]
        soft = "next_std_names"
        case = "next_wrong_accepts"
        errors = [label(hard), label(soft), label(case)]
        self.assertEqual(dr.waived(errors, [], {hard: 1, soft: 54, case: 29}, {hard: 1, soft: 54, case: 29}),
                         [label(hard), label(soft)])
        self.assertEqual(dr.waived(errors, [], {hard: 1, soft: 53, case: 29}, {hard: 1, soft: 54, case: 29}),
                         [label(hard)])

    def test_whole_program_inputs_count_dyn_views_failing_lane_is_key_rule(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            src = root / "crates/cpl-next-decls/src"
            src.mkdir(parents=True)
            (src / "lib.rs").write_text(
                "pub trait Keyed {\n    fn f(&self, id: FnId) -> &FnDecl;\n    fn s(&self, s: Sym, i: usize) -> Vec<FnId>;\n}\n"
                "pub trait Texty: Keyed {\n    fn named(&self, m: ModId, name: &str) -> Option<Named>;\n}\n"
                "pub trait Keyless {\n    fn widest(&self) -> usize;\n}\n"
                "pub trait Inherits: Keyless {\n    fn g(&self, g: GlobalId) -> &Ty;\n}\n"
                "pub trait Every {\n    fn all_fns(&self, m: ModId) -> Vec<FnId>;\n}\n"
                "struct Cx<'a> {\n    k: &'a dyn Keyed,\n    t: &'a dyn Texty,\n}\n"
                "fn a(k: &dyn Keyed, i: &dyn Inherits, e: &dyn Every, b: Box<dyn Keyless>) {}\n"
                "fn b(s: &impl Texty) {}\n"
            )
            self.assertEqual(dr.next_whole_program_inputs(root),
                             [f"crates/cpl-next-decls/src/lib.rs:{i}" for i in (19, 21)])

    def test_decls_const_op_arms_count_operator_arms_not_literal_kinds(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            src = root / "crates/cpl-next-decls/src/decls"
            src.mkdir(parents=True)
            (src / "tyconv.rs").write_text(
                "match t.kind() {\n    TokenKind::Int => i64::try_from(v).ok(),\n    _ => None,\n}\n"
                "Expr::Prefix(p) if p.op()?.kind() == TokenKind::Minus => neg(),\n"
                "match b.op()?.kind() {\n    TokenKind::Plus => x.checked_add(y),\n"
                "    TokenKind::StarStar => x.checked_pow(y),\n}\n"
                "let m = k == TokenKind::Minus;\n")
            self.assertEqual(dr.decls_const_op_arms(root), [
                "crates/cpl-next-decls/src/decls/tyconv.rs:5", "crates/cpl-next-decls/src/decls/tyconv.rs:7",
                "crates/cpl-next-decls/src/decls/tyconv.rs:8"])
            self.assertIn("decls_const_op_arms", dr.HARD_ZERO)

    def test_synthetic_name_tests_count_only_dollar_prefix_tests_in_the_checker(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            src = root / "crates/cpl-next-check/src"
            src.mkdir(parents=True)
            (src / "place.rs").write_text(
                "let a = name.starts_with('$');\nlet b = !n.starts_with(\"$\");\n"
                "let c = name.starts_with('_');\nlet d = local.synthetic;\n")
            other = root / "crates/cpl-next-ir/src"
            other.mkdir(parents=True)
            (other / "lib.rs").write_text("let a = name.starts_with('$');\n")
            self.assertEqual(dr.next_synthetic_name_tests(root),
                             ["crates/cpl-next-check/src/place.rs:1", "crates/cpl-next-check/src/place.rs:2"])

    def test_a_view_keyed_by_a_concrete_type_value_is_a_view(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            src = root / "crates/cpl-next-decls/src"
            src.mkdir(parents=True)
            (src / "lib.rs").write_text(
                "pub trait ImplChoice {\n    fn impl_for(&self, tr: TraitId, t: &Ty) -> Option<Chosen>;\n"
                "    fn impls_on(&self, t: &ir::Ty) -> Vec<(TraitId, Chosen)>;\n}\n"
                "pub trait Tabled {\n    fn on(&self, t: &TyTable) -> Option<Chosen>;\n    fn of(&self, ts: &[Ty]) -> usize;\n}\n"
                "fn a(c: &dyn ImplChoice, t: &dyn Tabled) {}\n"
            )
            self.assertEqual(dr.next_whole_program_inputs(root), ["crates/cpl-next-decls/src/lib.rs:9"])

    def test_audit_counters_find_soundness_core_checks_ownership_and_foreign_deps(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            tests = root / dr.NEXT_TESTS
            tests.mkdir(parents=True)
            (tests / "wrong.tsv").write_text("a.cp\twrong-accept\twanted E0410\nb.cp\twrong-reject\tE0201\n")
            self.assertEqual(dr.next_wrong_accepts(root), ["a.cp"])
            own = root / "crates/cpl-next-own/src"
            own.mkdir(parents=True)
            (own / "lib.rs").write_text("pub fn lends(f: Fn) {}\n")
            ir = root / "crates/cpl-next-ir/src"
            ir.mkdir(parents=True)
            (ir / "lib.rs").write_text("pub fn lends(self) {}\n")
            eng = root / "crates/cpl-next-engine/src"
            eng.mkdir(parents=True)
            (eng / "lib.rs").write_text("let s = Stmt::Drop(v);\nStmt::Drop(x) => {}\nf.drops.push(v);\n"
                                        "if m == CORE {}\n// m != CORE\nif CORE != m {}\n")
            self.assertEqual(dr.next_ownership_outside_own(root),
                             ["crates/cpl-next-engine/src/lib.rs:1 Stmt::Drop", "crates/cpl-next-engine/src/lib.rs:3 Func field"])
            (root / "std").mkdir(exist_ok=True)
            (root / "std/bits.cp").write_text(
                "impl Word for Int:\n    fn word(self) -> U64 = U64(0)\n"
                "@fast(popcount)\n/// doc\npub fn ones<W: Word>(x: W) -> Int = 0\n"
                "@fast(ctz)\npub fn low(x: U64) -> Int = 0\n"
                "@fast(clz)\npub fn two(x: Int, y: Int) -> Int = 0\n"
                "@fast(bit_width)\npub fn wide(x: F64) -> Int = 0\n")
            self.assertEqual(dr.fast_unchecked(root), ["std/bits.cp:8 two", "std/bits.cp:10 wide"])
            (root / "std/bits.cp").write_text("impl Word for Bits:\n@fast(popcount)\npub fn ones<W: Word>(x: W) -> Int = 0\n")
            self.assertEqual(dr.fast_unchecked(root), ["std/bits.cp:2 ones"])
            self.assertEqual(dr.next_core_checks(root),
                             ["crates/cpl-next-engine/src/lib.rs:4", "crates/cpl-next-engine/src/lib.rs:6"])
            (root / "crates/cpl-next-engine/src/set.rs").write_text(
                'match name {\n    Some("ranges" | "div") if on => {}\n    Some("x") => go(),\n}\n')
            self.assertEqual(dr.next_ignored_settings(root),
                             ["crates/cpl-next-engine/src/set.rs:2 ranges", "crates/cpl-next-engine/src/set.rs:2 div"])
            (root / "crates/cpl-next-engine/src/set.rs").unlink()
            (root / "crates/cpl-next-engine/src/set.rs").write_text(
                "impl check::Consts for E {}\nlet v = cpl_next_engine::eval(p, f, g, 9);\n")
            self.assertEqual(dr.next_hidden_const_evaluators(root), [])
            self._write(root, "crates/cpl-next-lower/src/c.rs",
                        "impl<'a> check::Consts for E<'a> {}\nlet v = cpl_next_engine::eval(p, f, g, 9);\n// Consts for\n")
            self.assertEqual(dr.next_hidden_const_evaluators(root),
                             ["crates/cpl-next-lower/src/c.rs:1", "crates/cpl-next-lower/src/c.rs:2"])
            (root / "crates/cpl-next-lower/src/c.rs").unlink()
            (root / "crates/cpl-next-check").mkdir(parents=True)
            (root / "crates/cpl-next-check/Cargo.toml").write_text(
                '[dependencies]\ncpl-syntax.workspace = true\ncpl-judge.workspace = true\ncpl-next-ir.workspace = true\n')
            self.assertEqual(dr.next_layer_foreign_deps(root), ["cpl-next-check -> cpl-judge"])
            self._write(root, "crates/cpl-next/Cargo.toml",
                        "[dependencies]\ncpl-env.workspace = true\ncpl-next-check.workspace = true\n")
            self.assertEqual(dr.next_layer_foreign_deps(root),
                             ["cpl-next -> cpl-env", "cpl-next-check -> cpl-judge"])
            (root / "crates/cpl-next-check/src").mkdir()
            (root / "crates/cpl-next-check/src/lib.rs").write_text(
                "use cpl_syntax::ast::Expr;\nuse cpl_next_syntax::ast;\nfn f() { cpl_syntax::parse_file(\"\"); }\n")
            self.assertEqual(dr.next_syntax_old_refs(root), [
                "crates/cpl-next-check/Cargo.toml: dependencies cpl-syntax",
                "crates/cpl-next-check/src/lib.rs:1:5 cpl_syntax", "crates/cpl-next-check/src/lib.rs:3:10 cpl_syntax"])
            # A grouped import counts each use behind it; an alias counts as its name; a comment does not.
            (root / "crates/cpl-next-check/src/lib.rs").write_text(
                "use cpl_syntax::{\n    ast::{self, StmtKind as SK},\n    decorators::Mark,\n};\n"
                "fn f(s: &ast::Stmt) -> bool {\n    // ast::Expr\n    matches!(s.kind, SK::Pass) && Mark::of(s)\n}\n")
            self.assertEqual(dr.next_syntax_old_refs(root), [
                "crates/cpl-next-check/Cargo.toml: dependencies cpl-syntax", "crates/cpl-next-check/src/lib.rs:1:5 cpl_syntax",
                "crates/cpl-next-check/src/lib.rs:5:10 ast", "crates/cpl-next-check/src/lib.rs:7:22 SK",
                "crates/cpl-next-check/src/lib.rs:7:35 Mark"])
            self.assertEqual(dr.next_old_syntax_analyses(root), ["crates/cpl-next-check/src/lib.rs:7:35 Mark"])

    def test_next_run_api_unbuilt(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "handbook/design/next/contracts/00-snapshot.md",
                        "```rust\nstruct RunInput { entry: Entry, limits: Limits /* a, b */ }\n"
                        "enum Entry { Program, // file\n Call { f: DeclId, args: List<Value> } }\n```\n")
            self._write(root, "crates/cpl-next/src/run.rs",
                        "pub struct RunInput {\n    pub limits: Limits,\n}\npub enum Entry {\n    Program,\n}\n")
            self.assertEqual(dr.next_run_api_unbuilt(root),
                             ["00-snapshot.md Entry.Call", "00-snapshot.md RunInput.entry"])

    def test_next_facade_split(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "crates/cpl-facade/src/session.rs",
                        "impl Session {\n    pub fn a(&self) -> u32 {\n    pub fn b(&self) {\n"
                        "    pub fn d(&self) -> u8 {\n}\n")
            # `c` is cpl-next's alone, with no old-side half: the one facade's own, not a split.
            self._write(root, "crates/cpl-next/src/facade.rs",
                        "impl Session {\n    pub fn a(&self) -> u32 {\n    pub fn c(\n        &self,\n"
                        "    ) -> u8 {\n    pub fn d(&self) -> u16 {\n}\npub struct Snapshot(u8);\n")
            self.assertEqual(dr.next_facade_split(root), [
                "cpl-facade Session::pub fn b(&self)", "cpl-facade Session::pub fn d(&self) -> u8",
                "cpl-next Session::pub fn d(&self) -> u16", "crates/cpl-next/src/facade.rs:8"])

    def test_lent_outside_own_and_asks_refused(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for c, body in [("cpl-next-lower", "let t = Ty::Lent(Box::new(t));\n// Ty::Lent(Box::new(x))\n"
                                               "(_, Ty::Lent(s)) => Ty::Lent(Box::new(f(s))),\nTy::Lent(t) => t,\n"),
                            ("cpl-next-own", "Ty::Lent(Box::new(t))\n"), ("cpl-next-check", "Ty::Lent(Box::new(t))\n")]:
                self._write(root, f"crates/{c}/src/lib.rs", body)
            # Lowering runs before own marks anything, so a lent type it matches counts as well as one it makes.
            self.assertEqual(dr.next_lent_outside_own(root), ["crates/cpl-next-lower/src/lib.rs:1",
                                                              "crates/cpl-next-lower/src/lib.rs:3",
                                                              "crates/cpl-next-lower/src/lib.rs:4"])
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            self._write(root, "spec/conformance/a/slow.cp", "// expect: ok\n// timeout: 20\nfn main() {}\n")
            self._write(root, "spec/conformance/a/fine.cp", "// expect: ok\nfn main() {}\n// judge: no\n")
            self._write(root, "spec/conformance/a/copy.cp", "// expect: ok\n// copies: 1\nfn main() {}\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.next_asks_refused(root), ["spec/conformance/a/copy.cp"])

    def test_backend_owns_reads_count_every_engine_file(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "crates/cpl-next-engine/src/compile.rs",
                        "fn owns(&mut self, t: &CTy) -> bool {\nif self.owns(&e.ty) {}\n// lays.owns[l]\nc.lays.owns[l as usize]\n")
            self._write(root, "crates/cpl-next-engine/src/exec.rs", "if !c.lays.owns[lay as usize] {}\n")
            self._write(root, "crates/cpl-next-cpp/src/emit.rs", "let o = owns(ty);\n")
            self.assertEqual(dr.next_backend_owns_reads(root), [
                "crates/cpl-next-engine/src/compile.rs:2", "crates/cpl-next-engine/src/compile.rs:4",
                "crates/cpl-next-engine/src/exec.rs:1", "crates/cpl-next-cpp/src/emit.rs:1"])

    def test_name_keys_and_structural_link_edge(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "crates/cpl-next-check/src/domain.rs",
                        'let a = self.bind(&format!("{name}$lo"), Ty::Int);\n'
                        'let b = format!("{b}.{}", f);\nlet t = format!("$tuple{}", s);\n'
                        'Ty::Range(lo, hi) => format!("{lo}..={hi}"),\nlet k = format!("{b}.{field}");\n'
                        'let p = format!("{}${}", d, e);\nlet q = format!("{0}.{1}", d, e);\n'
                        'let r = format!("{}..{}", lo, hi);\n')
            self.assertEqual(dr.next_check_name_keys(root), ["crates/cpl-next-check/src/domain.rs:1",
                                                             "crates/cpl-next-check/src/domain.rs:2",
                                                             "crates/cpl-next-check/src/domain.rs:5",
                                                             "crates/cpl-next-check/src/domain.rs:6",
                                                             "crates/cpl-next-check/src/domain.rs:7"])
            self._write(root, "crates/cpl-next-link/Cargo.toml", "[dependencies]\ncpl-next-check.workspace = true\n")
            self._write(root, "crates/cpl-next-lower/Cargo.toml", "[dependencies]\ncpl-next-check.workspace = true\n")
            self.assertEqual(dr.link_reads_check(root), ["crates/cpl-next-link/Cargo.toml: reaches the checker "
                                                         "via cpl-next-link -> cpl-next-check"])

    def test_weak_oracles_and_syntax_std_names(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "crates/cpl-next/tests/passing.txt", "a/x.cp\na/y.cp\na/z.cp\nnext/n.cp\n")
            self._write(root, "spec/conformance/a/x.cp", "// expect: exit 1\nfn main() {}\n")
            self._write(root, "spec/conformance/a/y.cp", "// expect: ok\n// output: 1\nprint(1)\n")
            self._write(root, "spec/conformance/a/z.cp", "// expect: ok\nprint(1)\n// output: 1\n")
            self._write(root, "crates/cpl-next/tests/programs/n.cp", "// expect: E0606\nlet x = 1\n")
            self.assertEqual(dr.next_weak_oracle_passes(root), ["a/x.cp", "a/z.cp"])
            self._write(root, "std/cut.cp", "pub struct Cut<D>:\n    lo: D\n")
            self._write(root, "crates/cpl-syntax/src/parser/types.rs", 'if name.text == "Bounds" { name.text = "Cut".into(); }\n')
            self._write(root, "crates/cpl-syntax/src/recovery_tests.rs", 'let t = "Cut";\n')
            self.assertEqual(dr.syntax_std_names(root), ["crates/cpl-syntax/src/parser/types.rs:1: Cut"])
            self._write(root, "crates/cpl-next-lower/src/lib.rs", "(v, p, Ty::Ref(Box::new(ty)), true)\nTy::Ref(e) => Ty::Ref(Box::new(f(e))),\n")
            self.assertEqual(dr.next_lent_outside_own(root), ["crates/cpl-next-lower/src/lib.rs:1",
                                                              "crates/cpl-next-lower/src/lib.rs:2"])

    def test_wrong_rises_by_a_hidden_case_only_when_named(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            git = lambda *a: subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *a], cwd=root,
                                            check=True, env=dr._git_env(), capture_output=True)
            git("init", "-q")
            (root / "scripts").mkdir()
            (root / "scripts" / "debt-ratchet.json").write_text('{"next_wrong": 10}')
            for c in ["hid", "passed", "wrong"]:
                case = root / "spec" / "conformance" / "core-01" / f"{c}.cp"
                case.parent.mkdir(parents=True, exist_ok=True)
                case.write_text("// expect: ok\n")
            tests = root / dr.NEXT_TESTS
            tests.mkdir(parents=True)
            (tests / "passing.txt").write_text("core-01/passed.cp\n")
            (tests / "wrong.tsv").write_text("core-01/wrong.cp\twrong-reject\n")
            git("add", "-A")
            git("commit", "-qm", "base")
            git("branch", "base")
            rows = ["core-01/wrong.cp", "core-01/hid.cp", "core-01/passed.cp", "core-01/new.cp"]
            (tests / "wrong.tsv").write_text("".join(f"{r}\twrong-reject\n" for r in rows))
            self.assertEqual(dr.unhidden_wrong(root, "base"), ["core-01/hid.cp"])
            # An exposure is capped at 0: it rises only as far as `Regressed:` names cases.
            self.assertEqual(len(dr.unexplained_bumps(root, {"next_wrong": 11}, ref="base")), 1)
            base = subprocess.run(["git", "rev-parse", "base"], cwd=root, env=dr._git_env(),
                                  capture_output=True, text=True).stdout.strip()
            git("commit", "-qam", f"factory(next): lists\n\nRegressed: core-01/hid.cp {base}\nLane: G")
            self.assertEqual(dr.unexplained_bumps(root, {"next_wrong": 11}, ref="base"), [])
            self.assertEqual(len(dr.unexplained_bumps(root, {"next_wrong": 12}, ref="base")), 1)

    def test_a_split_main_refused_enters_the_split_set_only_named(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            git = lambda *a: subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *a], cwd=root,
                                            check=True, env=dr._git_env(), capture_output=True)
            git("init", "-q")
            (root / "scripts").mkdir()
            (root / "scripts" / "debt-ratchet.json").write_text(
                '{"next_split": 1, "next_split_ids": ["core-01/wrong.cp"]}')
            for c in ["hid", "passed", "wrong"]:
                case = root / "spec" / "conformance" / "core-01" / f"{c}.cp"
                case.parent.mkdir(parents=True, exist_ok=True)
                case.write_text("// expect: ok\n")
            tests = root / dr.NEXT_TESTS
            tests.mkdir(parents=True)
            (tests / "passing.txt").write_text("core-01/passed.cp\n")
            (tests / "wrong.tsv").write_text("core-01/wrong.cp\tsplit\n")
            git("add", "-A")
            git("commit", "-qm", "base")
            git("branch", "base")
            (tests / "wrong.tsv").write_text("".join(f"core-01/{c}.cp\tsplit\n" for c in ["hid", "wrong"]))
            hid = {"next_split": 2, "next_split_ids": ["core-01/hid.cp", "core-01/wrong.cp"]}
            self.assertEqual(len(dr.unexplained_bumps(root, hid, ref="base")), 2)
            base = subprocess.run(["git", "rev-parse", "base"], cwd=root, env=dr._git_env(),
                                  capture_output=True, text=True).stdout.strip()
            git("commit", "-qam", f"factory(next): lists\n\nRegressed: core-01/hid.cp {base}\nLane: G")
            self.assertEqual(dr.unexplained_bumps(root, hid, ref="base"), [])
            (tests / "passing.txt").write_text("")
            (tests / "wrong.tsv").write_text("".join(f"core-01/{c}.cp\tsplit\n" for c in ["passed", "wrong"]))
            passed = {"next_split": 2, "next_split_ids": ["core-01/passed.cp", "core-01/wrong.cp"]}
            raised = [b for b in dr.unexplained_bumps(root, passed, ref="base") if "baseline raised" in b]
            self.assertEqual([b.partition(":")[0] for b in raised], ["next_split", "next_split_ids"])

    def test_a_set_counter_fails_on_a_swap_its_total_hides(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            git = lambda *a: subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *a], cwd=root,
                                            check=True, env=dr._git_env(), capture_output=True)
            git("init", "-q")
            (root / "scripts").mkdir()
            (root / "scripts" / "debt-ratchet.json").write_text('{"next_wrong": 2}')
            tests = root / dr.NEXT_TESTS
            tests.mkdir(parents=True)
            (tests / "passing.txt").write_text("core-01/eq.cp\ncore-01/ok.cp\n")
            (tests / "wrong.tsv").write_text("core-01/tags.cp\twrong-accept\ncore-01/rej.cp\twrong-reject\n"
                                             "core-01/acc.cp\twrong-reject\n")
            (tests / "masked.txt").write_text("")
            git("add", "-A")
            git("commit", "-qm", "base")
            git("branch", "base")
            self.assertEqual(dr.set_swaps(root, "base"), [])
            (tests / "passing.txt").write_text("core-01/tags.cp\n")
            (tests / "wrong.tsv").write_text("core-01/eq.cp\twrong-accept\ncore-01/new.cp\twrong-reject\n"
                                             "core-01/acc.cp\twrong-accept\n")
            (tests / "masked.txt").write_text("core-01/ok.cp\ncore-01/rej.cp\n")
            self.assertEqual([s.split(";")[0] for s in dr.set_swaps(root, "base")], [
                "core-01/acc.cp: a wrong-accept base does not have",
                "core-01/eq.cp: passes on base, wrong here (wrong-accept)",
                "core-01/ok.cp: passes on base, masked here",
                "core-01/rej.cp: wrong on base, masked here",
            ])
            self.assertEqual(len(dr.unexplained_bumps(root, {"next_wrong": 2}, ref="base")), 4)
            cause = git("rev-parse", "base").stdout.decode().strip()
            (root / "other.md").write_text("x\n")
            git("add", "-A")
            git("commit", "-qm", "factory: lists and more", "-m", f"Regressed: core-01/ok {cause}")
            self.assertEqual(len(dr.set_swaps(root, "base")), 4)
            (tests / "masked.txt").write_text("core-01/rej.cp\n")
            (root / "plan").mkdir()
            (root / "plan/END-STATE.md").write_text("written from the lists\n")
            git("add", "-A")
            git("commit", "-qm", "factory: regenerate", "-m",
                f"Regressed: core-01/eq.cp {cause}\nRegressed: core-01/acc.cp {cause}\nRegressed: core-01/rej.cp 0123456789ab")
            self.assertEqual([s.split(":")[0] for s in dr.set_swaps(root, "base")], ["core-01/rej.cp"])
            self.assertEqual(dr.regressed_named(root, "base"), {"core-01/eq.cp", "core-01/acc.cp"})
            (root / "spec/conformance/core-01").mkdir(parents=True)
            (root / "spec/conformance/core-01/ok.cp").write_text("print(1)\n")
            (tests / "passing.txt").write_text("")
            self.assertIn("core-01/ok.cp: passes on base, refused here",
                          [s.split(";")[0] for s in dr.set_swaps(root, "base")])
            git("add", str(tests / "passing.txt"))
            with mock.patch.dict(os.environ, {"CPL_REGRESSED": f"core-01/ok.cp {cause};core-01/rej.cp {cause}"}):
                self.assertEqual(dr.set_swaps(root, "base"), [])
            git("add", "-A")
            with mock.patch.dict(os.environ, {"CPL_REGRESSED": f"core-01/ok.cp {cause}"}):
                self.assertEqual(len(dr.set_swaps(root, "base")), 2)

    def test_a_pass_a_fresh_run_lost_fails_though_the_lists_hold_it(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            git = lambda *a: subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *a], cwd=root,
                                            check=True, env=dr._git_env(), capture_output=True)
            git("init", "-q")
            tests = root / dr.NEXT_TESTS
            tests.mkdir(parents=True)
            (tests / "passing.txt").write_text("core-01/ok.cp\n")
            (tests / "wrong.tsv").write_text("")
            git("add", "-A")
            git("commit", "-qm", "base")
            git("branch", "base")
            run = root / "target/cpl-next"
            run.mkdir(parents=True)
            (run / "passing.txt").write_text("")
            (run / "wrong.tsv").write_text("core-01/ok.cp\twrong-output\n")
            self.assertEqual([s.split(";")[0] for s in dr.set_swaps(root, "base")],
                             ["core-01/ok.cp: passes on base, wrong here (wrong-output)"])
            os.utime(run / "passing.txt", (0, 0))
            os.utime(run / "wrong.tsv", (0, 0))
            self.assertEqual(dr.set_swaps(root, "base"), [])

    def test_a_wrong_case_may_not_become_refused_unless_decisions_names_it(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            git = lambda *a: subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *a], cwd=root,
                                            check=True, env=dr._git_env(), capture_output=True)
            git("init", "-q")
            (root / "scripts").mkdir()
            (root / "scripts" / "debt-ratchet.json").write_text('{"next_wrong": 2}')
            for c in ["fixed", "hidden", "kept"]:
                case = root / "spec" / "conformance" / "core-01" / f"{c}.cp"
                case.parent.mkdir(parents=True, exist_ok=True)
                case.write_text("// expect: ok\n")
            tests = root / dr.NEXT_TESTS
            tests.mkdir(parents=True)
            (tests / "passing.txt").write_text("")
            (tests / "wrong.tsv").write_text("".join(f"core-01/{c}.cp\twrong-reject\n" for c in ["fixed", "hidden", "kept"]))
            git("add", "-A")
            git("commit", "-qm", "base")
            git("branch", "base")
            (tests / "passing.txt").write_text("core-01/fixed.cp\n")
            (tests / "wrong.tsv").write_text("core-01/kept.cp\twrong-reject\n")
            self.assertEqual(dr.wrong_to_refused(root, "base"), ["core-01/hidden.cp"])
            self.assertEqual(len(dr.unexplained_bumps(root, {"next_wrong": 1}, ref="base")), 1)
            (root / "decisions.md").write_text("Notes: `core-01/hidden.cp`.\n")
            self.assertEqual(len(dr.unexplained_bumps(root, {"next_wrong": 1}, ref="base")), 1)
            (root / "decisions.md").write_text("## 7. Cut a feature\n\n`core-01/hidden.cp` refuses: its feature is cut.\n")
            self.assertEqual(dr.unexplained_bumps(root, {"next_wrong": 1}, ref="base"), [])

    def test_a_deleted_case_counts_until_removed_md_names_it(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            git = lambda *a: subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *a], cwd=root,
                                            check=True, env=dr._git_env(), capture_output=True)
            git("init", "-q")
            case = root / "spec" / "conformance" / "core-01" / "a.cp"
            case.parent.mkdir(parents=True)
            case.write_text("// expect: ok\n")
            git("add", "-A")
            git("commit", "-qm", "base")
            git("branch", "base")
            case.unlink()
            self.assertEqual(dr.next_cases_removed(root, "base"), ["spec/conformance/core-01/a.cp"])
            (root / "spec" / "conformance" / "REMOVED.md").write_text(
                "- `spec/conformance/core-01/a.cp`: the rule it tested is gone\n")
            self.assertEqual(dr.next_cases_removed(root, "base"), [])

    def test_a_case_the_ref_gained_after_the_fork_is_not_counted_as_deleted(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            git = lambda *a: subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *a], cwd=root,
                                            check=True, env=dr._git_env(), capture_output=True)
            git("init", "-q", "-b", "work")
            (root / "x").write_text("x\n")
            git("add", "-A")
            git("commit", "-qm", "fork")
            git("checkout", "-qb", "main")
            case = root / "spec" / "conformance" / "core-01" / "a.cp"
            case.parent.mkdir(parents=True)
            case.write_text("// expect: ok\n")
            git("add", "-A")
            git("commit", "-qm", "main gains a case")
            git("checkout", "-q", "work")
            self.assertEqual(dr.next_cases_removed(root, "main"), [])


class CiOnlyCountersTest(unittest.TestCase):
    def test_a_counter_left_to_ci_is_never_tightened_from_its_absence(self) -> None:
        self.assertIn("explain_copy_mismatches", dr.CI_ONLY)
        self.assertNotIn("explain_copy_mismatches",
                         dr.improved({"rs_file_size_allowlist": {}}, {"explain_copy_mismatches": 3}))
        self.assertEqual(dr.improved({"rs_file_size_allowlist": {}, "explain_copy_mismatches": 1},
                                     {"explain_copy_mismatches": 3}), ["explain_copy_mismatches"])

    def test_a_local_check_skips_the_ci_only_counters(self) -> None:
        seen = {}
        def measure(root, max_lines, only=None):
            seen["only"] = only
            raise SystemExit(0)
        with mock.patch.object(dr, "measure", measure), mock.patch.dict(os.environ, {}, clear=False):
            os.environ.pop("CI", None)
            os.environ.pop("CPL_RATCHET_ALL", None)
            with self.assertRaises(SystemExit):
                dr.main(["--check"])
        self.assertTrue(seen["only"])
        self.assertFalse(set(dr.CI_ONLY) & seen["only"])


class CppRuntimeLinesTest(unittest.TestCase):
    def test_comments_and_a_test_module_are_not_emitter_cpp(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            src = root / "crates/cpl-next-syntax/src"
            src.mkdir(parents=True)
            (src / "lexer.rs").write_text(
                '// r"not a raw string\nconst H: &str = r"int f();";\n'
                '#[cfg(test)]\nmod tests {\n    const T: &str = r"x\n    y";\n}\n')
            self.assertEqual(dr.next_cpp_runtime_lines(root), ["crates/cpl-next-syntax/src/lexer.rs:2"])


if __name__ == "__main__":
    unittest.main()
