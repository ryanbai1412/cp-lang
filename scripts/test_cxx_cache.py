#!/usr/bin/env python3
"""Unit tests for scripts/cxx-cache.py, against a fake g++ that logs its calls."""

import os
import subprocess
import tempfile
import unittest

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

FAKE_GXX = """#!/bin/sh
echo "$@" >> "$FAKE_LOG"
out=""; src=""; prev=""
for a in "$@"; do
    [ "$prev" = "-o" ] && out="$a"
    case "$a" in *.cpp) src="$a" ;; -) src=/dev/stdin ;; esac
    prev="$a"
done
body=$(cat "$src")
case "$body" in *FAIL*) echo "error: FAIL" >&2; exit 1 ;; esac
echo "warning: compiled" >&2
printf 'bin %s %s\\n' "$*" "$body" > "$out"
"""


class CxxCacheTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        t = self.tmp.name
        fake_dir, shim_dir = os.path.join(t, "fake"), os.path.join(t, "shim")
        os.makedirs(fake_dir)
        os.makedirs(shim_dir)
        with open(os.path.join(fake_dir, "g++"), "w") as f:
            f.write(FAKE_GXX)
        os.chmod(os.path.join(fake_dir, "g++"), 0o755)
        os.symlink(os.path.join(ROOT, "scripts", "cxx-cache.py"), os.path.join(shim_dir, "g++"))
        self.log = os.path.join(t, "log")
        self.env = dict(
            os.environ,
            PATH=os.pathsep.join([shim_dir, fake_dir, os.environ["PATH"]]),
            CPL_CXX_SHIM_DIR=shim_dir,
            CPL_CXX_CACHE_DIR=os.path.join(t, "cache"),
            FAKE_LOG=self.log,
        )
        self.src = os.path.join(t, "a.cpp")
        self.write("int main() {}")

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, body):
        with open(self.src, "w") as f:
            f.write(body)

    def gxx(self, *args, stdin=None):
        return subprocess.run(["g++", *args], env=self.env, input=stdin,
                              capture_output=True, text=True, cwd=self.tmp.name)

    def compiles(self):
        if not os.path.exists(self.log):
            return 0
        with open(self.log) as f:
            return len(f.readlines())

    def read(self, path):
        with open(os.path.join(self.tmp.name, path)) as f:
            return f.read()

    def test_same_source_and_flags_reuse_the_binary_and_its_warnings(self):
        r1 = self.gxx("-O2", self.src, "-o", "x")
        r2 = self.gxx("-O2", self.src, "-o", "y")
        self.assertEqual((r1.returncode, r2.returncode), (0, 0))
        self.assertEqual(self.compiles(), 1)
        self.assertEqual(self.read("x"), self.read("y"))
        self.assertIn("warning: compiled", r2.stderr)
        self.assertTrue(os.access(os.path.join(self.tmp.name, "y"), os.X_OK))

    def test_a_changed_source_or_flag_compiles_again(self):
        self.gxx("-O2", self.src, "-o", "x")
        self.write("int main() { return 0; }")
        self.gxx("-O2", self.src, "-o", "x")
        self.gxx("-O0", self.src, "-o", "x")
        self.assertEqual(self.compiles(), 3)
        self.assertIn("-O0", self.read("x"))

    def test_a_failed_compile_is_not_kept(self):
        self.write("FAIL")
        self.assertEqual(self.gxx(self.src, "-o", "x").returncode, 1)
        r = self.gxx(self.src, "-o", "x")
        self.assertEqual(r.returncode, 1)
        self.assertIn("error: FAIL", r.stderr)
        self.assertEqual(self.compiles(), 2)

    def test_stdin_source_is_keyed_by_its_bytes(self):
        self.gxx("-x", "c++", "-", "-o", "x", stdin="one")
        self.gxx("-x", "c++", "-", "-o", "x", stdin="one")
        self.gxx("-x", "c++", "-", "-o", "x", stdin="two")
        self.assertEqual(self.compiles(), 2)
        self.assertIn("two", self.read("x"))

    def test_an_input_it_cannot_see_into_always_compiles(self):
        header = os.path.join(self.tmp.name, "h.hpp")
        with open(header, "w") as f:
            f.write("#pragma once")
        for _ in range(2):
            self.gxx("-include", header, self.src, "-o", "x")
        self.assertEqual(self.compiles(), 2)


if __name__ == "__main__":
    unittest.main()
