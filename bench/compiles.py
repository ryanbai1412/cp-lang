#!/usr/bin/env python3
"""Compile the C++ of every program that builds with a standard g++.

    python3 bench/compiles.py [--judge generic] [--cxx g++] [--cpl PATH] [-j N] [PATH...]

Every program that `cpl build` accepts must compile as one C++17 file; a
failure is a back-end bug. Prints one line per program that builds but does
not compile, and exits 1 if there is one.
"""
import argparse, os, shlex, subprocess, sys, tempfile
from concurrent.futures import ThreadPoolExecutor

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sizes import ROOT, programs  # noqa: E402

FLAGS = "-std=c++17 -O2"


def check(cpl, judge, cxx, path, tmp):
    """None when it compiles or does not build, else the first compile error."""
    cpp = os.path.join(tmp, path.replace("/", "_") + ".cpp")
    env = {k: v for k, v in os.environ.items() if k not in ("CPL_EDITION", "CPL_STD", "CPL_SPEC")}
    built = subprocess.run([cpl, "build", path, "--judge", judge, "-o", cpp],
                           cwd=ROOT, capture_output=True, text=True, timeout=120, env=env)
    if built.returncode != 0:
        return "unbuilt"
    run = subprocess.run(shlex.split(cxx) + shlex.split(FLAGS) + ["-fsyntax-only", cpp],
                         capture_output=True, text=True, timeout=300)
    if run.returncode == 0:
        return None
    return next((l for l in run.stderr.splitlines() if "error" in l), run.stderr.strip()[:200])


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--judge", default="generic")
    ap.add_argument("--cxx", default="g++")
    ap.add_argument("--cpl", default=os.path.join(ROOT, "target", "debug", "cpl"))
    ap.add_argument("-j", type=int, default=os.cpu_count())
    ap.add_argument("paths", nargs="*")
    args = ap.parse_args()
    cxx = args.cxx
    paths = args.paths or programs()
    with tempfile.TemporaryDirectory() as tmp, ThreadPoolExecutor(args.j) as pool:
        got = list(pool.map(lambda p: check(args.cpl, args.judge, cxx, p, tmp), paths))
    failed = [(p, why) for p, why in zip(paths, got) if why not in (None, "unbuilt")]
    for p, why in failed:
        print(f"compiles: {p}: {why}")
    built = sum(why != "unbuilt" for why in got)
    print(f"{built} programs built for `{args.judge}`; {len(failed)} do not compile with {cxx}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
