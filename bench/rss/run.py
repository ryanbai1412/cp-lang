#!/usr/bin/env python3
"""Peak RSS of I/O-bound, zero-filled, sorting and buffer-reusing programs: each bench/rss/X.cp, built by `cpl build`,
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
# Cases the whole program of is gated nearer its hand C++: a small peak hides
# a regression of a megabyte under BOUND_KB (2587e0258 took cf2268b 4148 -> 5120).
TIGHT_KB = {"cf2268b": 512}


def ints(n, per=1):
    """`n`, then n * `per` numbers in 0..=1e9, about 9.5 bytes each."""
    body = " ".join(str(i * 7919 % 1_000_000_007 % 1_000_000_001) for i in range(n * per))
    return f"{n}\n{body}\n"


def aho_words(n, m):
    """`n` distinct words of 8-10 letters from "abc", then `m` 500-letter texts."""
    import random
    r, seen = random.Random(1), set()
    while len(seen) < n:
        seen.add("".join(r.choice("abc") for _ in range(r.randint(8, 10))))
    texts = ["".join(r.choice("abc") for _ in range(500)) for _ in range(m)]
    return f"{n} {m}\n" + "\n".join(sorted(seen)) + "\n" + "\n".join(texts) + "\n"


def words20(n):
    """`n`, then n lines of 20 letters each."""
    lines = ("".join(chr(97 + (i * 7919 + j * 31) % 26) for j in range(20)) for i in range(n))
    return f"{n}\n" + "\n".join(lines) + "\n"


def edges(n):
    """`n`, then n lines `u v w` with u, v in 1..=n and w in 1..=1000."""
    lines = (f"{i * 7919 % n + 1} {i * 104729 % n + 1} {i % 1000 + 1}" for i in range(n))
    return f"{n}\n" + "\n".join(lines) + "\n"


def table(r, c):
    """`r c`, then an r x c table of numbers in 0..=500."""
    rows = (" ".join(str((i * c + j) * 7919 % 501) for j in range(c)) for i in range(r))
    return f"{r} {c}\n" + "\n".join(rows) + "\n"


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
    "hash_array_param": lambda: "1000000\n",  # 4 ranged maps in an array, filled through `mut`
    "sieve": lambda: "10000000\n",  # primality only: no smallest-factor table
    "enum_grid": lambda: "4000\n",  # 16e6 cells: 16 MB at a byte, 64 MB at four
    "max_k": lambda: ints(500_000),  # the 30 largest of 5e5: O(k) beside the array (cf2165c)
    "print_narrow": lambda: "4000000\n",  # a List<0..=536_870_911> printed whole: 16 MB at 4 B (cf2201b)
    "edge_records": lambda: edges(3_000_000),  # 3e6 read (u, v, w) at 12 B, not 24 (coci21c4p4)
    "hill_records": lambda: edges(1_000_000),  # (u: V, v: V, d) at 12 B and arcs at 8 B (aac5p3, noi19p1)
    "runtime_bound": lambda: "4000000\n",  # List<1..=n> and [0..n]0..=n at 4 B (dmopc21c5p4, cf2187f1)
    "read_table": lambda: table(3000, 3000),  # [0..r, 0..c]0..=500 = read(): 18 MB at 2 B (apio09p1)
    "merge_sorted": lambda: f"1000000\n{' '.join(str(3 * i) for i in range(10**6))}\n{' '.join(str(3 * i + 1) for i in range(10**6))}\n",  # two sorted 1e6 arrays walked merged: no third list (cf2262c)
    # sort() 4e5 (Int, Bool, Int) events, noi19p1's sweep
    "sort_tuples": lambda: "200000\n" + "".join(f"{i * 7919 % 1001} {i * 104729 % 1001}\n" for i in range(200_000)),
    "sort_boxes": lambda: "20\n",  # sort() 2^20 (l: Int, r: Int) records, btoi08p6's boxes
    "cf2124c": lambda: gen("cf2124c"),  # b.adjacent over 6e5 [0..n]1..=1e9: 4 B cells, not 8
    "ucrpc21f": lambda: gen("ucrpc21f"),  # dijkstra over 1e6 cells read by p[v]: no settling-order list
    "release_walked": lambda: words20(400_000),  # words freed once a comprehension last walks them
    "year2018p7": lambda: gen("year2018p7"),  # 2e4 RollingHash of short words: two pair lists each, not four
    "aho_corasick": lambda: aho_words(50_000, 1000),  # 56k states over 3 used letters of 26
    "row_lists": lambda: "50000 200\n",  # [Row]List<Int> made anew 200 times, as dmpg16g2 does
    "sort_neg": lambda: ints(2_000_000),  # sort(by: i => (-a[i], i)) and (desc(a[i]), i), 2e6 positions
    "sort_distinct": lambda: ints(1_000_000),  # sorted(I, by: i => a[i] % 1000): ties kept by position
    "heap_grow": lambda: "2000000\n",  # two lists pushed to 2e6 in a program that sorts (ioi08p1)
    "cf2268b": lambda: gen("cf2268b"),  # 2e5 point updates into a pushed answer list: within 512 KiB
    "sort_comp": lambda: ints(2_000_000),  # sorted([(a[i], i) for i in 0..n]): 8 B pairs, reserved (cf2187a)
    "ucup5s02e": lambda: gen("ucup5s02e"),  # rank.pairs() over three 2e5 rows, then a row's inverse
}


def peak(exe, inp, out, tmp):
    """(peak RSS in KiB, sha256 of stdout) of `exe` on the file `inp`."""
    rss = os.path.join(tmp, "rss")
    with open(inp, "rb") as i, open(out, "wb") as o:
        subprocess.run([launcher(), rss, exe], stdin=i, stdout=o, check=True)
    with open(out, "rb") as f:
        digest = hashlib.file_digest(f, "sha256").hexdigest()
    with open(rss) as f:
        return int(f.read().split()[0]), digest  # the launcher writes `kb us`


def verdict(rows):
    """Messages for `rows` (case, cpl KiB, hand KiB, same output) and whether --check fails."""
    bad = []
    for case, ours, hand, same in rows:
        if not same:
            bad.append(f"rss: {case} prints something else than its hand C++")
        elif ours > hand + TIGHT_KB.get(case, BOUND_KB):
            bad.append(f"rss: {case} peaks at {ours} KiB, over {hand} KiB + {TIGHT_KB.get(case, BOUND_KB)} for its hand C++")
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
