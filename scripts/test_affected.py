"""Tests for scripts/affected.py, the commit tier's path selection."""

import os
import subprocess
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import affected  # noqa: E402

DIRS = {"crates/cpl-fmt/": "cpl-fmt", "crates/cpl-check/": "cpl-check",
        "crates/cpl-prover/": "cpl-prover"}


class AffectedTest(unittest.TestCase):
    def sel(self, *paths):
        return affected.select(list(paths), DIRS)

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
        for path in ("corpus/problems/a/sol.cp", "spec/core/types.md", "new-dir/x", "scripts/no-test.sh"):
            s = self.sel("crates/cpl-fmt/src/lib.rs", path)
            self.assertEqual((s["all"], s["reason"]), (True, path))

    def test_scripts_select_their_unit_tests(self):
        s = self.sel("scripts/land.sh", "scripts/affected.py", ".githooks/pre-commit")
        self.assertEqual(s["script_tests"], ["scripts/test_affected.py", "scripts/test_check_cache.py"])

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

    def test_corpus_programs_select_the_tests_that_read_them(self):
        s = self.sel("corpus/problems/cf1/spec.cp", "corpus/problems/cf2/port.cp")
        self.assertEqual((s["all"], s["packages"]), (False, []))
        self.assertEqual(s["data_filter"], "(package(cpl-lint) & test(/corpus_warnings/))")
        self.assertEqual(s["check_files"], ["corpus/problems/cf1/spec.cp", "corpus/problems/cf2/port.cp"])

    def test_site_paths_select_the_snippet_gate(self):
        self.assertTrue(self.sel("web/src/pages/index.astro")["site"])

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


    def test_base_selects_the_paths_changed_since_it(self):
        out = subprocess.run([sys.executable, os.path.join(affected.ROOT, "scripts", "affected.py"),
                              "--json", "--base", "HEAD"],
                             capture_output=True, text=True, check=True, cwd=affected.ROOT).stdout
        self.assertIn('"packages": []', out)
        self.assertIn('"all": false', out)

if __name__ == "__main__":
    unittest.main()
