#!/usr/bin/env python3
"""Input-reading benchmarks: each X.cp, built by `cpl build`, against the
matched hand-written X_hand.cpp on the same generated input, both with
g++ -std=c++17 -O2. Prints a Markdown table; writes nothing tracked.

    cargo build --release -p cpl-cli
    python3 bench/io/run.py               # every bench, best of 7

A bench with an X_judge.cpp is interactive: the program and the judge talk
through pipes, and the output compared is what the judge writes to stderr.
"""
import argparse, os, random, subprocess, sys, tempfile, time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
CPL = os.environ.get("CPL", os.path.join(ROOT, "target", "release", "cpl"))


def ints(n, seed):
    rng = random.Random(seed)
    return f"{n}\n" + " ".join(str(rng.randint(-10**9, 10**9)) for _ in range(n)) + "\n"


def positive(n, seed):
    rng = random.Random(seed)
    return f"{n}\n" + " ".join(str(rng.randint(1, 10**6)) for _ in range(n)) + "\n"


BENCHES = {
    "read_ints": lambda: ints(6 * 10**6, 1),
    "read_range": lambda: positive(6 * 10**6, 2),
    "ask_loop": lambda: "",
    "print_ints": lambda: "4000000\n",
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


def best_judged(exe, judge, runs):
    times, out = [], None
    for _ in range(runs):
        to_judge, from_prog = os.pipe()
        to_prog, from_judge = os.pipe()
        t = time.perf_counter()
        j = subprocess.Popen([judge], stdin=to_judge, stdout=from_judge, stderr=subprocess.PIPE)
        p = subprocess.Popen([exe], stdin=to_prog, stdout=from_prog)
        for fd in (to_judge, from_prog, to_prog, from_judge):
            os.close(fd)
        p.wait()
        err = j.communicate()[1]
        times.append(time.perf_counter() - t)
        if p.returncode or j.returncode:
            sys.exit(f"{exe} against {judge}: exit {p.returncode} / {j.returncode}")
        out = err.decode().strip()
    return min(times), out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("benches", nargs="*", default=list(BENCHES))
    ap.add_argument("--runs", type=int, default=7)
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
            judge_src = os.path.join(HERE, b + "_judge.cpp")
            if os.path.exists(judge_src):
                judge = os.path.join(tmp, b + "_judge")
                subprocess.run(["g++", "-std=c++17", "-O2", "-o", judge, judge_src], check=True)
                (ta, oa), (tb, ob) = (best_judged(e, judge, args.runs) for e in exes)
            else:
                (ta, oa), (tb, ob) = best(exes[0], inp, args.runs), best(exes[1], inp, args.runs)
            same = "yes" if oa == ob else f"NO ({oa} vs {ob})"
            print(f"| {b} | {ta:.3f} | {tb:.3f} | {ta / tb:.2f}x | {same} |", flush=True)


if __name__ == "__main__":
    sys.exit(main())
