#!/usr/bin/env python3
"""Peak RSS of I/O-bound, zero-filled and sorting programs: each bench/rss/X.cp, built by `cpl build`,
against the matched X_hand.cpp (scanf/printf) on the same generated input,
both with g++ -std=c++17 -O2, measured by scripts/corpus.py's launcher.

    python3 bench/rss/run.py [--cpl PATH] [--check] [CASE...]

Prints a Markdown table. --check fails when a program's output differs from
its hand C++ or its peak RSS is over `BOUND_KB` above the hand C++'s: input
and output are streamed through one fixed chunk each, so the gap may not
grow with the input or the line.
"""
import argparse, hashlib, os, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(ROOT, "scripts"))
from corpus import launcher  # noqa: E402

# How far above the hand C++ a program's peak may be, in KiB.
BOUND_KB = 2048


def ints(n, per=1):
    """`n`, then n * `per` numbers in 0..=1e9, about 9.5 bytes each."""
    body = " ".join(str(i * 7919 % 1_000_000_007 % 1_000_000_001) for i in range(n * per))
    return f"{n}\n{body}\n"


def gen(problem):
    """corpus/problems/`problem`/gen.py --max's input."""
    script = os.path.join(ROOT, "corpus", "problems", problem, "gen.py")
    return subprocess.run([sys.executable, script, "--max"], check=True,
                          capture_output=True, text=True).stdout


CASES = {
    "read_ints": lambda: ints(8_000_000),  # about 76 MB
    "read_tree": lambda: gen("aac5p3"),  # about 6.4 MB
    "print_range": lambda: "10000000\n",
    "print_gen": lambda: "10000000\n",
    "print_list_sep": lambda: "10000000\n",
    "floor": lambda: "5\n",
    "zero_fill": lambda: "5\n",  # 160 MB of cells, one touched
    "sort_records": lambda: ints(1_000_000, 2),  # sort(by:) a field of 1e6 (Int, Int)
    "sort_pairs": lambda: ints(400_000, 2),  # sort() 4e5 (Int, Int)
    "hash_counter": lambda: "3375000\n",
    "hash_str_grow": lambda: "300000\n",
    "sort_index": lambda: ints(2_000_000),  # sort(by: i => (a[i], i)) 2e6 positions
    "filter_comp": lambda: ints(4_000_000),  # [.. for x in a if ..] keeping half, then 1 in 1000
    "sort_field": lambda: "",  # sort(by: e => es[e].w) 250k edges, as std kruskal does
    "hash_counter_narrow": lambda: "3375000\n",
    "sieve": lambda: "10000000\n",  # primality only: no smallest-factor table
    "enum_grid": lambda: "4000\n",  # 16e6 cells: 16 MB at a byte, 64 MB at four
    "max_k": lambda: ints(500_000),  # the 30 largest of 5e5: O(k) beside the array (cf2165c)
}


def peak(exe, inp, out, tmp):
    """(peak RSS in KiB, sha256 of stdout) of `exe` on the file `inp`."""
    rss = os.path.join(tmp, "rss")
    with open(inp, "rb") as i, open(out, "wb") as o:
        subprocess.run([launcher(), rss, exe], stdin=i, stdout=o, check=True)
    with open(out, "rb") as f:
        digest = hashlib.file_digest(f, "sha256").hexdigest()
    with open(rss) as f:
        return int(f.read()), digest


def verdict(rows):
    """Messages for `rows` (case, cpl KiB, hand KiB, same output) and whether --check fails."""
    bad = []
    for case, ours, hand, same in rows:
        if not same:
            bad.append(f"rss: {case} prints something else than its hand C++")
        elif ours > hand + BOUND_KB:
            bad.append(f"rss: {case} peaks at {ours} KiB, over {hand} KiB + {BOUND_KB} for its hand C++")
    return bad, bool(bad)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cases", nargs="*", default=list(CASES))
    ap.add_argument("--cpl", default=os.environ.get("CPL", os.path.join(ROOT, "target", "release", "cpl")))
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    rows = []
    print("| case | cpl (KiB) | hand C++ (KiB) | same output |")
    print("|---|---|---|---|")
    with tempfile.TemporaryDirectory() as tmp:
        for case in args.cases:
            inp = os.path.join(tmp, case + ".in")
            with open(inp, "w") as f:
                f.write(CASES[case]())
            ours_cpp = os.path.join(tmp, case + ".cpp")
            subprocess.run([args.cpl, "build", os.path.join(HERE, case + ".cp"), "-o", ours_cpp],
                           check=True, capture_output=True)
            got = []
            for src, name in ((ours_cpp, "ours"), (os.path.join(HERE, case + "_hand.cpp"), "hand")):
                exe = os.path.join(tmp, f"{case}_{name}")
                subprocess.run(["g++", "-std=c++17", "-O2", "-o", exe, src], check=True)
                got.append(peak(exe, inp, os.path.join(tmp, name + ".out"), tmp))
            (ours, a), (hand, b) = got
            rows.append((case, ours, hand, a == b))
            print(f"| {case} | {ours} | {hand} | {'yes' if a == b else 'NO'} |", flush=True)
            os.remove(inp)
    messages, failed = verdict(rows)
    for m in messages:
        print(m, file=sys.stderr)
    return 1 if args.check and failed else 0


if __name__ == "__main__":
    sys.exit(main())
