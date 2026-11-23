import importlib.util
import json
import os
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
            self._write(root, "handbook/design/architecture.md", "`crates/cpl-lint` and `crates/gone/x.rs`\n")
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

    def test_register_rows_counts_missing_and_unknown_checks(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._write(root, "scripts/ok.py", "")
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
                "## Suites", "| e | `missing` |", ""]))
            self.assertEqual(dr.register_rows(root), [
                "handbook/domains/x.md:9 missing: b",
                "handbook/domains/x.md:10 no such check scripts/gone.py: c",
                "handbook/domains/x.md:15 missing: g",
                "handbook/domains/x.md:17 missing: k",
            ])

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


    def test_debt_already_on_main_is_not_this_changes_error(self):
        base = {"lint_std_names": 10, "rs_file_size_allowlist": {"a.rs": 2100}}
        main = {"lint_std_names": 12, "rs_file_size_allowlist": {"a.rs": 2200}}
        eff = dr.inherited_baseline(base, main)
        same = {"lint_std_names": 12, "rs_file_size_allowlist": {"a.rs": 2200}}
        self.assertEqual(dr.evaluate(same, eff)[0], [])
        more = {"lint_std_names": 13, "rs_file_size_allowlist": {"a.rs": 2201}}
        self.assertEqual(len(dr.evaluate(more, eff)[0]), 2)
        self.assertEqual(base["lint_std_names"], 10)

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

    def test_own_databases_and_copy_baselines(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            for rel in ("crates/cpl-lsp/src/server.rs", "crates/cpl-lsp/src/fix_tests.rs",
                        "crates/cpl-lsp/tests/a.rs", "crates/cpl-check/src/db.rs"):
                (root / rel).parent.mkdir(parents=True, exist_ok=True)
                (root / rel).write_text("let db = Database::new();\n")
            self.assertEqual(dr.own_databases(root), ["crates/cpl-lsp/src/server.rs:1"])
            tests = root / "crates/cpl-cli/tests"
            tests.mkdir(parents=True)
            (tests / "read_only_copies.rs").write_text(
                '    ("deep_copies", deep_copies, 3),\n    ("std_functions", std_functions, 2),\n')
            self.assertEqual(len(dr.cpp_deep_copies(root)), 3)
            self.assertEqual(len(dr.cpp_std_functions(root)), 2)

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
