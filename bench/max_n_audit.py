#!/usr/bin/env python3
"""Stage-6 Max-N Judge & Constant-Factor Audit harness for CP Lang.

Compiles 30 diverse ports from proto/ports/ (plus their corpus originals) with
judge flags (`g++ -std=c++17 -O2`, no `-DLOCAL`), generates realistic worst-case
max-N contest inputs, verifies token-for-token output equality, and measures:
  1. Median wall-clock runtime (CP Lang emitted C++17 vs original C++).
  2. Peak RSS in KB (CP Lang emitted C++17 vs original C++).
  3. Emitted C++17 file size in bytes (vs the 64 KB judge ceiling).

Also supports targeted adversarial probes (`--adversarial`):
  - Anti-hash collision attack against `CpHash` (unseeded splitmix64).
  - Stack-depth probe for `@memo` recursion on a 10^5 chain tree.
  - `CpView`/`lends` bounds-check microbenchmark at N=10^7.

Never mutates tracked files in `proto/ports/`: every `.cp` is copied to a
temporary directory before running `proto/cp.py build`.
"""

import argparse
import json
import os
import random
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from typing import Callable, List, Optional

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PORTS_DIR = os.path.join(REPO_ROOT, "proto", "ports")
CP_PY = os.path.join(REPO_ROOT, "proto", "cp.py")


@dataclass
class PortSpec:
    name: str          # relative path under proto/ports without .cp
    category: str      # DP, Graph, Tree, Range, String, Window/Hash, Grid
    features: str      # CP Lang abstractions exercised
    limits: str        # human-readable max-N input description
    gen: Callable[[str], None]  # writes input to given file path


# ---------------------------------------------------------------------------
# Realistic worst-case max-N generators (deterministic seed per problem)
# ---------------------------------------------------------------------------

def gen_dpo(path: str) -> None:
    """AtCoder DP O: N=21, dense compatibility matrix (all 1s -> all 2^21 states active)."""
    n = 21
    row = " ".join(["1"] * n)
    with open(path, "w") as f:
        f.write(f"{n}\n")
        for _ in range(n):
            f.write(row + "\n")


def gen_dpp(path: str) -> None:
    """AtCoder DP P: N=100,000, deep broom/caterpillar tree (depth ~50,000 + bushy tail)."""
    rng = random.Random(102)
    n = 100_000
    lines = [str(n)]
    # 40k chain + 60k attached randomly to stress recursion depth and branching
    for i in range(2, 40_001):
        lines.append(f"{i - 1} {i}")
    for i in range(40_001, n + 1):
        p = rng.randint(1, i - 1)
        lines.append(f"{p} {i}")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_dps(path: str) -> None:
    """AtCoder DP S: |K|=10,000 digits, D=100."""
    rng = random.Random(103)
    k = "9" + "".join(str(rng.randint(0, 9)) for _ in range(9999))
    with open(path, "w") as f:
        f.write(f"{k}\n100\n")


def gen_dpv(path: str) -> None:
    """AtCoder DP V: N=100,000, non-prime M=10^9, high-degree + deep tree."""
    rng = random.Random(104)
    n = 100_000
    m = 1_000_000_000
    lines = [f"{n} {m}"]
    # Mix of a 20k-degree star, a 20k chain, and 60k random tree nodes
    for i in range(2, 20_002):
        lines.append(f"1 {i}")
    for i in range(20_002, 40_002):
        lines.append(f"{i - 1} {i}")
    for i in range(40_002, n + 1):
        lines.append(f"{rng.randint(1, i - 1)} {i}")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_btoi18p6(path: str) -> None:
    """BOI 2018 Paths: N=300,000, M=300,000, K=5."""
    rng = random.Random(105)
    n, m, k = 300_000, 300_000, 5
    cols = [str(rng.randint(1, k)) for _ in range(n)]
    lines = [f"{n} {m} {k}", " ".join(cols)]
    # Spanning chain + random cross-colour edges
    for i in range(1, n):
        lines.append(f"{i} {i + 1}")
    for _ in range(m - (n - 1)):
        u = rng.randint(1, n)
        v = rng.randint(1, n - 1)
        if v >= u:
            v += 1
        lines.append(f"{u} {v}")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_ceoi18p1(path: str) -> None:
    """CEOI 2018 Cloud Computing: N=2000 machines, M=2000 orders, max 50 cores -> 10^5 DP states."""
    rng = random.Random(106)
    n, m = 2000, 2000
    lines = [str(n)]
    for _ in range(n):
        c = rng.randint(25, 50)
        f_rate = rng.randint(1, 10**9)
        p = rng.randint(1, 10**9)
        lines.append(f"{c} {f_rate} {p}")
    lines.append(str(m))
    for _ in range(m):
        c = rng.randint(25, 50)
        f_rate = rng.randint(1, 10**9)
        p = rng.randint(1, 10**9)
        lines.append(f"{c} {f_rate} {p}")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_ioi04p4(path: str) -> None:
    """IOI 2004 Phidias: W=600, H=600, N=200 plates (large plates so full DP table is explored)."""
    rng = random.Random(107)
    w, h, n = 600, 600, 200
    lines = [f"{w} {h}", str(n)]
    seen = set()
    while len(seen) < n:
        pw = rng.randint(100, w)
        ph = rng.randint(100, h)
        if (pw, ph) not in seen:
            seen.add((pw, ph))
            lines.append(f"{pw} {ph}")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_cco18p1(path: str) -> None:
    """CCO 2018 P1 Geese vs Hawks: N=1000 games."""
    rng = random.Random(108)
    n = 1000
    sa = "".join(rng.choice("WL") for _ in range(n))
    pa = " ".join(str(rng.randint(1, 10**6)) for _ in range(n))
    sb = "".join(rng.choice("WL") for _ in range(n))
    pb = " ".join(str(rng.randint(1, 10**6)) for _ in range(n))
    with open(path, "w") as f:
        f.write(f"{n}\n{sa}\n{pa}\n{sb}\n{pb}\n")


