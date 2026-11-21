#!/usr/bin/env python3
"""Prefix/Diff/FillPrefix stdlib benchmarks: each X.cp, built by `cpl build`,
against matched hand-written X_hand.cpp on identical inputs. Both are compiled
with g++ -std=c++17 -O2. Prints a Markdown table with best-of-5 wall time and
emitted C++ bytes; writes nothing tracked.

    cargo build --release -p cpl-cli
    python3 bench/prefix/run.py
    python3 bench/prefix/run.py prefix_1d --runs 3
"""
import argparse, os, statistics, subprocess, sys, tempfile, time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
CPL = os.environ.get("CPL", os.path.join(ROOT, "target", "release", "cpl"))

BENCHES = [
    ("prefix_1d", "n=524k, q=10M", "524288 10000000 42\n"),
    ("prefix_2d", "1024x1024, q=10M", "1024 1024 10000000 42\n"),
    ("diff_1d", "n=524k, q=10M", "524288 10000000 42\n"),
    ("diff_2d", "1024x1024, q=10M", "1024 1024 10000000 42\n"),
    ("fill_1d_fwd", "n=10M", "10000000 42\n"),
    ("fill_1d_rev", "n=10M", "10000000 42\n"),
    ("fill_2d_fwd", "2048x2048", "2048 2048 42\n"),
    ("fill_2d_rev", "2048x2048", "2048 2048 42\n"),
]


def run_timed(binary, data, runs):
    best = float("inf")
    out = None
    for _ in range(runs):
        t0 = time.perf_counter()
        p = subprocess.run([binary], input=data, capture_output=True, text=True, check=True)
        dt = time.perf_counter() - t0
        if out is None:
            out = p.stdout.strip()
        elif p.stdout.strip() != out:
            raise RuntimeError(f"non-deterministic output: {out!r} vs {p.stdout.strip()!r}")
        if dt < best:
            best = dt
    return best, out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("only", nargs="*")
    ap.add_argument("--runs", type=int, default=5)
    args = ap.parse_args()

    selected = [b for b in BENCHES if not args.only or b[0] in args.only]
    if not selected:
        sys.exit(f"no benchmarks matched {args.only}")

    ratios = []
    print("| bench | input | cpl (s) | hand C++ (s) | ratio | emitted (B) |")
    print("|---|---|---:|---:|---:|---:|")
    with tempfile.TemporaryDirectory() as tmp:
        for name, label, data in selected:
            cp_src = os.path.join(HERE, f"{name}.cp")
            hand_src = os.path.join(HERE, f"{name}_hand.cpp")
            emit_cpp = os.path.join(tmp, f"{name}.cpp")
            cp_bin = os.path.join(tmp, f"{name}_cp")
            hand_bin = os.path.join(tmp, f"{name}_hand")

            subprocess.run([CPL, "build", cp_src, "-o", emit_cpp], check=True, capture_output=True)
            emitted_bytes = os.path.getsize(emit_cpp)
            subprocess.run(["g++", "-std=c++17", "-O2", "-o", cp_bin, emit_cpp], check=True)
            subprocess.run(["g++", "-std=c++17", "-O2", "-o", hand_bin, hand_src], check=True)

            t_cp, out_cp = run_timed(cp_bin, data, args.runs)
            t_hand, out_hand = run_timed(hand_bin, data, args.runs)
            if out_cp != out_hand:
                raise RuntimeError(f"{name}: output mismatch: cp={out_cp!r} vs hand={out_hand!r}")
            ratio = t_cp / t_hand
            ratios.append(ratio)
            print(f"| {name} | {label} | {t_cp:.3f} | {t_hand:.3f} | {ratio:.2f}x | {emitted_bytes:,} |")

    if ratios:
        p50 = statistics.median(ratios)
        mx = max(ratios)
        print(f"\np50 = {p50:.2f}x (target <=1.10x), max = {mx:.2f}x (target <=1.50x)")


if __name__ == "__main__":
    main()
