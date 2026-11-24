#!/usr/bin/env python3
"""Unit tests for scripts/cxx-cache.py, against a fake g++ that logs its calls."""

import os
import subprocess
import tempfile
import unittest

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

FAKE_GXX = """#!/bin/sh
if [ "$1" = -print-search-dirs ]; then
    [ -n "$FAKE_NO_DIRS" ] || echo "libraries: =$FAKE_LIB"
    exit 0
fi
if [ "$1" = -xc++ ] && [ "$2" = -E ]; then
    if [ -z "$FAKE_NO_DIRS" ]; then
        printf '#include <...> search starts here:\\n %s\\nEnd of search list.\\n' "$FAKE_INC" >&2
    fi
    exit 0
fi
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
        self.inc, self.lib = os.path.join(t, "inc"), os.path.join(t, "lib")
        os.makedirs(os.path.join(self.inc, "bits"))
        os.makedirs(self.lib)
        with open(os.path.join(self.inc, "bits", "stdc++.h"), "w") as f:
            f.write("v1")
        self.env = dict(
            os.environ,
            PATH=os.pathsep.join([shim_dir, fake_dir, os.environ["PATH"]]),
            CPL_CXX_SHIM_DIR=shim_dir,
            CPL_CXX_CACHE_DIR=os.path.join(t, "cache"),
            FAKE_LOG=self.log,
            FAKE_INC=self.inc,
            FAKE_LIB=self.lib,
        )
        for v in ("CPL_CXX_SYSTEM_KEY", "CPL_CXX_CACHE_MAX_BYTES", "CPATH",
                  "CPLUS_INCLUDE_PATH", "C_INCLUDE_PATH", "LIBRARY_PATH"):
            self.env.pop(v, None)
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

    def test_a_wrapper_or_second_shim_that_reruns_gxx_from_path_does_not_loop(self):
        t = self.tmp.name
        wrap, shim2 = os.path.join(t, "wrap"), os.path.join(t, "shim2")
        os.makedirs(wrap)
        os.makedirs(shim2)
        with open(os.path.join(wrap, "g++"), "w") as f:
            f.write('#!/bin/sh\nexec g++ "$@"\n')
        os.chmod(os.path.join(wrap, "g++"), 0o755)
        os.symlink(os.path.join(ROOT, "scripts", "cxx-cache.py"), os.path.join(shim2, "g++"))
        shim, rest = self.env["PATH"].split(os.pathsep, 1)
        self.env["PATH"] = os.pathsep.join([shim, shim2, wrap, rest])
        r = subprocess.run(["g++", "-O2", self.src, "-o", "x"], env=self.env, capture_output=True,
                           text=True, cwd=t, timeout=30)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(self.compiles(), 1)

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

    def test_a_native_build_on_another_cpu_compiles_again(self):
        cpu = os.path.join(self.tmp.name, "cpuinfo")
        self.env["CPL_CXX_CPU_INFO"] = cpu
        for model in ("model name : A\nflags : avx2", "model name : B\nflags : avx512f"):
            with open(cpu, "w") as f:
                f.write(model)
            self.gxx("-O2", "-march=native", self.src, "-o", "x")
            self.gxx("-O2", self.src, "-o", "y")
        self.assertEqual(self.compiles(), 3)

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

    def test_a_quoted_include_or_search_flag_always_compiles(self):
        self.write('#include "local.hpp"\nint main() {}')
        for _ in range(2):
            self.gxx(self.src, "-o", "x")
        self.write("int main() {}")
        for _ in range(2):
            self.gxx("-I.", self.src, "-o", "x")
        self.env["CPLUS_INCLUDE_PATH"] = self.tmp.name
        for _ in range(2):
            self.gxx(self.src, "-o", "x")
        self.assertEqual(self.compiles(), 6)

    def test_a_changed_system_header_or_library_compiles_again(self):
        self.gxx(self.src, "-o", "x")
        self.gxx(self.src, "-o", "x")
        self.assertEqual(self.compiles(), 1)
        header = os.path.join(self.inc, "bits", "stdc++.h")
        with open(header, "w") as f:
            f.write("v2, a different size")
        self.gxx(self.src, "-o", "x")
        self.assertEqual(self.compiles(), 2)
        with open(os.path.join(self.lib, "libasan.so"), "w") as f:
            f.write("new")
        self.gxx(self.src, "-o", "x")
        self.assertEqual(self.compiles(), 3)

    def test_the_system_key_from_the_gate_is_part_of_the_key(self):
        self.env["CPL_CXX_SYSTEM_KEY"] = "a"
        self.gxx(self.src, "-o", "x")
        self.gxx(self.src, "-o", "x")
        self.env["CPL_CXX_SYSTEM_KEY"] = "b"
        self.gxx(self.src, "-o", "x")
        self.assertEqual(self.compiles(), 2)
        r = subprocess.run(["g++", "--cpl-system-key"], env=dict(self.env, CPL_CXX_SYSTEM_KEY=""),
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 0)
        self.assertEqual(len(r.stdout.strip()), 64)

    def test_a_compiler_that_hides_its_search_dirs_always_compiles(self):
        self.env["FAKE_NO_DIRS"] = "1"
        for _ in range(2):
            self.gxx(self.src, "-o", "x")
        self.assertEqual(self.compiles(), 2)

    def test_the_least_recently_used_entries_are_evicted_past_the_cap(self):
        self.env["CPL_CXX_CACHE_MAX_BYTES"] = "400"
        for body in ("int a;", "int b;", "int c;"):
            self.write(body + " " * 100)
            self.gxx(self.src, "-o", "x")
        self.write("int a;" + " " * 100)
        self.gxx(self.src, "-o", "x")
        self.write("int c;" + " " * 100)
        self.gxx(self.src, "-o", "x")
        self.assertEqual(self.compiles(), 4)
        total = 0
        for r, _, fs in os.walk(self.env["CPL_CXX_CACHE_DIR"]):
            for f in fs:
                self.assertFalse(f.endswith(".tmp"), f)
                total += os.path.getsize(os.path.join(r, f))
        self.assertLessEqual(total, 400)


if __name__ == "__main__":
    unittest.main()
