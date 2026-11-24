#!/usr/bin/env python3
"""How long g++ -O2 takes on the C++ `cpl build --judge` writes for every
bench program, against bars measured on that set.

    python3 bench/compile_time.py [--judge J] [--cpl PATH] [-j N] [--check]

Each program's C++ is compiled and linked as a judge does (`g++ -std=c++17
-O2`), and its CPU time (user + system, from the child's rusage) is divided
by the CPU time of compiling BASELINE, the smallest program that includes
the whole standard library, on the same machine. The ratio leaves out how
fast the machine is, so the bars hold on a laptop and a hosted runner alike.
--check fails when the median ratio is over P50_BAR or any program's is over
MAX_BAR. Enforces the stdlib register's "Compiles fast on the judge".
"""
import argparse, os, statistics, subprocess, sys, tempfile
from concurrent.futures import ThreadPoolExecutor

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sizes import ROOT, programs, stale  # noqa: E402

FLAGS = ["-std=c++17", "-O2"]
BASELINE = "#include <bits/stdc++.h>\nint main() {}\n"
# Measured on the bench set: median 0.73, max 1.63 (bench/geometry/convex_hull_3d.cp).
# The bars leave about a third more for noise; lower them as compile time falls.
P50_BAR = 1.0
MAX_BAR = 2.2


def cpu(cmd):
    """CPU seconds (user + system) `cmd` took, or None when it failed."""
    proc = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    _, status, use = os.wait4(proc.pid, 0)
    return use.ru_utime + use.ru_stime if status == 0 else None


def baseline(tmp, runs=3):
    src = os.path.join(tmp, "baseline.cpp")
    with open(src, "w") as f:
        f.write(BASELINE)
    return min(cpu(["g++", *FLAGS, src, "-o", os.path.join(tmp, "baseline")]) for _ in range(runs))


def measure(cpl, judge, path, tmp):
    """CPU seconds g++ took on `path`'s C++, or None when it does not build."""
    stem = os.path.join(tmp, path.replace("/", "_"))
    env = {k: v for k, v in os.environ.items() if k not in ("CPL_EDITION", "CPL_STD", "CPL_SPEC")}
    built = subprocess.run([cpl, "build", path, "--judge", judge, "-o", stem + ".cpp"],
                           cwd=ROOT, capture_output=True, text=True, timeout=120, env=env)
    if built.returncode != 0:
        return None
    return cpu(["g++", *FLAGS, stem + ".cpp", "-o", stem])


def verdict(ratios):
    """What to say about `ratios` (path -> compile time over the baseline's), and
    whether --check fails."""
    if not ratios:
        return ["compile_time: no program compiled"], True
    p50 = statistics.median(ratios.values())
    worst = max(ratios, key=ratios.get)
    out = [f"compile_time: {len(ratios)} programs, median {p50:.2f}x the baseline "
           f"(bar {P50_BAR}), max {ratios[worst]:.2f}x ({worst}, bar {MAX_BAR})"]
    failed = p50 > P50_BAR
    if failed:
        out.append(f"compile_time: the median is over the {P50_BAR}x bar")
    for p, r in sorted(ratios.items(), key=lambda kv: -kv[1]):
        if r > MAX_BAR:
            out.append(f"compile_time: {p} takes {r:.2f}x the baseline, over the {MAX_BAR}x bar")
            failed = True
    return out, failed


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--judge", default="codeforces")
    ap.add_argument("--cpl", default=os.path.join(ROOT, "target", "debug", "cpl"))
    ap.add_argument("-j", type=int, default=max(1, (os.cpu_count() or 2) // 2))
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    newer = stale(args.cpl)
    if newer:
        print(f"compile_time: {args.cpl} is older than {newer}; rebuild it or pass --cpl", file=sys.stderr)
        return 1
    paths = [p for p in programs() if p.startswith("bench/")]
    with tempfile.TemporaryDirectory() as tmp:
        base = baseline(tmp)
        with ThreadPoolExecutor(args.j) as pool:
            got = list(pool.map(lambda p: measure(args.cpl, args.judge, p, tmp), paths))
    ratios = {p: t / base for p, t in zip(paths, got) if t is not None}
    messages, failed = verdict(ratios)
    print(f"compile_time: baseline {base:.2f} s CPU", file=sys.stderr)
    for m in messages:
        print(m, file=sys.stderr)
    return 1 if args.check and failed else 0


if __name__ == "__main__":
    sys.exit(main())
