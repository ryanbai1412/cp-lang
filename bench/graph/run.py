#!/usr/bin/env python3
"""Graph stdlib benchmarks: each X.cp, built by `cpl build`, against the
matched hand-written X_hand.cpp, on the same generated input. Both are
compiled with g++ -std=c++17 -O2. Prints a Markdown table; writes nothing
tracked.

    cargo build --release -p cpl-cli
    python3 bench/graph/run.py               # every bench, best of 5
    python3 bench/graph/run.py dijkstra --runs 3
"""
import argparse, os, random, subprocess, sys, tempfile, time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
CPL = os.environ.get("CPL", os.path.join(ROOT, "target", "release", "cpl"))


def graph(n, m, weight, seed):
    rng = random.Random(seed)
    rows = [f"{n} {m}"]
    for i in range(m):
        u = i % n + 1 if i < n - 1 else rng.randint(1, n)
        v = u + 1 if i < n - 1 else rng.randint(1, n)
        rows.append(f"{u} {v} {weight(rng)}" if weight else f"{u} {v}")
    return "\n".join(rows) + "\n"


def fixed_cost(n, seed):
    rng = random.Random(seed)
    k = 6
    rows = [f"{n} {k}"]
    for _ in range(k):
        rows.append(f"{rng.randint(2, n - 1)} {rng.randint(0, n - 1)} {rng.randint(1, 10**6)}")
    return "\n".join(rows) + "\n"


BENCHES = {
    "bfs": lambda: graph(10**6, 2 * 10**6, None, 1),
    "bfs01": lambda: graph(10**6, 2 * 10**6, lambda r: r.randint(0, 1), 2),
    "dijkstra": lambda: graph(5 * 10**5, 2 * 10**6, lambda r: r.randint(1, 10**9), 3),
    "fixed_cost": lambda: fixed_cost(2 * 10**6, 4),
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
                subprocess.run(["g++", "-std=c++17", "-O2", "-I", HERE, "-o", exe, src], check=True)
                exes.append(exe)
            (ta, oa), (tb, ob) = best(exes[0], inp, args.runs), best(exes[1], inp, args.runs)
            same = "yes" if oa == ob else f"NO ({oa} vs {ob})"
            print(f"| {b} | {ta:.3f} | {tb:.3f} | {ta / tb:.2f}x | {same} |", flush=True)


if __name__ == "__main__":
    sys.exit(main())