def gen_cco24p1(path: str) -> None:
    """CCO 2024 Treasure Hunt: N=500,000, M=1,000,000 sparse weighted graph."""
    rng = random.Random(109)
    n, m = 500_000, 1_000_000
    a = [str(rng.randint(1, 10**9)) for _ in range(n)]
    lines = [f"{n} {m}", " ".join(a)]
    # Connected backbone + random chords with moderate tolls to trigger many relaxations
    for i in range(1, n):
        lines.append(f"{i} {i + 1} {rng.randint(1, 10**5)}")
    for _ in range(m - (n - 1)):
        u = rng.randint(1, n)
        v = rng.randint(1, n - 1)
        if v >= u:
            v += 1
        lines.append(f"{u} {v} {rng.randint(1, 10**6)}")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_ioi11p4(path: str) -> None:
    """IOI 2011 Crocodile: N=100,000, M=1,000,000, K=100 exits."""
    rng = random.Random(110)
    n, m, k = 100_000, 1_000_000, 100
    lines = [f"{n} {m} {k}"]
    # 2-edge-connected ring + random edges so every node has degree >= 2
    for i in range(n):
        lines.append(f"{i} {(i + 1) % n} {rng.randint(1, 10**6)}")
    for i in range(n):
        lines.append(f"{i} {(i + 2) % n} {rng.randint(1, 10**6)}")
    for _ in range(m - 2 * n):
        u = rng.randint(0, n - 1)
        v = rng.randint(0, n - 2)
        if v >= u:
            v += 1
        lines.append(f"{u} {v} {rng.randint(1, 10**7)}")
    exits = [str(n - 1 - i) for i in range(k)]
    lines.append(" ".join(exits))
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_apio09p3(path: str) -> None:
    """APIO 2009 ATM: N=500,000, M=500,000 directed graph with cycles + long DAG."""
    rng = random.Random(111)
    n, m = 500_000, 500_000
    lines = [f"{n} {m}"]
    # Chain 1->2->...->n (499,999 edges) + 1 back-edge per block of 500 -> ~1000 SCCs
    edges = []
    for i in range(1, n):
        edges.append(f"{i} {i + 1}")
    # Overwrite? No, m=500,000 gives 1 extra edge; make m=500,000 with 400k chain + 100k cycle/cross edges
    edges = []
    # Build 1000 cycles of 400 nodes + chain links across cycles + remaining random edges
    for blk in range(1000):
        base = blk * 400 + 1
        for j in range(399):
            edges.append(f"{base + j} {base + j + 1}")
        edges.append(f"{base + 399} {base}")  # closes 400-node SCC
        if blk + 1 < 1000:
            edges.append(f"{base + 200} {base + 400}")
    # Connect remaining 100,000 nodes as a DAG tail
    for i in range(400_000, n):
        edges.append(f"{i} {i + 1}")
    while len(edges) < m:
        u = rng.randint(1, n - 1)
        v = rng.randint(u + 1, min(n, u + 5000))
        edges.append(f"{u} {v}")
    edges = edges[:m]
    lines.extend(edges)
    cash = [str(rng.randint(0, 4000)) for _ in range(n)]
    lines.append("\n".join(cash))
    p = 10_000
    lines.append(f"1 {p}")
    pubs = [str(n - i) for i in range(p)]
    lines.append(" ".join(pubs))
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_ccc18s5(path: str) -> None:
    """CCC 2018 S5: N=100,000, M=100,000, P=100,000, Q=100,000."""
    rng = random.Random(112)
    n, m, p, q = 100_000, 100_000, 100_000, 100_000
    lines = [f"{n} {m} {p} {q}"]
    # Connected flights (tree + 1)
    for i in range(1, m):
        lines.append(f"{i} {i + 1} {rng.randint(1, 10**8)}")
    lines.append(f"1 {m} {rng.randint(1, 10**8)}")
    # Connected portals (tree + 1)
    for i in range(1, n):
        lines.append(f"{i} {i + 1} {rng.randint(1, 10**8)}")
    lines.append(f"1 {n} {rng.randint(1, 10**8)}")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_ccc21s4(path: str) -> None:
    """CCC 2021 S4 Daily Commute: N=200,000, W=200,000, D=200,000."""
    rng = random.Random(113)
    n, w, d = 200_000, 200_000, 200_000
    lines = [f"{n} {w} {d}"]
    for _ in range(w):
        u = rng.randint(1, n)
        v = rng.randint(1, n)
        lines.append(f"{u} {v}")
    perm = list(range(2, n + 1))
    rng.shuffle(perm)
    perm = [1] + perm
    lines.append(" ".join(map(str, perm)))
    for _ in range(d):
        x = rng.randint(1, n)
        y = rng.randint(1, n - 1)
        if y >= x:
            y += 1
        lines.append(f"{x} {y}")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_noip09p3(path: str) -> None:
    """NOIP 2009 Optimal Trade: N=100,000, M=500,000."""
    rng = random.Random(114)
    n, m = 100_000, 500_000
    prices = [str(rng.randint(1, 100)) for _ in range(n)]
    lines = [f"{n} {m}", " ".join(prices)]
    for i in range(1, n):
        lines.append(f"{i} {i + 1} {rng.choice([1, 2])}")
    for _ in range(m - (n - 1)):
        u = rng.randint(1, n)
        v = rng.randint(1, n - 1)
        if v >= u:
            v += 1
        lines.append(f"{u} {v} {rng.choice([1, 2])}")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_btoi17p3(path: str) -> None:
    """BOI 2017 Railway: N=100,000, M=50,000, K=2, sum of plan sizes = 200,000."""
    rng = random.Random(115)
    n, m, k = 100_000, 50_000, 2
    lines = [f"{n} {m} {k}"]
    # Balanced + deep tree
    for i in range(2, n + 1):
        p = i - 1 if i <= 30_000 else rng.randint(1, i - 1)
        lines.append(f"{p} {i}")
    for _ in range(m):
        sz = 4
        pts = rng.sample(range(1, n + 1), sz)
        lines.append(f"{sz} " + " ".join(map(str, pts)))
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_noi15p2(path: str) -> None:
    """NOI 2015 Package Manager: N=100,000, Q=100,000 (binary tree -> max HLD chain hops)."""
    rng = random.Random(116)
    n, q = 100_000, 100_000
    # Complete binary tree maximizes HLD light-edge crossings (log2 N per install!)
    parents = [str((i - 1) // 2) for i in range(1, n)]
    lines = [str(n), " ".join(parents), str(q)]
    for i in range(q):
        if i % 3 != 2:
            # Install deep leaf to maximize chain hops
            u = rng.randint(n // 2, n - 1)
            lines.append(f"install {u}")
        else:
            u = rng.randint(0, n // 4)
            lines.append(f"uninstall {u}")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_ioi11p2(path: str) -> None:
    """IOI 2011 Race: N=200,000, K=1,000,000, complete binary tree (max small-to-large merges)."""
    rng = random.Random(117)
    n, k = 200_000, 1_000_000
    lines = [f"{n} {k}"]
    # Complete binary tree forces O(N log N) hash-map inserts/lookups!
    for i in range(1, n):
        p = (i - 1) // 2
        w = rng.randint(1, 50_000)
        lines.append(f"{p} {i} {w}")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_coi06p2(path: str) -> None:
    """COI 2006 Policija: N=100,000, M=500,000, Q=300,000."""
    rng = random.Random(118)
    n, m, q = 100_000, 500_000, 300_000
    lines = [f"{n} {m}"]
    edges = []
    # Tree backbone + extra edges creating biconnected blocks and bridges
    for i in range(2, n + 1):
        p = i - 1 if i % 1000 != 0 else rng.randint(1, i - 1)
        edges.append((p, i))
        lines.append(f"{p} {i}")
    for _ in range(m - (n - 1)):
        # Keep edges inside blocks of 1000 so there are real bridges AND cycles
        blk = rng.randint(0, 99)
        lo = blk * 1000 + 1
        hi = min(n, lo + 999)
        u = rng.randint(lo, hi)
        v = rng.randint(lo, hi - 1)
        if v >= u:
            v += 1
        edges.append((u, v))
        lines.append(f"{u} {v}")
    lines.append(str(q))
    for i in range(q):
        a = rng.randint(1, n)
        b = rng.randint(1, n - 1)
        if b >= a:
            b += 1
        if i % 2 == 0:
            u, v = edges[rng.randint(0, len(edges) - 1)]
            lines.append(f"1 {a} {b} {u} {v}")
        else:
            c = rng.randint(1, n)
            while c == a or c == b:
                c = rng.randint(1, n)
            lines.append(f"2 {a} {b} {c}")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_sac22cc3p4(path: str) -> None:
    """SAC '22 CC3 P4: N=200,000, Q=200,000."""
    rng = random.Random(119)
    n, q = 200_000, 200_000
    lines = [f"{n} {q}"]
    edges = []
    for i in range(2, n + 1):
        p = rng.randint(1, i - 1)
        edges.append((p, i))
        lines.append(f"{p} {i}")
    rng.shuffle(edges)
    # Interleave 100k cuts (type 2) and 100k distance queries (type 1)
    cut_idx = 0
    for i in range(q):
        if i % 2 == 0 and cut_idx < len(edges):
            u, v = edges[cut_idx]
            cut_idx += 1
            lines.append(f"2 {u} {v}")
        else:
            u = rng.randint(1, n)
            v = rng.randint(1, n)
            lines.append(f"1 {u} {v}")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_ccc21s5(path: str) -> None:
    """CCC 2021 S5 Math Homework: N=150,000, Q=100,000 (satisfiable -> full segtree build + 100k queries)."""
    rng = random.Random(120)
    n, q = 150_000, 100_000
    # Construct satisfiable intervals with x in {1, 2, 3, 4, 6, 8, 12} on disjoint or consistent segments
    lines = [f"{n} {q}"]
    # Partition array into blocks of 15 with fixed base value, emit queries inside blocks + global gcd=1
    for i in range(q):
        if i < q // 2:
            blk = rng.randint(0, n // 15 - 1)
            l = blk * 15 + 1
            r = l + 14
            x = (blk % 16) + 1
            lines.append(f"{l} {r} {x}")
        else:
            l = rng.randint(1, n - 500)
            r = rng.randint(l + 200, n)
            lines.append(f"{l} {r} 1")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_noi16p4(path: str) -> None:
    """NOI 2016 Intervals: N=500,000, M=200,000."""
    rng = random.Random(121)
    n, m = 500_000, 200_000
    lines = [f"{n} {m}"]
    # Overlapping intervals with diverse lengths so two-pointer sweeps most of N
    for i in range(n):
        length = rng.randint(1, 500_000)
        l = rng.randint(0, 1_000_000)
        lines.append(f"{l} {l + length}")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_cco20p1(path: str) -> None:
    """CCO 2020 P1 A Game with Grundy: N=100,000 friends."""
    rng = random.Random(122)
    n = 100_000
    l, r, y = -10**9, 10**9, 10**9
    lines = [str(n), f"{l} {r} {y}"]
    for _ in range(n):
        x = rng.randint(l, r)
        v = rng.randint(1, 100)
        h = rng.randint(1, 100)
        lines.append(f"{x} {v} {h}")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_ccc20s3(path: str) -> None:
    """CCC 2020 S3 Searching for Strings: |N|=100,000, |H|=200,000 (every window is a permutation!)."""
    # Repeating pattern of length 26 rotated/shuffled so ALL 100,001 windows match counts
    # and hit StrHash::of + Set::insert!
    rng = random.Random(123)
    # Build needle of length 100,000 as 3846 copies of 'a'..'z' + 4 chars
    unit = "abcdefghijklmnopqrstuvwxyz"
    needle = "".join("".join(rng.sample(unit, 26)) for _ in range(3846)) + "abcd"
    # Make hay = needle repeated twice (length 200,000) with local swaps in each 26-block
    # Actually if hay = needle + needle, EVERY window of length 100,000 is an anagram of needle!
    hay = needle + needle
    with open(path, "w") as f:
        f.write(f"{needle}\n{hay}\n")


def gen_noi14p4(path: str) -> None:
    """NOI 2014 Zoo: T=5, |S|=1,000,000 each (worst-case periodic/Fibonacci-like strings)."""
    rng = random.Random(124)
    t = 5
    n = 1_000_000
    strings = [
        "a" * n,                                      # maximum border depth (10^6)
        ("ab" * (n // 2)),                            # period 2
        ("aabaa" * (n // 5)),                         # period 5
        ("".join("a" if (i & (i - 1)) else "b" for i in range(1, n + 1))),  # quasi-periodic
        ("".join(rng.choice("ab") for _ in range(n))),
    ]
    with open(path, "w") as f:
        f.write(f"{t}\n" + "\n".join(strings) + "\n")


def gen_coci20c6p3(path: str) -> None:
    """COCI 2020 Anagramistica: N=2000, K=2000 (20 anagram classes of 100 words each)."""
    rng = random.Random(125)
    n, k = 2000, 2000
    lines = [f"{n} {k}"]
    bases = ["".join(rng.choice("abcdefghij") for _ in range(10)) for _ in range(20)]
    for i in range(n):
        w = list(bases[i % 20])
        rng.shuffle(w)
        lines.append("".join(w))
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_ccc22s4(path: str) -> None:
    """CCC 2022 S4 Good Triplets: N=1,000,000, C=1,000,000 (even C triggers diameter pass too)."""
    rng = random.Random(126)
    n, c = 1_000_000, 1_000_000
    pts = [str(rng.randint(0, c - 1)) for _ in range(n)]
    with open(path, "w") as f:
        f.write(f"{n} {c}\n" + " ".join(pts) + "\n")


def gen_noi01p5(path: str) -> None:
    """NOI 2001 P5: N=6, M=150 (3.375M sums per half -> heavy hash table / list churn)."""
    # Coefficients chosen so sums are well-spread AND have many collisions with negated half
    with open(path, "w") as f:
        f.write("6\n150\n1 3\n-1 3\n2 3\n-2 3\n3 3\n-3 3\n")


def gen_ioi10p3(path: str) -> None:
    """IOI 2010 Quality of Living: R=3000, C=3000, H=1001, W=1001 (9M cells)."""
    rng = random.Random(128)
    r, c, h, w = 3000, 3000, 1001, 1001
    # Fast deterministic permutation using LCG coprime stride (much faster than shuffling 9M ints in Python)
    n = r * c
    # gcd(3_000_017, 9_000_000) == 1
    stride = 3_000_017
    with open(path, "w") as f:
        f.write(f"{r} {c} {h} {w}\n")
        val = 123456
        for _ in range(r):
            row = []
            for _ in range(c):
                val = (val + stride) % n
                row.append(str(val + 1))
            f.write(" ".join(row) + "\n")


def gen_dmopc21c9p4(path: str) -> None:
    """DMOPC '21 Contest 9 P4: R=1000, C=1000 (solvable grid -> full 10^6 coin removal)."""
    rng = random.Random(129)
    r, c = 1000, 1000
    # Ensure odd number of 'H' rows (with odd 'H' count per row) so grid is solvable and prints 10^6 lines!
    lines = [f"{r} {c}"]
    for i in range(r):
        # First row odd H count, rest alternating so row-parity line has odd H count (solvable)
        row = ["H" if rng.random() < 0.5 else "T" for _ in range(c)]
        h_cnt = row.count("H")
        want_odd = (i == 0)  # exactly 1 odd row -> 1D game on rows is guaranteed solvable!
        if (h_cnt % 2 == 1) != want_odd:
            row[0] = "T" if row[0] == "H" else "H"
        lines.append("".join(row))
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def gen_ccc21s2(path: str) -> None:
    """CCC 2021 S2 Modern Art: R=500,000, C=10 (R*C=5e6), K=1,000,000 strokes."""
    rng = random.Random(130)
    r, c, k = 500_000, 10, 1_000_000
    lines = [str(r), str(c), str(k)]
    for i in range(k):
        if i % 4 != 0:
            lines.append(f"R {rng.randint(1, r)}")
        else:
            lines.append(f"C {rng.randint(1, c)}")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


PORTS: List[PortSpec] = [
    # DP (8)
    PortSpec("hillclimb_dp_graph/dpo", "DP", "Subset, Mod<P>, 2D Bool", "N=21 (2^21 states)", gen_dpo),
    PortSpec("hillclimb_dp_graph/dpp", "DP", "@memo(V,Bool), Rooted, Mod<P>", "N=1e5 broom tree", gen_dpp),
    PortSpec("hillclimb_dp_graph/dps", "DP", "Mod<P>, .cyc()/% , rolling array", "|K|=1e4, D=100", gen_dps),
    PortSpec("hillclimb_dp_graph/dpv", "DP", "Rooted, runtime % m (cp_emod)", "N=1e5 star+chain", gen_dpv),
    PortSpec("btoi18p6", "DP", "Subset, 2D [Used,V] flattening", "N=3e5, M=3e5, K=5", gen_btoi18p6),
    PortSpec("ceoi18p1", "DP", "[Spare]?Int, List sort, knapsack", "N=2000, M=2000, C=50", gen_ceoi18p1),
    PortSpec("ioi04p4", "DP", "@memo(range,range), Set<Tup>", "W=600, H=600, N=200", gen_ioi04p4),
    PortSpec("cco18p1", "DP", "@memo(bounds), cp_step, enum", "N=1000", gen_cco18p1),
    # Graphs (6)
    PortSpec("cco24p1", "Graph", "MinHeap, [V]List<(V,Int)>", "N=5e5, M=1e6", gen_cco24p1),
    PortSpec("hillclimb_dp_graph/ioi11p4", "Graph", "MinHeap, Graph.undirected", "N=1e5, M=1e6, K=100", gen_ioi11p4),
    PortSpec("hillclimb_dp_graph/apio09p3", "Graph", "scc, condense, [District]?Int", "N=5e5, M=5e5", gen_apio09p3),
    PortSpec("ccc18s5", "Graph", "Dsu, enum, sort(by:)", "N=1e5, M=1e5, P=1e5, Q=1e5", gen_ccc18s5),
    PortSpec("ccc21s4", "Graph", "bfs, SegTree(std::function)", "N=2e5, W=2e5, D=2e5", gen_ccc21s4),
    PortSpec("noip09p3", "Graph", "OptI32, dual List adj", "N=1e5, M=5e5", gen_noip09p3),
    # Trees (5)
    PortSpec("btoi17p3", "Tree", "Rooted, lca, cp_emod, sort", "N=1e5, M=5e4, sum=2e5", gen_btoi17p3),
    PortSpec("hillclimb_dp_graph/noi15p2", "Tree", "Rooted HLD (chains/sub), AssignSum", "N=1e5, Q=1e5 binary", gen_noi15p2),
    PortSpec("ioi11p2", "Tree", "Rooted(weighted), HashMap(CpHash)", "N=2e5, K=1e6 binary", gen_ioi11p2),
    PortSpec("hillclimb_dp_graph/coi06p2", "Tree", "Rooted(lca/ancestor/step)", "N=1e5, M=5e5, Q=3e5", gen_coi06p2),
    PortSpec("hillclimb_dp_graph/sac22cc3p4", "Tree", "Rooted(lca), Dsu", "N=2e5, Q=2e5", gen_sac22cc3p4),
    # Segtree / Range / Sweep (3)
    PortSpec("ccc21s5", "Range", "SegTree(std::function+gcd), 2D diff", "N=1.5e5, Q=1e5", gen_ccc21s5),
    PortSpec("noi16p4", "Range", "AddMax lazy segtree, compress", "N=5e5, M=2e5", gen_noi16p4),
    PortSpec("cco20p1", "Range", "Steps(std::map), ediv", "N=1e5", gen_cco20p1),
    # Strings (3)
    PortSpec("ccc20s3", "String", "StrHash, Set(CpHash), CpDenseCounter", "|N|=1e5, |H|=2e5", gen_ccc20s3),
    PortSpec("noi14p4", "String", "KMP, Mod<P>, multitest, !unwrap", "T=5, |S|=1e6", gen_noi14p4),
    PortSpec("coci20c6p3", "String", "Map<[Letter]Int>, Mod<P>, 2D DP", "N=2000, K=2000", gen_coci20c6p3),
    # Sliding Window / Circular / Hashing (2)
    PortSpec("ccc22s4", "Window/Hash", "CpPrefix.wrap, cp_emod, cp_choose", "N=1e6, C=1e6", gen_ccc22s4),
    PortSpec("noi01p5", "Window/Hash", "CpCounter(CpHash), List comp", "N=6, M=150 (3.375M)", gen_noi01p5),
    # 2D Grids (3)
    PortSpec("ioi10p3", "Grid", "2D [Row,Col], CpPrefix, windows", "3000x3000, 1001x1001", gen_ioi10p3),
    PortSpec("dmopc21c9p4", "Grid", "2D Char, DenseSet, List", "1000x1000 solvable", gen_dmopc21c9p4),
    PortSpec("ccc21s2", "Grid", "enum match, [Row]Bool, I/O", "5e5x10, K=1e6", gen_ccc21s2),
]


# ---------------------------------------------------------------------------
# Build & timed execution helpers
# ---------------------------------------------------------------------------

def compile_cpp(src: str, out: str, extra_flags: Optional[List[str]] = None) -> None:
    cmd = ["g++", "-std=c++17", "-O2", *(extra_flags or []), "-o", out, src]
    r = subprocess.run(cmd, capture_output=True, text=True, timeout=180)
    if r.returncode != 0:
        raise RuntimeError(f"g++ failed for {src}:\n{r.stderr[:2000]}")


def run_timed(exe: str, inp_path: str, out_path: str, timeout: int = 120, stack_unlimited: bool = True):
    """Run exe with stdin=inp_path, stdout=out_path; return (wall_s, user_s, sys_s, rss_kb, rc)."""
    time_file = out_path + ".time"
    prefix = "ulimit -s unlimited; " if stack_unlimited else "ulimit -s 8192; "
    cmd = (
        f"{prefix}/usr/bin/time -f '%e %U %S %M' -o {time_file} "
        f"{exe} < {inp_path} > {out_path}"
    )
    t0 = time.perf_counter()
    p = subprocess.run(["bash", "-c", cmd], capture_output=True, text=True, timeout=timeout)
    t1 = time.perf_counter()
    wall_py = t1 - t0
    if not os.path.exists(time_file):
        return wall_py, 0.0, 0.0, 0, p.returncode
    lines = [l.strip() for l in open(time_file) if l.strip()]
    if not lines:
        return wall_py, 0.0, 0.0, 0, p.returncode
    parts = lines[-1].split()
    if len(parts) < 4:
        return wall_py, 0.0, 0.0, 0, p.returncode
    # Use high-res Python monotonic wall clock for sub-10ms precision, GNU time for user/sys/RSS
    try:
        return wall_py, float(parts[1]), float(parts[2]), int(parts[3]), p.returncode
    except ValueError:
        return wall_py, 0.0, 0.0, 0, p.returncode


def run_port(spec: PortSpec, tmpdir: str, runs: int, timeout: int) -> dict:
    slug = spec.name.replace("/", "__")
    cp_src = os.path.join(PORTS_DIR, spec.name + ".cp")
    orig_src = os.path.join(PORTS_DIR, spec.name + "_orig.cpp")
    work_cp = os.path.join(tmpdir, slug + ".cp")
    work_cpp = os.path.join(tmpdir, slug + ".cpp")
    ours_bin = os.path.join(tmpdir, slug + "_ours")
    orig_bin = os.path.join(tmpdir, slug + "_orig")
    inp_path = os.path.join(tmpdir, slug + ".in")
    ours_out = os.path.join(tmpdir, slug + "_ours.out")
    orig_out = os.path.join(tmpdir, slug + "_orig.out")

    shutil.copyfile(cp_src, work_cp)
    b = subprocess.run([sys.executable, CP_PY, "build", work_cp], capture_output=True, text=True, timeout=60)
    if b.returncode != 0:
        return {"name": spec.name, "status": "BUILD_FAIL", "error": b.stderr.strip()}

    cpp_bytes = os.path.getsize(work_cpp)
    cpp_lines = sum(1 for _ in open(work_cpp))

    try:
        compile_cpp(work_cpp, ours_bin)
        compile_cpp(orig_src, orig_bin)
    except Exception as e:
        return {"name": spec.name, "status": "CXX_FAIL", "cpp_bytes": cpp_bytes, "error": str(e)}

    spec.gen(inp_path)
    inp_kb = os.path.getsize(inp_path) / 1024.0

    # Warmup + verify output equality
    try:
        w_o, _, _, r_o, rc_o = run_timed(ours_bin, inp_path, ours_out, timeout=timeout)
        w_r, _, _, r_r, rc_r = run_timed(orig_bin, inp_path, orig_out, timeout=timeout)
    except subprocess.TimeoutExpired:
        return {"name": spec.name, "status": "TIMEOUT", "cpp_bytes": cpp_bytes}

    if rc_o != 0 or rc_r != 0:
        return {
            "name": spec.name,
            "status": f"RE(ours={rc_o},orig={rc_r})",
            "cpp_bytes": cpp_bytes,
            "ours_rss_kb": r_o,
            "orig_rss_kb": r_r,
        }

    # Verify token-for-token output equality (streaming chunked comparison for big outputs)
    with open(ours_out) as f1, open(orig_out) as f2:
        match = (f1.read().split() == f2.read().split())
    if not match:
        return {"name": spec.name, "status": "WA", "cpp_bytes": cpp_bytes}

    # Timed runs (adapt count if very slow)
    n_runs = runs if max(w_o, w_r) < 1.5 else max(1, min(runs, 2))
    ours_walls, orig_walls = [w_o], [w_r]
    ours_rss, orig_rss = [r_o], [r_r]
    for _ in range(n_runs - 1):
        wo, _, _, ro, _ = run_timed(ours_bin, inp_path, ours_out, timeout=timeout)
        wr, _, _, rr, _ = run_timed(orig_bin, inp_path, orig_out, timeout=timeout)
        ours_walls.append(wo)
        orig_walls.append(wr)
        ours_rss.append(ro)
        orig_rss.append(rr)

    # Remove large temp I/O files to save disk space
    for p in (inp_path, ours_out, orig_out):
        if os.path.exists(p):
            os.remove(p)

    med_ours = statistics.median(ours_walls)
    med_orig = statistics.median(orig_walls)
    max_ours_rss = max(ours_rss)
    max_orig_rss = max(orig_rss)
    time_ratio = med_ours / max(med_orig, 1e-6)
    rss_ratio = max_ours_rss / max(max_orig_rss, 1)

    return {
        "name": spec.name,
        "category": spec.category,
        "features": spec.features,
        "limits": spec.limits,
        "status": "OK",
        "inp_kb": round(inp_kb, 1),
        "cpp_bytes": cpp_bytes,
        "cpp_lines": cpp_lines,
        "under_64kb": cpp_bytes <= 65536,
        "ours_ms": round(med_ours * 1000, 1),
        "orig_ms": round(med_orig * 1000, 1),
        "time_ratio": round(time_ratio, 3),
        "ours_rss_kb": max_ours_rss,
        "orig_rss_kb": max_orig_rss,
        "rss_ratio": round(rss_ratio, 3),
    }


# ---------------------------------------------------------------------------
# Adversarial probes (anti-hash, 8MB stack depth, CpView/lends microbench)
# ---------------------------------------------------------------------------

def invert_splitmix64(h: int) -> int:
    """Invert CpHash (fixed-seed splitmix64) to produce colliding keys mod bucket_count."""
    M = (1 << 64) - 1
    # z ^= z >> 31 (self-inverse twice)
    z = h ^ (h >> 31) ^ (h >> 62)
    z = (z * pow(0x94D049BB133111EB, -1, 1 << 64)) & M
    # z ^= z >> 27 (3 iterations suffice for 64 bits)
    z = z ^ (z >> 27) ^ (z >> 54)
    z = (z * pow(0xBF58476D1CE4E5B9, -1, 1 << 64)) & M
    # z ^= z >> 30
    z = z ^ (z >> 30) ^ (z >> 60)
    x = (z - 0x9E3779B97F4A7C15) & M
    if x >= (1 << 63):
        x -= (1 << 64)
    return x


def run_adversarial(tmpdir: str) -> dict:
    results = {}

    # 1. Anti-hash collision attack on the actual runtime.hpp CpHash (both unordered_map and CpHashMap)
    probe_cpp = os.path.join(tmpdir, "antihash.cpp")
    probe_bin = os.path.join(tmpdir, "antihash")
    runtime_inc = os.path.join(REPO_ROOT, "proto/cpc")
    with open(probe_cpp, "w") as f:
        f.write(r"""
#include <bits/stdc++.h>
#include "runtime.hpp"
int main(int argc, char** argv) {
    int n; cin >> n;
    bool use_custom = argc > 1 && string(argv[1]) == "custom";
    if (use_custom) {
        CpHashMap<ll, ll, CpHash> m;
        for (int i = 0; i < n; ++i) { ll k; cin >> k; m[k] += 1; }
        cout << m.size() << " " << m.cap << " 0\n";
    } else {
        unordered_map<ll, ll, CpHash> m;
        for (int i = 0; i < n; ++i) { ll k; cin >> k; m[k] += 1; }
        cout << m.size() << " " << m.bucket_count() << " " << m.bucket_size(0) << "\n";
    }
}
""")
    subprocess.run(["g++", "-std=c++17", "-O2", f"-I{runtime_inc}", probe_cpp, "-o", probe_bin], check=True)
    n_keys = 50_000
    rand_inp = os.path.join(tmpdir, "ah_rand.in")
    with open(rand_inp, "w") as f:
        f.write(f"{n_keys}\n" + "\n".join(str(i * 1234567 + 1) for i in range(n_keys)) + "\n")
    out_r = os.path.join(tmpdir, "ah_rand.out")
    w_rand, _, _, rss_rand, _ = run_timed(probe_bin, rand_inp, out_r)
    parts = open(out_r).read().split()
    bc = int(parts[1])  # exact libstdc++ prime bucket count at N=50,000

    # Generate 50,000 keys whose unseeded splitmix64(k) is a multiple of bc (used to collide 100% into bucket 0!)
    kill_inp = os.path.join(tmpdir, "ah_kill.in")
    kill_keys = [str(invert_splitmix64(i * bc)) for i in range(1, n_keys + 1)]
    with open(kill_inp, "w") as f:
        f.write(f"{n_keys}\n" + "\n".join(kill_keys) + "\n")
    out_k = os.path.join(tmpdir, "ah_kill.out")
    w_kill, _, _, rss_kill, _ = run_timed(probe_bin, kill_inp, out_k)
    kparts = open(out_k).read().split()
    results["antihash_cphash"] = {
        "n_keys": n_keys,
        "bucket_count": bc,
        "bucket0_size_killer": int(kparts[2]),
        "random_ms": round(w_rand * 1000, 1),
        "killer_ms": round(w_kill * 1000, 1),
        "slowdown": round(w_kill / max(w_rand, 1e-6), 2),
    }

    # Also test power-of-2 bucket killer keys against the real CpHashMap<ll, ll, CpHash>
    kill_p2_inp = os.path.join(tmpdir, "ah_kill_p2.in")
    p2_mod = 1 << 17  # 131072 slots for N=50,000
    with open(kill_p2_inp, "w") as f:
        f.write(f"{n_keys}\n" + "\n".join(str(invert_splitmix64(i * p2_mod)) for i in range(1, n_keys + 1)) + "\n")
    out_cr = os.path.join(tmpdir, "ah_crand.out")
    out_ck = os.path.join(tmpdir, "ah_ckill.out")
    t0 = time.perf_counter()
    subprocess.run([probe_bin, "custom"], stdin=open(rand_inp), stdout=open(out_cr, "w"), check=True)
    w_cr = time.perf_counter() - t0
    t0 = time.perf_counter()
    subprocess.run([probe_bin, "custom"], stdin=open(kill_p2_inp), stdout=open(out_ck, "w"), check=True)
    w_ck = time.perf_counter() - t0
    results["antihash_cphashmap_pow2"] = {
        "n_keys": n_keys,
        "random_ms": round(w_cr * 1000, 1),
        "killer_ms": round(w_ck * 1000, 1),
        "slowdown": round(w_ck / max(w_cr, 1e-6), 2),
    }

    # 2. Stack-overflow probe on dpp (chain of N=100,000 under default 8MB stack)
    dpp_cp = os.path.join(tmpdir, "dpp_stack.cp")
    shutil.copyfile(os.path.join(PORTS_DIR, "hillclimb_dp_graph/dpp.cp"), dpp_cp)
    subprocess.run([sys.executable, CP_PY, "build", dpp_cp], capture_output=True, check=True)
    dpp_bin = os.path.join(tmpdir, "dpp_stack_ours")
    dpp_orig = os.path.join(tmpdir, "dpp_stack_orig")
    compile_cpp(os.path.join(tmpdir, "dpp_stack.cpp"), dpp_bin)
    compile_cpp(os.path.join(PORTS_DIR, "hillclimb_dp_graph/dpp_orig.cpp"), dpp_orig)
    chain_in = os.path.join(tmpdir, "dpp_chain.in")
    n_chain = 100_000
    with open(chain_in, "w") as f:
        f.write(f"{n_chain}\n" + "\n".join(f"{i} {i + 1}" for i in range(1, n_chain)) + "\n")
    _, _, _, _, rc_ours_8mb = run_timed(dpp_bin, chain_in, os.path.join(tmpdir, "dpp_8.out"), stack_unlimited=False)
    _, _, _, _, rc_orig_8mb = run_timed(dpp_orig, chain_in, os.path.join(tmpdir, "dpp_o8.out"), stack_unlimited=False)
    w_cu, _, _, rss_cu, rc_ours_unlim = run_timed(dpp_bin, chain_in, os.path.join(tmpdir, "dpp_u.out"), stack_unlimited=True)
    w_ou, _, _, rss_ou, rc_orig_unlim = run_timed(dpp_orig, chain_in, os.path.join(tmpdir, "dpp_ou.out"), stack_unlimited=True)
    results["dpp_chain_100k"] = {
        "ours_rc_8mb_stack": rc_ours_8mb,
        "orig_rc_8mb_stack": rc_orig_8mb,
        "ours_rc_unlim": rc_ours_unlim,
        "orig_rc_unlim": rc_orig_unlim,
        "ours_ms_unlim": round(w_cu * 1000, 1),
        "orig_ms_unlim": round(w_ou * 1000, 1),
        "ours_rss_kb_unlim": rss_cu,
        "orig_rss_kb_unlim": rss_ou,
    }

    # 3. CpView / cp_win bounds-check microbenchmark at N=2e7 (simulating lends inner loop)
    lend_cpp = os.path.join(tmpdir, "lend_bench.cpp")
    lend_bin = os.path.join(tmpdir, "lend_bench")
    with open(lend_cpp, "w") as f:
        f.write(r"""
#include <bits/stdc++.h>
using namespace std;
using ll = long long;
[[noreturn]] static void cp_fail(const char* what, int line) { fprintf(stderr, "line %d: %s\n", line, what); exit(3); }
template <class T> struct CpView {
    T* p = nullptr; int lo = 0, hi = 0;
    CpView(T* p, ll lo, ll hi) : p(p), lo((int)lo), hi((int)(hi < lo ? lo : hi)) {}
    T& operator[](ll i) const { return p[i]; }
};
static inline ll cp_win(ll i, ll lo, ll hi, int line) {
    if (i < lo || i >= hi) cp_fail("position is outside the slice", line);
    return i;
}
__attribute__((noinline)) ll sum_checked(CpView<ll> v, int lo, int hi) {
    ll s = 0;
    for (int i = lo; i < hi; ++i) s += v[cp_win(i, v.lo, v.hi, 10)];
    return s;
}
__attribute__((noinline)) ll sum_raw(const ll* p, int lo, int hi) {
    ll s = 0;
    for (int i = lo; i < hi; ++i) s += p[i];
    return s;
}
int main() {
    const int N = 20000000;
    vector<ll> a(N);
    for (int i = 0; i < N; ++i) a[i] = i * 3 + 1;
    CpView<ll> v(a.data(), 0, N);
    auto bench = [&](auto fn) {
        volatile ll sink = 0;
        auto t0 = chrono::steady_clock::now();
        for (int r = 0; r < 10; ++r) sink = sink + fn();
        auto t1 = chrono::steady_clock::now();
        return chrono::duration<double, milli>(t1 - t0).count();
    };
    double t_chk = bench([&]{ return sum_checked(v, 0, N); });
    double t_raw = bench([&]{ return sum_raw(a.data(), 0, N); });
    cout << fixed << setprecision(2) << t_chk << " " << t_raw << " " << (t_chk / t_raw) << "\n";
}
""")
    compile_cpp(lend_cpp, lend_bin)
    l_out = subprocess.run([lend_bin], capture_output=True, text=True, check=True).stdout.strip().split()
    results["cp_win_microbench_2e8_ops"] = {
        "checked_ms": float(l_out[0]),
        "raw_ms": float(l_out[1]),
        "ratio": float(l_out[2]),
    }

    # 4. Euclidean modulo (cp_emod) vs native % microbenchmark (5e7 ops with runtime positive modulus)
    emod_cpp = os.path.join(tmpdir, "emod_bench.cpp")
    emod_bin = os.path.join(tmpdir, "emod_bench")
    with open(emod_cpp, "w") as f:
        f.write(r"""
#include <bits/stdc++.h>
using namespace std;
using ll = long long;
[[noreturn]] static void cp_fail(const char* what, int line) { fprintf(stderr, "line %d: %s\n", line, what); exit(3); }
static inline ll cp_emod(ll a, ll b, int line = 0) {
    if (b == 0) cp_fail("modulo by zero", line);
    ll r = a % b;
    if (r < 0) r += (b > 0 ? b : -b);
    return r;
}
__attribute__((noinline)) ll loop_emod(ll n, ll m) {
    ll acc = 1;
    for (ll i = 1; i <= n; ++i) acc = cp_emod(acc * i + 7, m, 42);
    return acc;
}
__attribute__((noinline)) ll loop_native(ll n, ll m) {
    ll acc = 1;
    for (ll i = 1; i <= n; ++i) acc = (acc * i + 7) % m;
    return acc;
}
int main() {
    const ll N = 50000000;
    volatile ll m = 1000000007;
    auto bench = [&](auto fn) {
        volatile ll sink = 0;
        auto t0 = chrono::steady_clock::now();
        sink = fn(N, m);
        auto t1 = chrono::steady_clock::now();
        return chrono::duration<double, milli>(t1 - t0).count();
    };
    double t_em = bench(loop_emod);
    double t_na = bench(loop_native);
    cout << fixed << setprecision(2) << t_em << " " << t_na << " " << (t_em / t_na) << "\n";
}
""")
    compile_cpp(emod_cpp, emod_bin)
    e_out = subprocess.run([emod_bin], capture_output=True, text=True, check=True).stdout.strip().split()
    results["cp_emod_microbench_5e7_ops"] = {
        "emod_ms": float(e_out[0]),
        "native_ms": float(e_out[1]),
        "ratio": float(e_out[2]),
    }

    return results


def main() -> None:
    ap = argparse.ArgumentParser(description="Stage-6 Max-N Judge & Constant-Factor Audit")
    ap.add_argument("--ports", help="Comma-separated subset of port names (substring match)")
    ap.add_argument("--runs", type=int, default=3, help="Timed runs per port (default 3)")
    ap.add_argument("--timeout", type=int, default=120, help="Per-run timeout in seconds")
    ap.add_argument("--adversarial", action="store_true", help="Also run anti-hash, stack, and CpView probes")
    ap.add_argument("--json", dest="json_out", help="Write full JSON results to this path")
    ap.add_argument("--list", action="store_true", help="List configured ports and exit")
    args = ap.parse_args()

    selected = PORTS
    if args.list:
        for i, p in enumerate(PORTS, 1):
            print(f"{i:2d}. {p.name:32s} [{p.category:11s}] {p.limits}  ({p.features})")
        return

    if args.ports:
        wanted = [x.strip() for x in args.ports.split(",") if x.strip()]
        selected = [p for p in PORTS if any(w in p.name for w in wanted)]
        if not selected:
            sys.exit(f"No ports matched {wanted}")

    tmpdir = tempfile.mkdtemp(prefix="cp_max_n_")
    try:
        rows = []
        hdr = f"{'Port':30s} {'Cat':11s} {'Ours ms':>8s} {'Orig ms':>8s} {'T-Ratio':>7s} {'Ours MB':>8s} {'Orig MB':>8s} {'M-Ratio':>7s} {'C++ KB':>7s} {'Status'}"
        print(hdr)
        print("-" * len(hdr))
        for spec in selected:
            res = run_port(spec, tmpdir, args.runs, args.timeout)
            rows.append(res)
            if res["status"] == "OK":
                flag = ""
                if res["time_ratio"] > 1.25:
                    flag += " [SLOW]"
                if res["rss_ratio"] > 1.5:
                    flag += " [MEM]"
                if not res["under_64kb"]:
                    flag += " [>64KB]"
                print(
                    f"{res['name']:30s} {res['category']:11s} "
                    f"{res['ours_ms']:8.1f} {res['orig_ms']:8.1f} {res['time_ratio']:7.2f}x "
                    f"{res['ours_rss_kb']/1024:7.1f}M {res['orig_rss_kb']/1024:7.1f}M {res['rss_ratio']:6.2f}x "
                    f"{res['cpp_bytes']/1024:6.1f}K OK{flag}"
                )
            else:
                print(f"{res['name']:30s} {spec.category:11s} {'-':>8s} {'-':>8s} {'-':>7s} {'-':>8s} {'-':>8s} {'-':>7s} {res.get('cpp_bytes',0)/1024:6.1f}K {res['status']}")
                if res.get("error"):
                    print(f"   error: {res['error'][:200]}")
            sys.stdout.flush()

        adv = {}
        if args.adversarial:
            print("\n=== Adversarial Probes ===")
            adv = run_adversarial(tmpdir)
            print(json.dumps(adv, indent=2))

        if args.json_out:
            with open(args.json_out, "w") as f:
                json.dump({"ports": rows, "adversarial": adv}, f, indent=2)
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)


if __name__ == "__main__":
    main()
