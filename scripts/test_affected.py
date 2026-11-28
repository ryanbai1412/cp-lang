"""Tests for scripts/affected.py, the commit tier's path selection."""

import io
import os
import subprocess
import tempfile
import sys
import re
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import affected  # noqa: E402

PKGS = "corpus/problems"

DIRS = {"crates/cpl-fmt/": "cpl-fmt", "crates/cpl-check/": "cpl-check",
        "crates/cpl-prover/": "cpl-prover"}


class AffectedTest(unittest.TestCase):
    def sel(self, *paths):
        return affected.select(list(paths), DIRS)

    def test_std_and_the_checker_select_every_dream(self):
        for path in ("std/vec.cp", "crates/cpl-check/src/lib.rs"):
            self.assertTrue(self.sel(path)["dreams"], path)
        for path in ("crates/cpl-fmt/src/lib.rs", "corpus/problems/p1/dream.cp", "web/x.ts"):
            self.assertFalse(self.sel(path)["dreams"], path)

    def test_the_shared_lexer_and_old_crates_select_old_pipeline_same(self):
        for path in ("crates/cpl-next-syntax/src/lexer.rs", "crates/cpl-syntax/src/lib.rs",
                     "crates/cpl-facade/src/lib.rs", "crates/cpl-check/src/lib.rs", "Cargo.lock"):
            self.assertTrue(self.sel(path)["old_same"], path)
        for path in ("crates/cpl-next-check/src/lib.rs", "std/vec.cp", "web/x.ts"):
            self.assertFalse(self.sel(path)["old_same"], path)

    def test_a_crate_file_selects_its_package_by_name_not_directory(self):
        s = self.sel("crates/cpl-prover/src/lib.rs")
        self.assertEqual((s["all"], s["packages"]), (False, ["cpl-prover"]))
        self.assertEqual(s["rust_files"], ["crates/cpl-prover/src/lib.rs"])

    def test_prose_selects_nothing(self):
        s = self.sel("plan/QUESTIONS.md", "decisions.md", "research/x.md")
        self.assertEqual((s["all"], s["packages"], s["script_tests"], s["site"]),
                         (False, [], [], False))

    def test_build_settings_and_std_select_every_crate(self):
        for path in ("Cargo.lock", "std/seq.cp", ".config/nextest.toml"):
            self.assertEqual(self.sel(path)["packages"], sorted(DIRS.values()), path)

    def test_an_unknown_path_selects_the_whole_fast_gate(self):
        for path in (f"{PKGS}/a/sol.cp", "spec/core/types.md", "new-dir/x", "scripts/no-test.py"):
            s = self.sel("crates/cpl-fmt/src/lib.rs", path)
            self.assertEqual((s["all"], s["reason"]), (True, path))

    def test_an_unknown_path_keeps_the_crates_the_other_paths_select(self):
        # 277f37f42: cpl-check plus spec/core/08-traits-generics.md; land's cut rerun
        # ran nothing, so rdeps(cpl-check)'s cpl-lsp red reached main.
        s = self.sel("spec/core/08-traits-generics.md", "crates/cpl-check/src/check/call.rs", "new-dir/x")
        self.assertEqual((s["all"], s["reason"], s["packages"]), (True, "spec/core/08-traits-generics.md", ["cpl-check"]))

    def test_scripts_select_their_unit_tests(self):
        s = self.sel("scripts/land.sh", "scripts/affected.py", ".githooks/pre-commit")
        self.assertEqual(s["script_tests"], ["scripts/test_affected.py", "scripts/test_check_cache.py"])

    def test_a_workflow_selects_its_tests_and_actionlint(self):
        s = self.sel(".github/workflows/ci.yml", ".github/actions/install-tools/action.yml")
        self.assertEqual((s["all"], s["script_tests"]), (False, ["scripts/test_workflows.py"]))

    def test_the_justfile_selects_the_test_that_just_parses_it(self):
        s = self.sel("justfile")
        self.assertEqual((s["all"], s["script_tests"]), (False, ["scripts/test_justfile.py"]))

    def test_a_checking_script_selects_its_own_step_not_the_fast_gate(self):
        s = self.sel("scripts/std-trait-matrix.py", "bench/sizes.py", "scripts/debt-ratchet.py")
        self.assertFalse(s["all"])
        self.assertEqual(s["steps"], ["bench/sizes.py", "scripts/debt-ratchet.py", "scripts/std-trait-matrix.py"])
        self.assertEqual(s["script_tests"], ["scripts/test_debt_ratchet.py"])

    def test_a_script_only_ci_runs_selects_nothing(self):
        self.assertFalse(self.sel("scripts/site-deployable.sh")["all"])
        self.assertTrue(self.sel("scripts/gen-unmapped-thing.py")["all"])

    def test_editor_files_select_the_crate_that_reads_them(self):
        s = self.sel("editors/neovim/autoload/cplang.vim", "editors/lsp-matrix.json")
        self.assertEqual((s["all"], s["packages"]), (False, ["cpl-cli", "cpl-lsp"]))
        self.assertTrue(self.sel("editors/emacs/cp-mode.el")["all"])

    def test_a_conformance_case_selects_itself_not_the_whole_gate(self):
        s = self.sel("spec/conformance/lints/tree-order.cp")
        self.assertEqual((s["all"], s["conform_cases"]), (False, ["spec/conformance/lints/tree-order.cp"]))
        self.assertTrue(self.sel("spec/conformance/COVERAGE.md")["all"])

    def test_handbook_docs_select_the_handbook_check(self):
        s = self.sel("handbook/domains/infra.md")
        self.assertEqual((s["all"], s["packages"]), (False, []))
        self.assertEqual(s["script_tests"], ["scripts/test_check_handbook.py"])

    def test_a_corpus_problem_change_runs_the_package_audit(self):
        self.assertTrue(self.sel(f"{PKGS}/cf2/checker.py")["audit"])
        self.assertTrue(self.sel(f"{PKGS}/cf2/spec.cp")["audit"])
        self.assertFalse(self.sel("research/acceptance/solutions/x/linux/cf1.cp")["audit"])

    def test_a_corpus_package_file_runs_only_the_audit(self):
        for leaf in ("gen.py", "tests.toml", "orig.cpp", "meta.json"):
            sel = self.sel(f"{PKGS}/cf2/{leaf}")
            self.assertFalse(sel["all"], leaf)
            self.assertTrue(sel["audit"], leaf)

    def test_idioms_std_and_checker_changes_run_the_lint_catalog_check(self):
        for path in ("spec/idioms/lints.md", "std/seq.cp", "crates/cpl-check/src/lib.rs"):
            self.assertTrue(self.sel(path)["idioms_catalog"], path)
        self.assertFalse(self.sel("crates/cpl-lint/src/lib.rs")["idioms_catalog"])

    def test_a_std_signature_change_runs_the_cases_that_call_it(self):
        # 8f1685382 made splice_back's `other` mut; three cases calling it went red on main.
        old = {"std/deque.cp": "    pub fn splice_back(mut self, other: LinkedList<T>):\n"
                               "    fn helper(x: Int):\n    pub fn moved(a: Int):\n        pass\n"}
        new = {"std/deque.cp": "    pub fn splice_back(mut self, mut other: LinkedList<T>):\n"
                               "    fn helper(y: Int):\n    pub fn moved(a: Int):\n        pass\n"}
        self.assertEqual(affected.changed_pub_fns(old, new), ["splice_back"])
        # ea6920634 changed only matroid_intersection's `weight:` line; flow-matching calls it
        # as `matroid_intersection<E>(..)` and went red on main.
        sig = "pub fn matroid_intersection<E, A: Matroid<E>, B: Matroid<E> > (\n    a: A,\n    weight: {} ,\n) -> [E]Bool:\n    loop:\n"
        self.assertEqual(affected.changed_pub_fns({"std/flow.cp": sig.format("?(E -> Int) = None")},
                                                  {"std/flow.cp": sig.format("E -> Int = _ => 0")}),
                         ["matroid_intersection"])
        self.assertEqual(affected.changed_pub_fns({"std/flow.cp": sig.format("x")}, {"std/flow.cp": sig.format("x") + "        pass\n"}), [])
        self.assertIn("spec/conformance/std-models/flow-matching.cp", affected.callers(["matroid_intersection"]))
        # 0c04f356d added a law to Action, changing no signature; laws-are-a-derived-member uses std.algebra.
        found = affected.importers(["std/algebra.cp"])
        self.assertIn("spec/conformance/core-08/laws-are-a-derived-member.cp", found)
        self.assertNotIn("spec/conformance/std-deque/splice-empty-lists.cp", found)
        self.assertEqual(affected.importers(["crates/cpl-check/src/lib.rs"]), [])
        found = affected.callers(["splice_back"])
        self.assertIn("spec/conformance/std-deque/splice-empty-lists.cp", found)
        self.assertFalse([f for f in found if not f.startswith(("spec/conformance/", "corpus/problems/"))])
        s = self.sel("std/deque.cp")
        affected.add_callers(s, ["spec/conformance/a.cp", "corpus/problems/p/port.cp"])
        self.assertEqual((s["conform_cases"], s["check_files"]), (["spec/conformance/a.cp"], ["corpus/problems/p/port.cp"]))

    def test_std_and_traits_toml_changes_run_the_traits_spec_gate(self):
        # 2e752de52 added `impl Show for Filtered` with no traits.toml case; only regen noticed.
        for path in ("std/seq.cp", "spec/conformance/traits.toml"):
            self.assertTrue(self.sel(path)["spec_traits"], path)
        self.assertFalse(self.sel("crates/cpl-check/src/lib.rs")["spec_traits"])

    def test_a_lint_record_runs_the_code_uniqueness_test(self):
        s = self.sel("spec/lints/loop_reduction/lint.toml", "spec/checker-lints/x/lint.toml")
        self.assertEqual(s["data_filter"], "(package(cpl-lint) & test(/record_codes_are_unique/))")
        self.assertEqual(s["check_files"], [])

    def test_acceptance_and_perf_programs_get_cpl_check_only(self):
        s = self.sel("research/acceptance/solutions/2026-11-23-5/linux/cf1.cp",
                     f"{PKGS}/cf2/perf.cp")
        self.assertFalse(s["all"])
        self.assertEqual(s["data_filter"], "")
        self.assertEqual(s["check_files"], ["research/acceptance/solutions/2026-11-23-5/linux/cf1.cp",
                                            f"{PKGS}/cf2/perf.cp"])

    def test_corpus_programs_select_the_tests_that_read_them(self):
        s = self.sel(f"{PKGS}/cf1/spec.cp", f"{PKGS}/cf2/port.cp")
        self.assertEqual((s["all"], s["packages"]), (False, []))
        self.assertEqual(s["data_filter"], "(package(cpl-lint) & test(/corpus_warnings/))")
        self.assertEqual(s["check_files"], [f"{PKGS}/cf1/spec.cp", f"{PKGS}/cf2/port.cp"])

    def test_site_paths_select_the_snippet_gate(self):
        self.assertTrue(self.sel("web/src/pages/index.astro")["site"])

    def test_a_removed_std_spelling_selects_the_snippet_gate(self):
        for path in ("crates/cpl-check/src/check/removed.rs", "std/prelude.cp"):
            self.assertTrue(self.sel(path)["site"], path)
        self.assertFalse(self.sel("crates/cpl-check/src/check/expr.rs")["site"])

    def test_the_real_workspace_maps_every_crate_directory(self):
        dirs = affected.crate_dirs()
        for d in sorted(os.listdir(os.path.join(affected.ROOT, "crates"))):
            if os.path.isfile(os.path.join(affected.ROOT, "crates", d, "Cargo.toml")):
                self.assertIn(f"crates/{d}/", dirs)

    def test_shell_output_names_the_rdeps_filter(self):
        out = subprocess.run([sys.executable, os.path.join(affected.ROOT, "scripts", "affected.py"),
                              "crates/cpl-fmt/src/lib.rs", "crates/cpl-check/src/lib.rs"],
                             capture_output=True, text=True, check=True).stdout
        self.assertIn("filter=rdeps(cpl-check) | rdeps(cpl-fmt)\n", out)
        self.assertIn("all=0\n", out)


    def test_std_and_vm_paths_select_their_owners_slices(self):
        self.assertEqual(self.sel("std/seq.cp")["slices"], ["vm-trace", "corpus-warnings", "ports-check", "browser-std", "silent-unknowns",
                                                         "copies", "editor-contract"])
        noip = affected.shard_of("corpus/problems/noip18p4/port.cp", 16)
        self.assertEqual(self.sel("corpus/problems/noip18p4/port.cp")["slices"], ["silent-unknowns", f"copies@{noip}"])
        bag = affected.shard_of("spec/conformance/std-bag/x.cp", 16)
        self.assertEqual(self.sel("spec/conformance/std-bag/x.cp")["slices"], ["silent-unknowns", f"copies@{bag}", "readable-inputs"])
        self.assertEqual(self.sel("crates/cpl-codegen/src/emit.rs")["slices"],
                         ["vm-trace", "corpus-warnings", "ports-check", "copies", "readable-inputs", "editor-contract"])
        self.assertEqual(self.sel("crates/cpl-interp/src/vm/run.rs")["slices"],
                         ["vm-trace", "corpus-warnings", "ports-check", "copies", "readable-inputs", "editor-contract"])
        self.assertEqual(self.sel("crates/cpl-fmt/src/lib.rs")["slices"],
                         ["vm-trace", "ports-check", "copies", "editor-contract"])

    def test_every_slice_maps_the_inputs_its_tests_read(self):
        """Each slice test's own source, the repo data it walks (path literals), and the
        workspace crates it names select its slice, so a change to any of them can't land unrun."""
        root = Path(affected.ROOT)
        lits = re.compile(r'"((?:\.\./\.\./)?(?:corpus/[a-z_]+|spec/[a-z_]+|std|tests/programs))"|(STD_DIR)')
        missing = []
        for name, _, _, tests in affected.SLICES:
            for t in tests:
                src, crate = affected.test_source(t), t.split("::")[0].split(" ")[0]
                self.assertTrue((root / src).is_file(), src)
                text = (root / src).read_text()
                inputs = [src]
                for m in lits.finditer(text):
                    d = "std" if m[2] else m[1].removeprefix("../../")
                    d = f"crates/{crate}/{d}" if d == "tests/programs" else d
                    # A package's tests read its spec.cp at least.
                    sample = next((p for p in sorted((root / d).glob("*/spec.cp" if d == affected.PACKAGES else "**/*.cp"))), None)
                    if sample:
                        inputs.append(str(sample.relative_to(root)))
                inputs += [f"crates/{d.replace('_', '-')}/src/lib.rs"
                           for d in set(re.findall(r"\b(cpl_[a-z_]+)::", text)) if d.replace("_", "-") != crate]
                missing += [f"{name}: {t} reads {p}" for p in inputs if name not in affected.base(self.sel(p)["slices"])]
        self.assertEqual(missing, [], "map these in affected.SLICES:\n" + "\n".join(missing))


    def test_every_front_end_crate_runs_the_trace_slice(self):
        """A trace program goes through the parser, the checker and its messages before
        lowering and the VM run it, so a change to any of them can stop it checking."""
        for c in ("cpl-syntax", "cpl-diag", "cpl-check", "cpl-ir", "cpl-interp"):
            self.assertIn("vm-trace", self.sel(f"crates/{c}/src/lib.rs")["slices"], c)

    def test_what_an_editor_reads_runs_the_editor_contract_slice(self):
        """The server, the commands' output and the extensions' settings got past land
        unrun (06ca25c43, d641b26af, 426b6cd57), since the commit tier skips cpl-cli's and
        cpl-lsp's integration tests."""
        for path in ("crates/cpl-lsp/src/server/exec.rs", "editors/vscode/src/extension.ts",
                     "editors/jetbrains/src/main/kotlin/Cpl.kt", "crates/cpl-cli/src/run_view.rs",
                     "crates/cpl-diag/src/view.rs",
                     "spec/tools/cli.md", "spec/tools/lsp.md"):
            self.assertIn("editor-contract", self.sel(path)["slices"], path)

    def test_a_crate_a_tested_binary_links_runs_the_slice(self):
        """41ffcffe5 changed only cpl-codegen's output and landed with cpl-cli's
        output_pins unrun: output_pins runs the `cpl` binary, which links cpl-codegen."""
        for path in ("crates/cpl-codegen/src/emit.rs", "crates/cpl-codegen/src/lib.rs"):
            self.assertIn("editor-contract", self.sel(path)["slices"], path)
        self.assertLessEqual({"cpl-cli", "cpl-codegen"}, affected.crate_deps("cpl-cli"))

    def test_a_lint_or_checker_change_runs_the_corpus_warnings_slice(self):
        for path in ("crates/cpl-lint/src/rules.rs", "crates/cpl-check/src/lib.rs"):
            self.assertIn("corpus-warnings", self.sel(path)["slices"], path)

    def test_a_slice_filter_names_its_binaries_and_tests(self):
        self.assertEqual(affected.slice_filter(["corpus-warnings"]),
                         "(binary_id(cpl-lint) & test(=corpus_tests::corpus_warnings_only_go_down))")
        self.assertEqual(affected.slice_filter(["vm-trace"]),
                         "binary_id(cpl-cli::trace_goldens) | binary_id(cpl-cli::vm_fallbacks)")

    def test_a_change_to_programs_alone_runs_only_their_shards(self):
        # crates/cpl-cli/tests/read_only_copies.rs pins the same paths to the same shards.
        self.assertEqual(affected.shard_of("spec/conformance/std-models/heaps.cp", 16), 8)
        self.assertEqual(affected.shard_of("corpus/problems/cf2165c/port.cp", 16), 4)
        both = ["spec/conformance/std-models/heaps.cp", "corpus/problems/cf2165c/port.cp"]
        self.assertEqual(self.sel(*both)["slices"], ["silent-unknowns", "copies@4,8", "readable-inputs"])
        self.assertEqual(self.sel(both[0], "crates/cpl-codegen/src/emit.rs")["slices"],
                         ["vm-trace", "corpus-warnings", "ports-check", "silent-unknowns", "copies", "readable-inputs",
                          "editor-contract"])
        self.assertEqual(affected.slice_filter(["copies@8", "copies@4"]),
                         "(binary_id(cpl-cli::read_only_copies) & (not test(/^shard_/) | test(=shard_04) | test(=shard_08)))")
        self.assertEqual(affected.slice_filter(["copies@8", "copies"]), "binary_id(cpl-cli::read_only_copies)")

    def test_a_slice_takes_its_slowest_test(self):
        out = ("        PASS [  14.789s] (2/3) cpl-cli::trace_goldens every_program\n"
               "        PASS [   3.000s] (1/3) cpl-cli::vm_fallbacks shrink\n"
               "        FAIL [  21.500s] (3/3) cpl-lint corpus_tests::corpus_warnings_only_go_down\n"
               "        PASS [  30.000s] (3/3) cpl-lint other::test\n")
        self.assertEqual(affected.slice_times(out, ["vm-trace", "corpus-warnings"]),
                         {"vm-trace": 14.789, "corpus-warnings": 21.5})

    def test_a_slow_slice_warns_without_refusing_the_commit(self):
        out = "        PASS [  33.800s] (1/1) cpl-cli::vm_fallbacks shrink\n"
        done = subprocess.CompletedProcess([], 0, out, "")
        with tempfile.TemporaryDirectory() as tmp, \
                mock.patch.object(affected, "ROOT", tmp), \
                mock.patch.object(affected, "build_slices", lambda names: 0), \
                mock.patch.object(affected.subprocess, "run", lambda *a, **k: done), \
                mock.patch("sys.stdout", new_callable=io.StringIO) as printed:
            os.mkdir(os.path.join(tmp, ".git"))
            self.assertEqual(affected.run_slices(["vm-trace"]), 0)
            with open(os.path.join(tmp, ".git", "cpl-slow-slices.log"), encoding="utf-8") as f:
                self.assertRegex(f.read(), r"^\d+ vm-trace 33.8s consumers \(trace\)\n$")
        self.assertIn("WARNING: slice vm-trace took 33.8s", printed.getvalue())
        failed = subprocess.CompletedProcess([], 100, out.replace("PASS", "FAIL"), "")
        with mock.patch.object(affected, "ROOT", "/nonexistent"), \
                mock.patch.object(affected, "build_slices", lambda names: 0), \
                mock.patch.object(affected.subprocess, "run", lambda *a, **k: failed), \
                mock.patch("sys.stdout", new_callable=io.StringIO):
            self.assertEqual(affected.run_slices(["vm-trace"]), 100)

    def test_a_slice_with_a_base_fails_only_on_the_reds_the_base_lacks(self):
        ran = []
        done = subprocess.CompletedProcess([], 0, "", "")
        with mock.patch.object(affected, "ROOT", "/nonexistent"), \
                mock.patch.object(affected, "build_slices", lambda names: 0), \
                mock.patch.object(affected.subprocess, "run", lambda cmd, **k: ran.append(cmd) or done), \
                mock.patch("sys.stdout", new_callable=io.StringIO):
            self.assertEqual(affected.main(["--run-slices", "--base", "abc", "vm-trace"]), 0)
            self.assertEqual(affected.main(["--run-slices", "vm-trace"]), 0)
        self.assertEqual(ran[0][1:5], ["/nonexistent/scripts/new_reds.py", "--base", "abc", "--"])
        self.assertEqual(ran[0][5:], ran[1])
        self.assertEqual(ran[1][:3], ["cargo", "nextest", "run"])

    def test_ci_runs_every_slice_and_counts_the_slow_ones_in_its_summary(self):
        out = ("        PASS [  33.800s] (1/2) cpl-cli::vm_fallbacks shrink\n"
               "        PASS [   2.000s] (2/2) cpl-cli::read_only_copies every_program\n")
        done = subprocess.CompletedProcess([], 0, out, "")
        ran = []
        with tempfile.TemporaryDirectory() as tmp, \
                mock.patch.object(affected, "ROOT", tmp), \
                mock.patch.object(affected, "build_slices", lambda names: ran.append(names) or 0), \
                mock.patch.object(affected.subprocess, "run", lambda *a, **k: done), \
                mock.patch.dict(os.environ, {"GITHUB_ACTIONS": "true",
                                             "GITHUB_STEP_SUMMARY": os.path.join(tmp, "summary.md")}), \
                mock.patch("sys.stdout", new_callable=io.StringIO) as printed:
            self.assertEqual(affected.run_slices(["all"]), 0)
            with open(os.path.join(tmp, "summary.md"), encoding="utf-8") as f:
                self.assertEqual(f.read(), f"slow slices: 1 of {len(affected.SLICES)} over 20 s\n"
                                           "- vm-trace 33.8s (consumers (trace))\n")
        self.assertEqual(ran, [[s[0] for s in affected.SLICES]])
        self.assertIn("::warning::affected: WARNING: slice vm-trace took 33.8s", printed.getvalue())

    def test_every_slice_has_an_owner_paths_and_tests(self):
        names = [s[0] for s in affected.SLICES]
        self.assertEqual(len(names), len(set(names)))
        for name, owner, globs, tests in affected.SLICES:
            self.assertTrue(owner and globs and tests, name)

    def test_an_unknown_slice_is_refused(self):
        self.assertEqual(affected.run_slices(["nope"]), 2)

    def test_base_selects_the_paths_changed_since_it(self):
        out = subprocess.run([sys.executable, os.path.join(affected.ROOT, "scripts", "affected.py"),
                              "--json", "--base", "HEAD"],
                             capture_output=True, text=True, check=True, cwd=affected.ROOT).stdout
        self.assertIn('"packages": []', out)
        self.assertIn('"all": false', out)

if __name__ == "__main__":
    unittest.main()
