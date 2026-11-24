import importlib.util
import json
import os
import subprocess
import re
import tempfile
import unittest
from unittest import mock
from pathlib import Path

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
"""


class DebtRatchetTest(unittest.TestCase):
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
        (root / "std/seq.cp").write_text("pub fn foo() = todo\nfn bar():\n    todo\n")
        (root / "crates/cpl-cli/tests/pending/case1.cp").write_text("print(1)\n")

    def test_corpus_test_files_counts_every_test_file(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            files = ["corpus/problems/a/tests/sample1.in", "corpus/problems/a/tests/1.in",
                     "corpus/problems/a/tests/1.ans", "corpus/problems/a/gen.py"]
            for f in files:
                (root / f).parent.mkdir(parents=True, exist_ok=True)
                (root / f).write_text("1\n")
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(["git", "-C", str(root), "add", "."], check=True)
            self.assertEqual(dr.corpus_test_files(root), [
                "corpus/problems/a/tests/1.ans", "corpus/problems/a/tests/1.in",
                "corpus/problems/a/tests/sample1.in"])

    def test_corpus_no_max_counts_packages_without_a_max_mode(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            for f, text in [("a/port.cp", ""), ("a/gen.py", "if sys.argv[1:] == ['--max']:\n"),
                            ("b/port.cp", ""), ("b/gen.py", "print(1)\n"), ("c/port.cp", ""),
                            ("d/port.cp", ""), ("d/interactor.py", ""), ("e/gen.py", "")]:
                (root / "corpus/problems" / f).parent.mkdir(parents=True, exist_ok=True)
                (root / "corpus/problems" / f).write_text(text)
            self.assertEqual(dr.corpus_no_max(root), ["corpus/problems/b", "corpus/problems/c"])

    def test_v02_port_files_counts_ports_not_in_the_v03_set(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            for f in ["corpus/problems/a/port.cp", "corpus/problems/b/port.cp",
                      "corpus/problems/c/spec.cp", "corpus/sets/v03.txt"]:
                (root / f).parent.mkdir(parents=True, exist_ok=True)
                (root / f).write_text("b\n")
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(["git", "-C", str(root), "add", "."], check=True)
            self.assertEqual(dr.v02_port_files(root), ["corpus/problems/a/port.cp"])

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
            "stdlib_todo": 2,
            "rust_ignore": 1,
            "dead_code_allows": 1,
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
            "dead_code_allows": 2,
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

    def test_crate_rules_come_from_architecture_md(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self.assertEqual(dr.crate_rules(root), {})
            self._write(root, "handbook/design/architecture.md", CRATE_RULES)
            self.assertEqual(dr.crate_rules(root)["back end"], ["cpl-ir", "cpl-interp", "cpl-codegen"])
            self.assertIn(("cpl-lint", "cpl-prover"), dr.forbidden_edges(root))
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
        self.assertTrue(dr.idiom_edit_stateable(*one_span))
        self.assertTrue(dr.idiom_edit_stateable(*two_findings))
        self.assertFalse(dr.idiom_edit_stateable(*leaves_block))
        self.assertFalse(dr.idiom_edit_stateable(*shared_binding))
        self.assertTrue(dr.idiom_edit_stateable(*two_places))
        self.assertFalse(dr.idiom_edit_stateable("x\n", "x\n"))
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            for name, group, (before, after) in [("in_rust", "idiom", one_span), ("in_std", "idiom", one_span),
                                                 ("style", "style", one_span), ("hard", "idiom", leaves_block)]:
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

    def test_cfg_test_skips_only_the_item_it_gates(self) -> None:
        src = ["a.unwrap();", "#[cfg(test)]", "#[allow(x)]", "fn t() {", '    let s = "}";', "    t.unwrap();",
               "}", "b.unwrap();", "#[cfg(test)]", "mod helpers;", "c.unwrap();", "#[cfg(test)]",
               "mod tests {", "    fn x() { y.unwrap(); }", "}"]
        self.assertEqual([i for i, _ in dr._ungated_lines(src)], [1, 8, 11])

    def test_ci_retries_spares_only_the_helper(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            self._write(root, "scripts/retry.sh", "retry() {\n")
            self._write(root, "scripts/r.py", "for _ in range(tries):\n")
            self._write(root, ".github/workflows/x.yml", "nextest run --retries 1 --flaky-result fail\n"
                        "for try in 1 2 3; do\n  try() { x; }\nnextest run --retries 2\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.ci_retries(root), [".github/workflows/x.yml:1", ".github/workflows/x.yml:2", ".github/workflows/x.yml:3",
                                                   ".github/workflows/x.yml:4", "scripts/r.py:1"])

    def test_web_skip_only_reads_only_js_and_ts(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            self._write(root, "web/tests/a.test.ts", "it.only('x', () => {})\n")
            self._write(root, "web/lib/b.mjs", "describe.skip('y')\n")
            self._write(root, "web/content/c.cp", "let g = G.only(v => v > 0)\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(sorted(dr.web_skip_only(root)), ["web/lib/b.mjs:1", "web/tests/a.test.ts:1"])

    def test_unrun_script_tests(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=dr._git_env())
            self._write(root, "scripts/check.sh", "mapfile -t script_tests < <(git ls-files 'scripts/test_*.py')\n"
                        "uv run python -m unittest tools/x/test_named.py\n")
            for rel in ("scripts/test_a.py", "tools/x/test_named.py", "tools/y/test_b.py", "corpus/test_c.py"):
                self._write(root, rel, "")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            self.assertEqual(dr.unrun_script_tests(root), ["corpus/test_c.py", "tools/y/test_b.py"])

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

    def test_new_debt_counters(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=dr._git_env())
            self._write(root, "crates/a/src/lib.rs", "\n".join([
                "fn f() { x.unwrap(); y.expect(\"e\"); panic!(\"p\") }",
                "// z.unwrap()", "#[cfg(test)]", "mod tests { fn t() { q.unwrap(); } }", ""]))
            self._write(root, "crates/a/src/x_tests.rs", "fn t() { q.unwrap(); }\n")
            self._write(root, "crates/a/build.rs", "fn main() { q.unwrap(); }\n")
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
            self._write(root, "corpus/problems/p/dream.cp", "print(1)\n")
            self._write(root, "corpus/problems/p/meta.json", "{}")
            self._write(root, "corpus/problems/q/dream.cp", "print(2)\n")
            self._write(root, "plan/domains/stdlib/measure.py", "")
            self._write(root, "plan/domains/tools/measure.py", "")
            self._write(root, "scripts/test_stdlib_measure.py", "")
            self._write(root, "scripts/walk.py", "for p in glob.glob('corpus/problems/*'):\n    pass\n")
            self._write(root, "scripts/changed.sh", "git diff --cached --name-only -- 'corpus/problems/*/tests/*'\n")
            self._write(root, "scripts/one.py", "paths = glob.glob('docs/*') + ['!corpus/problems/cf1a/dream.cp']\n")
            self._write(root, "scripts/corpus.py", "os.listdir('corpus/problems')\n")
            self._write(root, "tools/new-allow.txt", "x\n")
            subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=dr._git_env())
            blob = subprocess.run(["git", "rev-parse", ":corpus/problems/q/dream.cp"], cwd=root, check=True,
                                  capture_output=True, text=True, env=dr._git_env()).stdout.strip()
            self._write(root, "corpus/problems/q/meta.json", json.dumps({"dream_stress": {"ok": True, "sha": blob}}))
            self.assertEqual(dr.unwrap_input(root), ["crates/a/src/lib.rs:1"] * 3)
            self.assertEqual(len(dr.unsupported_sites(root)), 2)
            self.assertEqual(dr.lsp_string_types(root), ["crates/cpl-lsp/src/hover.rs:1", "crates/cpl-lsp/src/hover.rs:2"])
            self.assertEqual(dr.lint_post_filters(root), ["crates/cpl-lint/src/typed.rs TYPED"] * 2)
            self.assertEqual(dr.or_true(root), [".github/workflows/ci.yml:5", ".github/workflows/ci.yml:9"])
            self.assertEqual(dr.continue_on_error(root), [".github/workflows/ci.yml:3", ".github/workflows/ci.yml:6"])
            self.assertEqual(dr.dream_unverified(root), ["corpus/problems/p/dream.cp"])
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
            self._write(root, "crates/cpl-check/src/check/advice.rs", "fn a() {}\n    pub(crate) fn b() {}\n")
            self._write(root, "crates/cpl-stress/src/engine.rs", "fn run_case() {}\n")
            self._write(root, "crates/cpl-stress/src/cli.rs", "Verdict::Ok\n")
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
            self.assertEqual(dr.stress_engines(root), ["crates/cpl-stress/src/engine.rs:1", "crates/cpl-stress/src/cli.rs"])
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

    def test_hard_zero_is_assigned_once(self) -> None:
        src = Path(dr.__file__).read_text(encoding="utf-8")
        self.assertEqual(len(re.findall(r"^HARD_ZERO\s*[|:]?=", src, re.M)), 1,
                         "add zero counters to the one HARD_ZERO frozenset, never rebind it")
        self.assertIsInstance(dr.HARD_ZERO, frozenset)

    def test_every_zero_baseline_is_hard_zero(self) -> None:
        baseline = json.loads(dr.DEFAULT_RATCHET.read_text(encoding="utf-8"))
        gated = {n for n, (_, _, g) in dr.COUNTERS.items() if g}
        self.assertEqual({n for n in gated if baseline.get(n) == 0} - dr.HARD_ZERO, set())
        self.assertEqual(dr.HARD_ZERO - set(dr.COUNTERS), set())
        self.assertEqual({n for n in gated if n not in baseline}, set())

    def test_a_gated_counter_without_a_baseline_fails(self) -> None:
        errs, _, _ = dr.evaluate({"stdlib_todo": 1, "rust_ignore": 0}, {"rust_ignore": 0})
        self.assertEqual(len(errs), 1)
        self.assertIn("no baseline", errs[0])

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
            self.assertEqual(dr.uncited_rules(root), ["lint bad", "lint noid", "lint none", "gate scripts/b.py"])

    def test_upstream_on_a_fresh_tip_is_measured_except_in_ci(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            env = dr._git_env()
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=env)
            self._write(root, "crates/demo/src/a.rs", "fn a() {}\n")
            for args in (["add", "-A"], ["-c", "user.name=t", "-c", "user.email=t@t",
                                         "commit", "-qm", "x"]):
                subprocess.run(["git", "-C", str(root), *args], check=True, env=env)
            old = os.environ.pop("CI", None)
            try:
                self.assertIsNotNone(dr.measure_upstream(root, 20, ref="HEAD"))
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
        baseline = {"near_dup_fns": 3, "dup_fn_bodies": 9, "crate_edges": ["a -> b", "a -> c"]}
        actual = {"near_dup_fns": 4, "dup_fn_bodies": 9, "crate_edges": ["a -> b", "a -> d"]}
        errs, _, notes = dr.evaluate(actual, baseline)
        self.assertEqual(len(errs), 2)
        self.assertIn("near-identical", errs[0])
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
            # A hard-zero counter above 0 is never written as a baseline.
            self.assertEqual(dr.main(["--update", "--root", str(root), "--ratchet", str(ratchet)]), 1)
            (root / "crates/cpl-cli/tests/pending/case1.cp").unlink()
            (root / "crates/cpl-ir/src").mkdir(parents=True)
            (root / "crates/cpl-ir/src/instance.rs").write_text("use codes::GROWING_INSTANTIATION;\n")
            (root / dr.SCALING_BOARD).parent.mkdir(parents=True)
            (root / dr.SCALING_BOARD).write_text('{"cases": {}, "unscaled": []}')
            self.assertEqual(dr.main(["--update", "--root", str(root), "--ratchet", str(ratchet)]), 0)
            self.assertTrue(json.loads(ratchet.read_text())["stdlib_todo"] == 2)
            self.assertEqual(dr.main(["--check", "--root", str(root), "--ratchet", str(ratchet)]), 0)


    def test_debt_already_on_main_is_not_this_changes_error(self):
        base = {"lint_std_names": 10, "rs_file_size_allowlist": {"a.rs": 2100}}
        main = {"lint_std_names": 12, "rs_file_size_allowlist": {"a.rs": 2200}}
        eff = dr.inherited_baseline(base, main)
        same = {"lint_std_names": 12, "rs_file_size_allowlist": {"a.rs": 2200}}
        self.assertEqual(dr.evaluate(same, eff)[0], [])
        more = {"lint_std_names": 13, "rs_file_size_allowlist": {"a.rs": 2201}}
        self.assertEqual(len(dr.evaluate(more, eff)[0]), 2)
        self.assertEqual(base["lint_std_names"], 10)

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
            tests = root / "crates/cpl-cli/tests"
            tests.mkdir(parents=True)
            (tests / "vm_fallbacks.txt").write_text("# header\np untraced expr:Read\np traced stmt:Return\n")
            self.assertEqual(len(dr.vm_fallbacks(root)), 2)

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
            case.write_text("// expect: ok\n// copies: 2\n// std_functions: 1\nprint(1)\n// copies: 9\n")
            port = root / "corpus/problems/cf1/port.cp"
            port.parent.mkdir(parents=True)
            port.write_text("// copies: 1\nprint(1)\n")
            (root / "corpus/problems/cf1/notes.md").write_text("// copies: 5\n")
            tests = root / "crates/cpl-cli/tests"
            tests.mkdir(parents=True)
            (tests / "read_only_copies.rs").write_text('    [("std_functions", std_functions, 2)];\n')
            self.assertEqual(len(dr.cpp_deep_copies(root)), 3)
            self.assertEqual(len(dr.cpp_std_functions(root)), 2)

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

    def test_a_raised_baseline_needs_a_decision_naming_it(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            git = lambda *a: subprocess.run(["git", *a], cwd=root, check=True, capture_output=True,
                                            env=dr._git_env())
            git("init", "-q")
            (root / "scripts").mkdir()
            (root / "scripts/debt-ratchet.json").write_text(json.dumps({"lint_std_names": 5}))
            (root / "decisions.md").write_text("# Decisions\n")
            git("add", ".")
            git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "base")
            raised = {"lint_std_names": 6}
            self.assertEqual(len(dr.unexplained_bumps(root, raised, ref="HEAD")), 1)
            self.assertEqual(dr.unexplained_bumps(root, {"lint_std_names": 4}, ref="HEAD"), [])
            with open(root / "decisions.md", "a") as f:
                f.write("## 9. lint_std_names grows by one for X (2026-11-23)\n")
            self.assertEqual(dr.unexplained_bumps(root, raised, ref="HEAD"), [])

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


if __name__ == "__main__":
    unittest.main()
