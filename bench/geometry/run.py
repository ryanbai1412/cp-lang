#!/usr/bin/env python3
"""Geometry stdlib benchmarks: each X.cp, built by `cpl build`, against the
matched hand-written X_hand.cpp, on the same generated input. Both are
compiled with g++ -std=c++17 -O2. Prints a Markdown table; writes nothing
tracked.

    cargo build --release -p cpl-cli
    python3 bench/geometry/run.py               # every bench, best of 5
    python3 bench/geometry/run.py convex_hull --runs 3
"""
import argparse, math, os, random, subprocess, sys, tempfile, time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
CPL = os.environ.get("CPL", os.path.join(ROOT, "target", "release", "cpl"))


def gen_points(n, seed):
    rng = random.Random(seed)
    rows = [str(n)]
    for _ in range(n):
        rows.append(f"{rng.randint(-10**8, 10**8)} {rng.randint(-10**8, 10**8)}")
    return "\n".join(rows) + "\n"


def gen_half_planes(n, seed):
    rng = random.Random(seed)
    rows = [str(n + 4)]
    # Bounding box [-10000, 10000]^2
    rows.append("-10000 -10000 1 0")
    rows.append("10000 -10000 0 1")
    rows.append("10000 10000 -1 0")
    rows.append("-10000 10000 0 -1")
    for _ in range(n):
        ang = rng.uniform(0.0, 2.0 * math.pi)
        r = rng.uniform(500.0, 2000.0)
        px = r * math.cos(ang)
        py = r * math.sin(ang)
        dx = -math.sin(ang)
        dy = math.cos(ang)
        rows.append(f"{px:.6f} {py:.6f} {dx:.6f} {dy:.6f}")
    return "\n".join(rows) + "\n"


def gen_segments(n, seed):
    rng = random.Random(seed)
    # Disjoint slanted segments arranged in horizontal bands so Shamos-Hoey runs all 2n events.
    rows = [str(n)]
    for i in range(n):
        y0 = i * 10
        x0 = rng.randint(0, 10**7)
        x1 = x0 + rng.randint(100, 10**6)
        y1 = y0 + rng.randint(-3, 3)
        rows.append(f"{x0} {y0} {x1} {y1}")
    return "\n".join(rows) + "\n"


def gen_points_3d(n, seed):
    rng = random.Random(seed)
    rows = [str(n)]
    for _ in range(n):
        rows.append(f"{rng.randint(-10**5, 10**5)} {rng.randint(-10**5, 10**5)} {rng.randint(-10**5, 10**5)}")
    return "\n".join(rows) + "\n"


BENCHES = {
    "convex_hull": lambda: gen_points(200_000, 1),
    "half_plane_intersect": lambda: gen_half_planes(100_000, 2),
    "closest_pair": lambda: gen_points(100_000, 3),
    "any_segments_meet": lambda: gen_segments(100_000, 4),
    "convex_hull_3d": lambda: gen_points_3d(20_000, 5),
}


def best(exe, inp, runs):
    times, out = [], None
    for _ in range(runs):
        with open(inp) as f:
            t = time.perf_counter()
            p = subprocess.run([exe], stdin=f, capture_output=True, text=True, check=True)
            times.append(time.perf_counter() - t)
        out = p.stdout.strip()
    return min(times), out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("benches", nargs="*", default=list(BENCHES))
    ap.add_argument("--runs", type=int, default=5)
    args = ap.parse_args()
    print("| bench | cpl (s) | hand C++ (s) | ratio | same output |")
    print("|---|---|---|---|---|")
    with tempfile.TemporaryDirectory() as tmp:
        for b in args.benches:
            inp = os.path.join(tmp, b + ".in")
            with open(inp, "w") as f:
                f.write(BENCHES[b]())
            ours_cpp = os.path.join(tmp, b + ".cpp")
            subprocess.run([CPL, "build", os.path.join(HERE, b + ".cp"), "-o", ours_cpp],
                           check=True, capture_output=True)
            exes = []
            for src, name in ((ours_cpp, "ours"), (os.path.join(HERE, b + "_hand.cpp"), "hand")):
                exe = os.path.join(tmp, f"{b}_{name}")
                subprocess.run(["g++", "-std=c++17", "-O2", "-o", exe, src], check=True)
                exes.append(exe)
            (ta, oa), (tb, ob) = best(exes[0], inp, args.runs), best(exes[1], inp, args.runs)
            same = "yes" if oa == ob else f"NO ({oa} vs {ob})"
            print(f"| {b} | {ta:.3f} | {tb:.3f} | {ta / tb:.2f}x | {same} |", flush=True)


if __name__ == "__main__":
    sys.exit(main())
