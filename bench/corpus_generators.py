#!/usr/bin/env python3
"""Expanded hand-written max-N input generators for `bench/corpus_p95_bench.py`.

Each entry in `SPECS` follows the same 6-tuple schema as `rust_max_n_audit.SPECS`:
    (port_rel_without_cp, original_path, topic, limits_label, gen_fn, compare_mode)
where `gen_fn(path, big, ctx)` writes a deterministic valid judge-limit input when
`big` is True (and a smaller sanity input otherwise).

Keeping the expanded corpus generators in this module leaves the 40-port CI gate
in `rust_max_n_audit.py` fast while lifting `corpus_p95_bench.py` past 250-300+
measured ports.
"""
import os
import random

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PORTS = os.path.join(ROOT, "proto", "ports")
CORPUS = os.path.join(ROOT, "corpus", "submissions")


def O(n):
    return os.path.join(CORPUS, f"{n}.cpp")


def P(rel):
    return os.path.join(PORTS, rel)


def W(path, lines):
    with open(path, "w") as f:
        f.write("\n".join(map(str, lines)) + "\n")


def J(xs):
    return " ".join(map(str, xs))


def letters(rng, n, alphabet="ab"):
    return "".join(rng.choice(alphabet) for _ in range(n))


def tree_edges(rng, n, one_based=True, chain_frac=0.3):
    """Connected tree with a deep chain + random branching (1-based or 0-based)."""
    off = 1 if one_based else 0
    chain = int(n * chain_frac)
    edges = []
    for i in range(1, n):
        p = i - 1 if i <= chain else rng.randint(0, i - 1)
        edges.append((p + off, i + off))
    return edges


# =============================================================================
# Group 1: Non-interactive ports previously skipped under `no input both agree`
# =============================================================================

def g_apio07p2(p, big, _):
    rng = random.Random(1001)
    n = 100_000 if big else 200
    k = n // 2
    pos = []
    cur = 0
    for _ in range(n):
        cur += rng.randint(1, 5000)
        pos.append(cur)
    W(p, [f"{n} {k}"] + pos)


def g_btoi08p6(p, big, _):
    rng = random.Random(1002)
    n = 20 if big else 10
    # Original uses 32-bit int sums, so keep sum(a), sum(b) < 1e9
    W(p, [n, J(rng.randint(1, 2 * 10**7) for _ in range(n)), J(rng.randint(1, 2 * 10**7) for _ in range(n))])


def g_btoi19p2(p, big, _):
    rng = random.Random(1003)
    r, c, m = (500, 500, 5000) if big else (40, 40, 200)
    grid = ["".join("#" if rng.random() < 0.25 else "." for _ in range(c)) for _ in range(r)]
    sigs = "".join(rng.choice("NESW??") for _ in range(m))
    W(p, [f"{r} {c} {m}"] + grid + [sigs])


def g_ccc02s4(p, big, _):
    rng = random.Random(1004)
    m, n = (20, 100) if big else (5, 20)
    lines = [m, n]
    for i in range(n):
        lines.append(f"p{i:03d}")
        lines.append(rng.randint(1, 100))
    W(p, lines)


def g_ccc21s2(p, big, _):
    rng = random.Random(1005)
    # Judge limits: M * N <= 5e6, K <= 1e6
    rows, cols, k = (2500, 2000, 1_000_000) if big else (50, 50, 200)
    lines = [rows, cols, k]
    for _ in range(k):
        if rng.random() < 0.5:
            lines.append(f"R {rng.randint(1, rows)}")
        else:
            lines.append(f"C {rng.randint(1, cols)}")
    W(p, lines)


def g_cco07p2(p, big, _):
    rng = random.Random(1006)
    n = 100_000 if big else 500
    lines = [n] + [J(rng.randint(1, 10**7) for _ in range(6)) for _ in range(n)]
    W(p, lines)


def g_cco19p3(p, big, _):
    rng = random.Random(1007)
    n = 200_000 if big else 500
    a = [rng.randint(1, 10000) for _ in range(n)]
    pars = [rng.randint(max(1, i - 4), i - 1) for i in range(2, n + 1)]
    W(p, [n, J(a), J(pars)])


def g_cco23p4(p, big, _):
    rng = random.Random(1008)
    n = 1_000_000 if big else 2000
    s = "".join(rng.choice("01") for _ in range(n))
    W(p, [s, "000"])


def g_ceoi22p2(p, big, _):
    rng = random.Random(1009)
    leaves = 200_000 if big else 200
    # Iteratively build a balanced-ish random min/max expression tree with `leaves` leaves
    # Output tokens without deep recursion
    # Represent nodes as either None (leaf) or (op, left, right)
    nodes = [None] * leaves
    while len(nodes) > 1:
        nxt = []
        i = 0
        while i + 1 < len(nodes):
            op = "min" if rng.random() < 0.5 else "max"
            nxt.append((op, nodes[i], nodes[i + 1]))
            i += 2
        if i < len(nodes):
            nxt.append(nodes[i])
        nodes = nxt
    out = []
    stack = [nodes[0]]
    while stack:
        cur = stack.pop()
        if isinstance(cur, str):
            out.append(cur)
        elif cur is None:
            out.append("?")
        else:
            op, l, r = cur
            out.append(op + "(")
            stack.append(")")
            stack.append(r)
            stack.append(",")
            stack.append(l)
    W(p, ["".join(out)])


def g_cheerio1s5(p, big, _):
    rng = random.Random(1010)
    n, k = (400, 150) if big else (40, 15)
    s = "".join(rng.choice("HT") for _ in range(n))
    W(p, [s, k])


def g_coci21c1p2(p, big, _):
    rng = random.Random(1011)
    n, k = (350, 150) if big else (40, 15)
    s = "".join(rng.choice("CP") for _ in range(n))
    if s.count("C") < 2 * k:
        s = "C" * (2 * k) + "P" * (n - 2 * k)
    W(p, [f"{n} {k}", s])


def g_dmopc19c4p2(p, big, _):
    n = 16 if big else 6
    # Start at (1, 1), target at (n, n), empty cells e1=(1,1) and e2=(n,n)
    W(p, [f"{n} 1 1 {n} {n}", f"1 1 {n} {n}"])


def g_dmopc21c7p3(p, big, _):
    rng = random.Random(1013)
    n, m = (1_000_000, 1_000_000) if big else (1000, 2000)
    lit = [rng.randint(0, 1) for _ in range(n)]
    # DAG edges u < v so no cycles force -1 trivially, plus some sparse structure
    edges = [f"{rng.randint(1, n - 1)} {rng.randint(2, n)}" for _ in range(m)]
    # Ensure u < v for DAG
    edges = []
    for _ in range(m):
        u = rng.randint(1, n - 1)
        v = rng.randint(u + 1, min(n, u + 50))
        edges.append(f"{u} {v}")
    W(p, [f"{n} {m}", J(lit)] + edges)


def g_dmopc21c8p3(p, big, _):
    n, k = (1_000_000, 10**15) if big else (50, 10**6)
    W(p, [f"{n} {k}"])


def g_dmpg16g2(p, big, _):
    rng = random.Random(1015)
    n = 500 if big else 40
    alpha = "......ABCDE"
    W(p, [n] + ["".join(rng.choice(alpha) for _ in range(n)) for _ in range(n)])


def g_dpg(p, big, _):
    rng = random.Random(1016)
    n, m = (100_000, 100_000) if big else (1000, 2000)
    edges = []
    for i in range(1, min(n, 30000)):
        edges.append(f"{i} {i + 1}")
    while len(edges) < m:
        u = rng.randint(1, n - 1)
        v = rng.randint(u + 1, n)
        edges.append(f"{u} {v}")
    W(p, [f"{n} {m}"] + edges)


def g_dps(p, big, _):
    rng = random.Random(1017)
    digits = 10_000 if big else 500
    k = "9" + "".join(str(rng.randint(0, 9)) for _ in range(digits - 1))
    W(p, [k, 100])


def g_egoi22p2(p, big, _):
    # Judge limit w <= 250000 (orig O(w^2) runs up to 1e4), so w=5000, h=1e9
    w, h = (5000, 10**9) if big else (200, 1000)
    W(p, [f"{w} {h}"])


def g_ioi00p2(p, big, _):
    rng = random.Random(1019)
    n, m, w = (20_000, 50, 50) if big else (200, 10, 10)
    cars = [rng.randint(1, m) for _ in range(n)]
    W(p, [f"{n} {m} {w}", J(cars)])


def g_ioi05p4(p, big, _):
    rng = random.Random(1020)
    n = 1_000_000 if big else 2000
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    W(p, [n, J(perm)])


def g_ioi08p4(p, big, _):
    rng = random.Random(1021)
    n, m = (1_000_000, 1_000_000) if big else (1000, 500)
    pts = list(range(1, 2 * n + 1))
    rng.shuffle(pts)
    lines = [n, m]
    for i in range(n):
        a, b = pts[2 * i], pts[2 * i + 1]
        if a > b:
            a, b = b, a
        lines.append(f"{a} {b}")
    W(p, lines)


def g_ioi10p3(p, big, _):
    rng = random.Random(1022)
    r, c, h, w = (1000, 1000, 301, 301) if big else (40, 40, 11, 11)
    vals = list(range(1, r * c + 1))
    rng.shuffle(vals)
    rows = [J(vals[i * c:(i + 1) * c]) for i in range(r)]
    W(p, [f"{r} {c} {h} {w}"] + rows)


def g_ioi11p3(p, big, _):
    rng = random.Random(1023)
    # Original's test wrapper reads `int b` (32-bit), so keep budget <= 1e9
    n, length, budget = (100_000, 10**9, 10**9) if big else (1000, 10**5, 10**6)
    xs = sorted(rng.randint(1, length) for _ in range(n))
    W(p, [f"{n} {length} {budget}"] + xs)


def g_ioi14p6io(p, big, _):
    rng = random.Random(1024)
    n, start, d = (100_000, 50_000, 150_000) if big else (500, 200, 400)
    a = [rng.randint(0, 10**9) for _ in range(n)]
    W(p, [f"{n} {start} {d}", J(a)])


def g_nccc9s2(p, big, _):
    rng = random.Random(1025)
    n = 1_000_000 if big else 2000
    chars = ["I"] * n + ["U"] * n
    rng.shuffle(chars)
    W(p, [n, "".join(chars)])


def g_noi02p3(p, big, _):
    rng = random.Random(1026)
    n, m, k = (300, 2, 150) if big else (40, 2, 15)
    edges = [f"{u} {v} {rng.randint(0, 100_000)}" for u, v in tree_edges(rng, n, True, 0.2)]
    W(p, [f"{n} {m} {k}"] + edges)


def g_noi03p1(p, big, _):
    # Equation with 1 matchstick move: 1+2+3+4+5+6+7+8+9=46# -> moving 1 match in 6->5 gives 45
    W(p, ["123+456+789-100+234+567-890+111+222-333+444=1624#"])


def g_noi06p4(p, big, _):
    rng = random.Random(1028)
    n, m = (5000, 50_000) if big else (200, 1000)
    costs = [rng.randint(1, 1000) for _ in range(n)]
    users = []
    for _ in range(m):
        a = rng.randint(1, n)
        b = rng.randint(1, n - 1)
        if b >= a:
            b += 1
        users.append(f"{a} {b} {rng.randint(1, 1000)}")
    W(p, [f"{n} {m}", J(costs)] + users)


def g_noi09p5(p, big, _):
    rng = random.Random(1029)
    n, m = (500, 500) if big else (40, 40)
    W(p, [f"{n} {m}", letters(rng, n, "AB"), letters(rng, m, "AB")])


def g_noimps22p5(p, big, _):
    rng = random.Random(1030)
    n = 400_000 if big else 1000
    # Valid balanced bracket string with nontrivial depth
    s = []
    bal = 0
    rem_open = n
    while len(s) < 2 * n:
        if rem_open == 0:
            s.append(")"); bal -= 1
        elif bal == 0:
            s.append("("); bal += 1; rem_open -= 1
        elif bal > 40 or rng.random() < 0.45:
            s.append(")"); bal -= 1
        else:
            s.append("("); bal += 1; rem_open -= 1
    w = [rng.randint(1, 10**7) for _ in range(n)]
    W(p, [f"{n} 1 0", "".join(s), J(w)])


def g_noip19p1(p, big, _):
    n = 64
    k = (1 << 64) - 123456789012345 if big else 987654321
    W(p, [f"{n} {k}"])


def g_noip20p3(p, big, _):
    rng = random.Random(1032)
    n, m = (50, 400) if big else (6, 20)
    balls = [c for c in range(1, n + 1) for _ in range(m)]
    rng.shuffle(balls)
    lines = [f"{n} {m}"] + [J(balls[i * m:(i + 1) * m]) for i in range(n)]
    W(p, lines)


def g_sac22cc2p4(p, big, _):
    rng = random.Random(1033)
    n, m = (1000, 1000) if big else (50, 50)
    W(p, [f"{n} {m}"] + ["".join(rng.choice(".C") for _ in range(m)) for _ in range(n)])


def g_sac22cc3p4(p, big, _):
    rng = random.Random(1034)
    n, q = (200_000, 200_000) if big else (1000, 1000)
    edges = tree_edges(rng, n, True, 0.25)
    cut_order = list(edges)
    rng.shuffle(cut_order)
    ci = 0
    qs = []
    for _ in range(q):
        if ci < len(cut_order) and rng.random() < 0.3:
            u, v = cut_order[ci]; ci += 1
            qs.append(f"2 {u} {v}")
        else:
            qs.append(f"1 {rng.randint(1, n)} {rng.randint(1, n)}")
    W(p, [f"{n} {q}"] + [f"{u} {v}" for u, v in edges] + qs)


def g_sac22cc4jp4(p, big, _):
    rng = random.Random(1035)
    n, k = (200_000, 10**18) if big else (1000, 10**9)
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    W(p, [f"{n} {k}", J(perm)])


def g_ucrpc21d(p, big, _):
    rng = random.Random(1036)
    n, m = (500, 500) if big else (30, 30)
    rows = []
    for r in range(2 * n + 1):
        row = []
        for c in range(2 * m + 1):
            if r == 0 or r == 2 * n:
                row.append("+" if c % 2 == 0 else "-")
            elif c == 0 or c == 2 * m:
                row.append("+" if r % 2 == 0 else "|")
            elif r % 2 == 0 and c % 2 == 0:
                row.append("+")
            elif r % 2 == 1 and c % 2 == 1:
                row.append(".")
            elif r % 2 == 0:
                row.append("-" if rng.random() < 0.45 else ".")
            else:
                row.append("|" if rng.random() < 0.45 else ".")
        rows.append("".join(row))
    W(p, [f"{n} {m}"] + rows)


def g_wossoly21s3(p, big, _):
    rng = random.Random(1037)
    n, m = (200, 200) if big else (30, 30)
    grid = [["#" if rng.random() < 0.2 else "." for _ in range(m)] for _ in range(n)]
    # Place 0,1,2,3,4 and 10 portal pairs A..J -> a..j at distinct cells
    coords = rng.sample([(r, c) for r in range(n) for c in range(m)], 5 + 20)
    for idx, ch in enumerate("01234"):
        r, c = coords[idx]
        grid[r][c] = ch
    for k in range(10):
        r1, c1 = coords[5 + 2 * k]
        r2, c2 = coords[5 + 2 * k + 1]
        grid[r1][c1] = chr(ord("A") + k)
        grid[r2][c2] = chr(ord("a") + k)
    W(p, [f"{n} {m}"] + ["".join(row) for row in grid])


def g_yac1p4(p, big, _):
    rng = random.Random(1038)
    n = 250 if big else 40
    # Uppercase letters; keep values moderate so bitset stays within original's maxb = 675*130
    W(p, [n, "".join(rng.choice("ABCDEFGHIJKLMNOP") for _ in range(n))])


def g_yac6p4(p, big, _):
    rng = random.Random(1039)
    n, m = (200_000, 100) if big else (1000, 100)
    W(p, [f"{n} {m}", J(rng.randint(1, m) for _ in range(n))])


SPECS = [
    ("scout_batch_13/today/apio07p2", O(5532752), "greedy+heap", "n=1e5, k=5e4", g_apio07p2, "exact"),
    ("scout_batch_38/today/btoi08p6", O(5325518), "pareto sweep", "n=20", g_btoi08p6, "exact"),
    ("scout_batch_20/btoi19p2", O(4456161), "bitset grid", "500x500, m=5000", g_btoi19p2, "exact"),
    ("blind_critic_2/ccc02s4", O(4157526), "DP+recon", "m=20, n=100", g_ccc02s4, "exact"),
    ("wave2/ccc21s2", O(4323443), "grid parity", "2500x2000, k=1e6", g_ccc21s2, "exact"),
    ("blind_critic_2/cco07p2", O(3742713), "hashing/set", "n=1e5", g_cco07p2, "exact"),
    ("scout_batch_8/cco19p3", O(3618571), "tree+subset", "n=2e5", g_cco19p3, "exact"),
    ("scout_batch_36/today/cco23p4", O(5579929), "binary string", "|s|=1e6, t=000", g_cco23p4, "exact"),
    ("scout_batch_19/today/ceoi22p2", O(4727898), "expr tree", "2e5 leaves", g_ceoi22p2, "exact"),
    ("scout_batch_40/today/cheerio1s5", O(5339756), "interval DP", "n=400, k=150", g_cheerio1s5, "exact"),
    ("scout_batch_23/today/coci21c1p2", O(4080842), "game DP", "n=350, k=150", g_coci21c1p2, "exact"),
    ("scout_batch_31/today/dmopc19c4p2", O(5323330), "4D BFS", "n=16", g_dmopc19c4p2, "exact"),
    ("scout_batch_25/today/dmopc21c7p3", O(4453037), "DAG DFS", "n=m=1e6", g_dmopc21c7p3, "exact"),
    ("scout_batch_34/today/dmopc21c8p3", O(4537755), "fenwick perm", "n=1e6, k=1e15", g_dmopc21c8p3, "exact"),
    ("scout_batch_39/today/dmpg16g2", O(4653887), "grid+bitmask", "n=500", g_dmpg16g2, "exact"),
    ("dpg", P("dpg_orig.cpp"), "DAG DP", "n=m=1e5", g_dpg, "exact"),
    ("hillclimb_dp_graph/dps", P("hillclimb_dp_graph/dps_orig.cpp"), "digit DP", "|K|=1e4, D=100", g_dps, "exact"),
    ("scout_batch_23/today/egoi22p2", O(5199888), "DP mod", "w=5000, h=1e9", g_egoi22p2, "exact"),
    ("scout_batch_49/today/ioi00p2", O(4682555), "cycle sim", "n=2e4, m=w=50", g_ioi00p2, "exact"),
    ("scout_batch_13/today/ioi05p4", O(5044895), "circular gap", "n=1e6", g_ioi05p4, "exact"),
    ("scout_batch_17/ioi08p4", O(5637680), "cycles+greedy", "n=m=1e6", g_ioi08p4, "exact"),
    ("ioi10p3", P("ioi10p3_orig.cpp"), "2D prefix+bs", "1000x1000, 301x301", g_ioi10p3, "exact"),
    ("scout_batch_13/today/ioi11p3", O(3732163), "two pointers", "n=1e5", g_ioi11p3, "exact"),
    ("scout_batch_52/ioi14p6io", O(4691224), "DnC DP+sqrt", "n=1e5, d=1.5e5", g_ioi14p6io, "exact"),
    ("scout_batch_41/today/nccc9s2", O(4051671), "greedy", "n=1e6", g_nccc9s2, "exact"),
    ("scout_batch_4/noi02p3", O(3718952), "tree DP", "n=300, m=2, k=150", g_noi02p3, "exact"),
    ("scout_batch_16/noi03p1", O(3738358), "string parse", "matchstick eq", g_noi03p1, "exact"),
    ("scout_batch_5/today/noi06p4", O(5032812), "max flow", "n=5000, m=5e4", g_noi06p4, "exact"),
    ("scout_batch_23/today/noi09p5", O(5108108), "DP mod", "n=m=500", g_noi09p5, "exact"),
    ("scout_batch_45/today/noimps22p5", O(5448138), "bracket+set", "n=4e5", g_noimps22p5, "exact"),
    ("scout_batch_25/today/noip19p1", O(5127723), "gray code", "n=64, k~2^64", g_noip19p1, "exact"),
    ("scout_batch_52/noip20p3", O(5445674), "DnC sort", "n=50, m=400", g_noip20p3, "exact"),
    ("blind_critic_2/sac22cc2p4", O(4121086), "0-1 BFS", "1000x1000", g_sac22cc2p4, "exact"),
    ("hillclimb_dp_graph/sac22cc3p4", P("hillclimb_dp_graph/sac22cc3p4_orig.cpp"), "offline tree", "n=q=2e5", g_sac22cc3p4, "exact"),
    ("scout_batch_8/sac22cc4jp4", O(4416096), "perm cycles", "n=2e5, k=1e18", g_sac22cc4jp4, "exact"),
    ("scout_batch_31/today/ucrpc21d", O(3944584), "grid maze", "500x500", g_ucrpc21d, "exact"),
    ("scout_batch_12/wossoly21s3", O(4224399), "grid BFS", "200x200", g_wossoly21s3, "exact"),
    ("scout_batch_8/yac1p4", O(4376742), "bitset DP", "n=250", g_yac1p4, "exact"),
    ("scout_batch_46/today/yac6p4", O(5149666), "nim/grundy", "n=2e5, m=100", g_yac6p4, "exact"),
]


# =============================================================================
# Group 2: Ports previously skipped under `no maximum-size input`
# =============================================================================

def g_996warmup8p3(p, big, _):
    rng = random.Random(2001)
    n = 100_000 if big else 200
    kl, kr = (n // 2, n // 2)
    pos = sorted(rng.sample(range(1, 10**9), n))
    dirs = [rng.choice([-1, 1]) for _ in range(n)]
    W(p, [f"{n} {kl} {kr}", J(pos), J(dirs)])


def g_btoi15p2(p, big, _):
    rng = random.Random(2002)
    n = 300_000 if big else 500
    ops = []
    for i in range(n):
        if i < 5 or rng.random() < 0.65:
            ops.append(rng.randint(1, 10**6))
        else:
            ops.append(-rng.randint(1, min(i, 50)))
    W(p, [n] + ops)


def g_btoi17p2(p, big, _):
    rng = random.Random(2003)
    k, n, m, o = (5, 50_000, 100_000, 50_000) if big else (5, 500, 1000, 200)
    layers = n // k
    # BTOI 2017 P2 guarantees at most one road per ordered (a, b) pair
    seen = set()
    edges = []
    while len(edges) < m:
        ly = rng.randint(0, layers - 2)
        a = ly * k + rng.randint(0, k - 1)
        b = (ly + 1) * k + rng.randint(0, k - 1)
        if (a, b) not in seen:
            seen.add((a, b))
            edges.append(f"{a} {b} {rng.randint(1, 10000)}")
    qs = []
    for _ in range(o):
        a = rng.randint(0, n - k - 1)
        b = rng.randint(a + 1, n - 1)
        qs.append(f"{a} {b}")
    W(p, [f"{k} {n} {m} {o}"] + edges + qs)


def g_btoi20p3(p, big, _):
    rng = random.Random(2004)
    n, m, q = (200_000, 200_000, 200_000) if big else (500, 1000, 500)
    edges = [f"{rng.randint(1, n)} {rng.randint(1, n)}" for _ in range(m)]
    qs = []
    for _ in range(q):
        l = rng.randint(1, m)
        r = rng.randint(l, m)
        qs.append(f"{l} {r}")
    W(p, [f"{n} {m} {q}"] + edges + qs)


def g_ccc01s5(p, big, _):
    # Solvable PCP instance (length-4 solution: 2, 1, 1, 3) with m=10
    W(p, [10, 3, "a", "abaaa", "ab", "aaa", "ab", "b"])


def g_ccc19s2(p, big, _):
    rng = random.Random(2006)
    t = 1000 if big else 50
    W(p, [t] + [rng.randint(4, 1_000_000) for _ in range(t)])


def g_ccc22s5(p, big, _):
    rng = random.Random(2007)
    n = 200_000 if big else 500
    edges = [f"{u} {v}" for u, v in tree_edges(rng, n, True, 0.25)]
    init_s = "".join(rng.choice("YN") for _ in range(n))
    cost = [rng.randint(1, 1000) for _ in range(n)]
    W(p, [n] + edges + [init_s, J(cost)])


def g_cco05p4(p, big, _):
    rng = random.Random(2008)
    t = 20 if big else 5
    lines = [t]
    for _ in range(t):
        n = 10_000 if big else 200
        # Even numbers so no length-2 prime sum except rare 1+2, forcing scan
        arr = [rng.randint(2, 5000) * 2 for _ in range(n - 1)] + [3]
        lines.append(f"{n} " + J(arr))
    W(p, lines)


def g_cco16p1(p, big, _):
    rng = random.Random(2009)
    n, m, k = (1_000_000, 800_000, 5) if big else (1000, 800, 5)
    # Disjoint paths and cycles (degree <= 2 everywhere, as required by CCO16P1)
    edges = []
    i = 1
    while i + 1 <= n and len(edges) < m:
        L = rng.randint(2, 15)
        end = min(n, i + L - 1)
        for u in range(i, end):
            if len(edges) < m:
                edges.append(f"{u} {u + 1}")
        if end - i >= 2 and rng.random() < 0.4 and len(edges) < m:
            edges.append(f"{end} {i}")
        i = end + 1
    W(p, [f"{n} {len(edges)} {k}"] + edges)


def g_cco16p5(p, big, _):
    rng = random.Random(2010)
    n, m, k, q = (10**9, 10**9, 2000, 10**8) if big else (1000, 1000, 100, 50)
    z = [f"{rng.randint(1, n)} {rng.randint(1, m)}" for _ in range(k)]
    W(p, [f"{n} {m}", k] + z + [q])


def g_cco18p5(p, big, _):
    rng = random.Random(2011)
    n, k, q = (1_000_000, 100_000, 100_000) if big else (1000, 100, 200)
    a = [rng.randint(1, 10**9) for _ in range(n)]
    upds = [f"{rng.randint(1, n)} {rng.randint(1, 10**9)}" for _ in range(q)]
    W(p, [f"{n} {k} {q}"] + a + upds)


def g_cco19p2(p, big, _):
    rng = random.Random(2012)
    n, m = (1000, 1000) if big else (50, 50)
    rows = ["".join("#" if rng.random() < 0.3 else "." for _ in range(m)) for _ in range(n)]
    W(p, [f"{n} {m}"] + rows)


def g_cco19p6(p, big, _):
    rng = random.Random(2013)
    n, m = (50, 50) if big else (15, 15)
    words = set()
    while len(words) < n:
        L = rng.randint(1, m)
        words.add("".join(rng.choice("01") for _ in range(L)))
    W(p, [f"{n} {m}"] + sorted(words))


def g_cco20p2(p, big, _):
    rng = random.Random(2014)
    n = 200_000 if big else 1000
    # Valid feasible deadlines: perm + small slack
    d = [min(n, i + rng.randint(0, 50)) for i in range(1, n + 1)]
    rng.shuffle(d)
    W(p, [n, J(d)])


def g_cco21p1(p, big, _):
    rng = random.Random(2015)
    n, k, q = (100_000, 100_000, 100_000) if big else (500, 200, 500)
    a = [rng.randint(1, k) for _ in range(n)]
    swaps = [rng.randint(1, k - 1) for _ in range(q)]
    W(p, [f"{n} {k} {q}", J(a)] + swaps)


def g_cco21p4(p, big, _):
    rng = random.Random(2016)
    n, m = (200_000, 200_000) if big else (500, 1000)
    edges = []
    for _ in range(m):
        a = rng.randint(1, n)
        b = rng.randint(1, n)
        edges.append(f"{a} {b} {rng.randint(1, 10**9)} {rng.randint(0, 10**9)}")
    W(p, [f"{n} {m}"] + edges)


def g_cco22p3(p, big, _):
    rng = random.Random(2017)
    # Original uses stack VLA `int dist[n + m + 1][2][2]`, so keep n + m <= 200_000 (< 8 MB stack)
    n, m, k = (100_000, 100_000, 1000) if big else (500, 500, 100)
    def make_disjoint(cnt):
        out = []
        t = 0
        for _ in range(cnt):
            t += rng.randint(1, 500)
            r = t + rng.randint(1, 500)
            out.append(f"{t} {r}")
            t = r
        return out
    W(p, [f"{n} {m} {k}"] + make_disjoint(n) + make_disjoint(m))


def g_ceoi19pp1(p, big, _):
    rng = random.Random(2018)
    n, m = (1500, 1500) if big else (100, 100)
    a, b = [], []
    cur = 0
    for _ in range(n): cur += rng.randint(1, 50); a.append(cur)
    cur = 0
    for _ in range(m): cur += rng.randint(1, 50); b.append(cur)
    W(p, [f"{n} {m}", J(a), J(b)])


def g_ceoi22p1(p, big, _):
    rng = random.Random(2019)
    n, q = (200_000, 1_000_000) if big else (500, 1000)
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    qs = [f"{rng.randint(0, 10**9)} {rng.randint(1, n)}" for _ in range(q)]
    W(p, [f"{n} {q}", J(perm)] + qs)


def g_cheerio1s3(p, big, _):
    rng = random.Random(2020)
    n = 2000 if big else 100
    xs = sorted(rng.sample(range(-10**9, 10**9), n))
    pts = [f"{x} {rng.randint(-10**9, 10**9)}" for x in xs]
    W(p, [n] + pts)


def g_coci20c6p5(p, big, _):
    rng = random.Random(2021)
    n, q = (200_000, 200_000) if big else (500, 500)
    cites = [rng.randint(1, n) for _ in range(n)]
    qs = []
    for _ in range(q):
        l = rng.randint(1, n)
        r = rng.randint(l, n)
        qs.append(f"{l} {r}")
    W(p, [f"{n} {q}", J(cites)] + qs)


def g_coci21c1p4(p, big, _):
    rng = random.Random(2022)
    k = 10 if big else 5
    n = 40_000 if big else 100
    cards = ["".join(rng.choice("123") for _ in range(k)) for _ in range(n)]
    W(p, [f"{n} {k}"] + cards)


def g_coci21c1p5(p, big, _):
    rng = random.Random(2023)
    n = 1_000_000 if big else 1000
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    W(p, [n, J(perm)])


def g_coci21c3p1(p, big, _):
    rng = random.Random(2024)
    n, k = (500, 3) if big else (50, 3)
    a = [rng.randint(1, 20) for _ in range(n)]
    W(p, [f"{n} {k}", J(a)])


def g_coci21c6p2(p, big, _):
    rng = random.Random(2025)
    n, m = (500, 500) if big else (40, 40)
    a, b = 10**6, 2 * 10**6
    rows = [J(rng.randint(1, 1000) for _ in range(m)) for _ in range(n)]
    W(p, [f"{n} {m} {a} {b}"] + rows)


def g_coci25c3p5(p, big, _):
    rng = random.Random(2026)
    n = 200_000 if big else 1000
    a = [rng.randint(1, 10**9) for _ in range(n)]
    b = [rng.randint(1, 10**9) for _ in range(n)]
    W(p, [n, J(a), J(b)])


def g_coi07p1(p, big, _):
    rng = random.Random(2027)
    n = 100_000 if big else 500
    k = 10.5
    xs = sorted(round(rng.uniform(0, 1e7), 3) for _ in range(n))
    W(p, [f"{k:.3f}", n] + [f"{x:.3f}" for x in xs])


def g_dmopc20c6p3(p, big, _):
    rng = random.Random(2028)
    n, m = (500_000, 500_000) if big else (1000, 2000)
    want = [rng.randint(0, 1) for _ in range(n)]
    edges = [f"{rng.randint(1, n)} {rng.randint(1, n)}" for _ in range(m)]
    W(p, [f"{n} {m}", J(want)] + edges)


def g_dmopc20c7p2(p, big, _):
    # Connected cluster of valid square/octagon tiles on the (x+y) parity lattice
    n = 10_000 if big else 200
    tiles = []
    side = int(n ** 0.5)
    for i in range(side):
        for j in range(side):
            tiles.append(f"{2 * i} {2 * j}")
    W(p, [len(tiles)] + tiles)


def g_dmopc21c2p4(p, big, _):
    rng = random.Random(2030)
    n, k = (1_000_000, 10) if big else (1000, 5)
    a = [rng.randint(1, 5) for _ in range(n)]
    W(p, [f"{n} {k}", J(a)])


def g_dmopc21c2p5(p, big, _):
    rng = random.Random(2031)
    # Original statically allocates memo[3005][2][3005]
    n, k = (1500, 750) if big else (100, 30)
    edges = []
    for i in range(2, n + 1):
        p_node = (i - 2) // 3 + 1
        edges.append(f"{p_node} {i}")
    W(p, [f"{n} {k}"] + edges)


def g_dmopc21c4p2(p, big, _):
    rng = random.Random(2032)
    n = 1_000_000 if big else 1000
    a = [rng.randint(1, 5_000_000) for _ in range(n)]
    W(p, [n, J(a)])


def g_dmopc21c7p2(p, big, _):
    rng = random.Random(2033)
    n, q = (1_000_000, 1_000_000) if big else (1000, 500)
    # Valid non-overlapping splice ops: track active order on small pool or simple valid ops
    # Every op moves singleton [x, x] right after k != x (and k != prev[x]), always valid!
    ops = []
    for _ in range(q):
        x = rng.randint(1, n)
        k = rng.randint(0, n)
        if k == x:
            k = (k + 1) % (n + 1)
        ops.append(f"{x} {x} {k}")
    W(p, [f"{n} {q}"] + ops)


def g_dmopc21c8p2(p, big, _):
    rng = random.Random(2034)
    n, m = (1_000_000, 1_000_000) if big else (1000, 500)
    top = [0 if rng.random() < 0.7 else rng.randint(1, m) for _ in range(n)]
    bot = [0 if rng.random() < 0.7 else rng.randint(1, m) for _ in range(n)]
    W(p, [f"{n} {m}", J(top), J(bot)])


def g_dmopc21c8p5(p, big, _):
    rng = random.Random(2035)
    n, k = (1_000_000_000, 1000) if big else (500, 30)
    a = [rng.randint(1, 10**6) for _ in range(k)]
    W(p, [f"{n} {k}", J(a)])


def g_dmopc21c9p2(p, big, _):
    rng = random.Random(2036)
    t = 100 if big else 10
    lines = [t]
    for _ in range(t):
        n = 5000 if big else 100
        a = "".join(rng.choice("abcd") for _ in range(n))
        b = "".join(rng.choice("abcd") for _ in range(n // 2))
        lines.append(f"{a} {b}")
    W(p, lines)


def g_dpr(p, big, _):
    rng = random.Random(2037)
    n, k = (50, 10**18) if big else (20, 10**9)
    rows = [J(rng.randint(0, 1) for _ in range(n)) for _ in range(n)]
    W(p, [f"{n} {k}"] + rows)


def g_dpw(p, big, _):
    rng = random.Random(2038)
    n, m = (200_000, 200_000) if big else (500, 500)
    rules = []
    for _ in range(m):
        l = rng.randint(1, n)
        r = rng.randint(l, n)
        rules.append(f"{l} {r} {rng.randint(-10**9, 10**9)}")
    W(p, [f"{n} {m}"] + rules)


def g_egoi22p1(p, big, _):
    rng = random.Random(2039)
    t = 500 if big else 20
    lines = [t]
    for _ in range(t):
        n = 50 if big else 15
        lines.append(str(n))
        lines.append(J(rng.randint(0, 10) for _ in range(n)))
    W(p, lines)


def g_etsr23r2p2(p, big, _):
    rng = random.Random(2040)
    t = 20 if big else 5
    lines = [t]
    for _ in range(t):
        n = 25 if big else 10
        s = "".join(rng.choice("0123456789") for _ in range(n))
        lines.append(f"{s} {rng.randint(0, 254)}")
    W(p, lines)


def g_etsr23r2p3(p, big, _):
    rng = random.Random(2041)
    # Original reads 32-bit `int a, b, c`
    t = 100_000 if big else 500
    lines = [t] + [f"{rng.randint(1, 10**9)} {rng.randint(1, 10**9)} {rng.randint(1, 10**9)}" for _ in range(t)]
    W(p, lines)


def g_ioi07p3(p, big, _):
    rng = random.Random(2042)
    n = 100_000 if big else 500
    masts = []
    for _ in range(n):
        h = rng.randint(1, 100_000)
        k = rng.randint(1, h)
        masts.append(f"{h} {k}")
    W(p, [n] + masts)


def g_ioi08p2(p, big, _):
    rng = random.Random(2043)
    n = 1_000_000 if big else 2000
    edges = []
    for i in range(1, n + 1):
        to = rng.randint(1, n - 1)
        if to >= i: to += 1
        edges.append(f"{to} {rng.randint(1, 10**8)}")
    W(p, [n] + edges)


def g_ioi09p2(p, big, _):
    rng = random.Random(2044)
    n, w = (500_000, 10**12) if big else (1000, 10**7)
    rows = [f"{rng.randint(1, 20000)} {rng.randint(1, 20000)}" for _ in range(n)]
    W(p, [f"{n} {w}"] + rows)


def g_ioi09p3(p, big, _):
    rng = random.Random(2045)
    n, t, pp = (2000, 2000, 1000) if big else (50, 50, 25)
    rows = [J(rng.randint(0, 1) for _ in range(t)) for _ in range(n)]
    W(p, [f"{n} {t} {pp}"] + rows)


def g_ioi09p6(p, big, _):
    rng = random.Random(2046)
    n, s = (800, 3) if big else (40, 2)
    g = [["T" if rng.random() < 0.2 else "G" for _ in range(n)] for _ in range(n)]
    g[0][0] = "M"
    g[n - 1][n - 1] = "D"
    for _ in range(5):
        r, c = rng.randint(1, n - 2), rng.randint(1, n - 2)
        g[r][c] = "H"
    W(p, [f"{n} {s}"] + ["".join(r) for r in g])


def g_mccc3j4(p, big, _):
    rng = random.Random(2047)
    n, t = (2000, 1000) if big else (100, 50)
    a = [rng.randint(1, n) for _ in range(n)]
    W(p, [f"{n} {t}", J(a)])


def g_mccc4j3(p, big, _):
    rng = random.Random(2048)
    n = 1_000_000 if big else 1000
    a = [rng.randint(1, 1000) for _ in range(n)]
    W(p, [n, J(a)])


def g_mccc5j3(p, big, _):
    rng = random.Random(2049)
    n, k, m = (500_000, 100, 10**6) if big else (1000, 20, 1000)
    a = [rng.randint(1, m) for _ in range(n)]
    b = [rng.randint(1, m) for _ in range(n)]
    W(p, [f"{n} {k} {m}", J(a), J(b)])


def g_mccc6s3(p, big, _):
    rng = random.Random(2050)
    n = 500_000 if big else 1000
    a = [rng.randint(1, n // 4 + 2) for _ in range(n)]
    W(p, [n, J(a)])


def g_nccc9s1(p, big, _):
    rng = random.Random(2051)
    t = 100_000 if big else 500
    lines = [t] + [f"{rng.randint(0, 10**9)} {rng.randint(0, 10**9)} {rng.randint(0, 10**9)}" for _ in range(t)]
    W(p, lines)


def g_noi02p4(p, big, _):
    rng = random.Random(2052)
    n = 15 if big else 8
    # Distinct caves + short lifetimes so a peaceful m exists well below 1e6
    caves = rng.sample(range(1, 101), n)
    lines = [n] + [f"{caves[i]} {rng.randint(1, 100)} {rng.randint(1, 30)}" for i in range(n)]
    W(p, lines)


def g_noi04p1(p, big, _):
    rng = random.Random(2053)
    q, floor = (300_000, 10000) if big else (1000, 100)
    cmds = []
    for _ in range(q):
        r = rng.random()
        if r < 0.4: cmds.append(f"I {rng.randint(5000, 50000)}")
        elif r < 0.6: cmds.append(f"A {rng.randint(0, 100)}")
        elif r < 0.8: cmds.append(f"S {rng.randint(0, 100)}")
        else: cmds.append(f"F {rng.randint(1, 200)}")
    W(p, [f"{q} {floor}"] + cmds)


def g_noi09p2(p, big, _):
    rng = random.Random(2054)
    t = 5 if big else 2
    lines = [t]
    for _ in range(t):
        n, l, pp = (100_000, 30, 3) if big else (200, 20, 3)
        lines.append(f"{n} {l} {pp}")
        for _ in range(n):
            lines.append("a" * rng.randint(1, 15))
    W(p, lines)


def g_noi10p3(p, big, _):
    rng = random.Random(2055)
    n = 500 if big else 30
    lines = [n]
    for _ in range(n + 1): lines.append(J(rng.randint(0, 10000) for _ in range(n)))
    for _ in range(n): lines.append(J(rng.randint(0, 10000) for _ in range(n + 1)))
    for _ in range(n + 1): lines.append(J(rng.randint(0, 10000) for _ in range(n)))
    for _ in range(n): lines.append(J(rng.randint(0, 10000) for _ in range(n + 1)))
    W(p, lines)


def g_noi14p4(p, big, _):
    rng = random.Random(2056)
    t = 5 if big else 2
    lines = [t]
    for _ in range(t):
        n = 1_000_000 if big else 2000
        lines.append("".join(rng.choice("ab") for _ in range(n)))
    W(p, lines)


def g_noi19p3(p, big, _):
    rng = random.Random(2057)
    t = 3 if big else 2
    lines = [t]
    for _ in range(t):
        n = 100_000 if big else 300
        k = n // 2
        l = n // 4
        lines.append(f"{n} {k} {l}")
        lines.append(J(rng.randint(1, 10**9) for _ in range(n)))
        lines.append(J(rng.randint(1, 10**9) for _ in range(n)))
    W(p, lines)


def g_noip09p2(p, big, _):
    rng = random.Random(2058)
    t = 2000 if big else 50
    lines = [t]
    for _ in range(t):
        a1 = rng.randint(1, 500)
        a0 = a1 * rng.randint(1, 5000)
        b0 = rng.randint(1, 5000)
        b1 = b0 * rng.randint(1, 400_000)
        lines.append(f"{a0} {a1} {b0} {b1}")
    W(p, lines)


def g_noip18p2(p, big, _):
    rng = random.Random(2059)
    t = 20 if big else 5
    lines = [t]
    for _ in range(t):
        n = 100 if big else 20
        coins = rng.sample(range(1, 25001), n)
        lines.append(str(n))
        lines.append(J(coins))
    W(p, lines)


def g_noip18p3(p, big, _):
    rng = random.Random(2060)
    n, m = (50_000, 10_000) if big else (500, 100)
    edges = [f"{u} {v} {rng.randint(1, 10000)}" for u, v in tree_edges(rng, n, True, 0.25)]
    W(p, [f"{n} {m}"] + edges)


def g_noip22p4(p, big, _):
    rng = random.Random(2061)
    n, q = (250_000, 250_000) if big else (500, 500)
    a = list(range(1, n + 1)); rng.shuffle(a)
    b = list(range(1, n + 1)); rng.shuffle(b)
    qs = []
    for _ in range(q):
        l = rng.randint(1, n)
        r = rng.randint(l, n)
        qs.append(f"{l} {r}")
    W(p, [f"0 {n}", J(a), J(b), q] + qs)


SPECS += [
    ("scout_batch_32/today/996warmup8p3", O(5324670), "bsearch+sim", "n=1e5", g_996warmup8p3, "exact"),
    ("scout_batch_33/btoi15p2", O(5640898), "binlift", "n=3e5", g_btoi15p2, "exact"),
    ("scout_batch_18/today/btoi17p2", O(4456186), "segtree+mat", "k=5, n=5e4, m=1e5, o=5e4", g_btoi17p2, "exact"),
    ("scout_batch_18/today/btoi20p3", O(4656866), "DnC+DSU", "n=m=q=2e5", g_btoi20p3, "exact"),
    ("scout_batch_22/ccc01s5", O(5025769), "DFS search", "m=40, n=3", g_ccc01s5, "exact"),
    ("scout_batch_29/ccc19s2", O(4325077), "sieve", "t=1000, n<=1e6", g_ccc19s2, "exact"),
    ("scout_batch_18/today/ccc22s5", O(4371349), "tree DP", "n=2e5", g_ccc22s5, "exact"),
    ("scout_batch_16/cco05p4", O(4665767), "prefix+prime", "t=20, n=1e4", g_cco05p4, "exact"),
    ("blind_critic_3/cco16p1", O(4583882), "DSU", "n=1e6, m=8e5", g_cco16p1, "exact"),
    ("scout_batch_24/today/cco16p5", O(4580695), "sweep+segtree", "k=2000", g_cco16p5, "exact"),
    ("scout_batch_24/today/cco18p5", O(4801867), "segtree", "n=1e6, q=1e5", g_cco18p5, "exact"),
    ("scout_batch_27/cco19p2", O(3613676), "grid+dijkstra", "1000x1000", g_cco19p2, "exact"),
    ("scout_batch_22/cco19p6", O(3617414), "string dijkstra", "n=50, m=50", g_cco19p6, "exact"),
    ("scout_batch_2/cco20p2", O(4561352), "fenwick+heap", "n=2e5", g_cco20p2, "exact"),
    ("scout_batch_18/today/cco21p1", O(4538969), "sqrt+fenwick", "n=k=q=1e5", g_cco21p1, "exact"),
    ("scout_batch_18/today/cco21p4", O(6419583), "graph peel", "n=m=2e5", g_cco21p4, "exact"),
    ("scout_batch_30/cco22p3", O(4801968), "DP+bsearch", "n=m=3e5", g_cco22p3, "exact"),
    ("scout_batch_48/today/ceoi19pp1", O(5343174), "hashmap", "n=m=1500", g_ceoi19pp1, "exact"),
    ("scout_batch_24/today/ceoi22p1", O(4728849), "fenwick+perm", "n=2e5, q=1e6", g_ceoi22p1, "exact"),
    ("cheerio1s3", P("cheerio1s3_orig.cpp"), "geometry", "n=2000", g_cheerio1s3, "exact"),
    ("scout_batch_7/coci20c6p5", O(4188128), "Mo's algo", "n=q=2e5", g_coci20c6p5, "exact"),
    ("scout_batch_35/today/coci21c1p4", O(4106618), "FWHT base 3", "n=4e4, k=10", g_coci21c1p4, "exact"),
    ("scout_batch_35/today/coci21c1p5", O(4086660), "LIS+DAG", "n=1e6", g_coci21c1p5, "exact"),
    ("scout_batch_35/today/coci21c3p1", O(4196130), "bruteforce", "n=500, k=3", g_coci21c3p1, "exact"),
    ("scout_batch_35/today/coci21c6p2", O(4529998), "2D two-ptr", "500x500", g_coci21c6p2, "exact"),
    ("scout_batch_29/coci25c3p5", O(7730259), "math+gcd", "n=2e5", g_coci25c3p5, "exact"),
    ("scout_batch_35/coi07p1", O(5323995), "greedy float", "n=1e5", g_coi07p1, "float"),
    ("scout_batch_34/today/dmopc20c6p3", O(3917634), "retrograde BFS", "n=m=5e5", g_dmopc20c6p3, "exact"),
    ("scout_batch_34/today/dmopc20c7p2", O(3686612), "geometry+map", "n=1e4", g_dmopc20c7p2, "exact"),
    ("scout_batch_34/today/dmopc21c2p4", O(3949962), "DP+runs", "n=1e6, k=10", g_dmopc21c2p4, "exact"),
    ("scout_batch_34/today/dmopc21c2p5", O(3968384), "tree+mod", "n=2e5", g_dmopc21c2p5, "exact"),
    ("scout_batch_44/today/dmopc21c4p2", O(4125290), "sieve+divisors", "n=1e6", g_dmopc21c4p2, "exact"),
    ("scout_batch_44/today/dmopc21c7p2", O(4452915), "linked list", "n=q=1e6", g_dmopc21c7p2, "exact"),
    ("scout_batch_44/today/dmopc21c8p2", O(4522631), "diff array+mod", "n=m=1e6", g_dmopc21c8p2, "exact"),
    ("scout_batch_34/today/dmopc21c8p5", O(4533533), "knapsack+greedy", "n=1e9, k=1000", g_dmopc21c8p5, "exact"),
    ("scout_batch_44/today/dmopc21c9p2", O(4563973), "deque", "t=100, n=5000", g_dmopc21c9p2, "exact"),
    ("wave2/dpr", O(4131862), "matrix exp", "n=50, k=1e18", g_dpr, "exact"),
    ("scout_batch_18/today/dpw", O(4133460), "lazy segtree", "n=m=2e5", g_dpw, "exact"),
    ("scout_batch_48/today/egoi22p1", O(5198974), "greedy", "t=500, n=50", g_egoi22p1, "exact"),
    ("scout_batch_43/etsr23r2p2", O(5494433), "interval DP", "t=20, n=25", g_etsr23r2p2, "exact"),
    ("scout_batch_43/etsr23r2p3", O(5494453), "bit ops", "t=1e5", g_etsr23r2p3, "exact"),
    ("scout_batch_24/today/ioi07p3", O(5120005), "lazy segtree", "n=1e5", g_ioi07p3, "exact"),
    ("scout_batch_2/ioi08p2", O(5107842), "functional graph", "n=1e6", g_ioi08p2, "exact"),
    ("scout_batch_29/ioi09p2", O(5102705), "sort+heap", "n=5e5", g_ioi09p2, "head1"),
    ("scout_batch_25/today/ioi09p3", O(5106576), "matrix+sort", "2000x2000", g_ioi09p3, "exact"),
    ("blind_critic_3/ioi09p6", O(5106126), "grid BFS+bs", "800x800", g_ioi09p6, "exact"),
    ("scout_batch_42/today/mccc3j4", O(4223440), "sim+queues", "n=2000, t=1000", g_mccc3j4, "exact"),
    ("scout_batch_42/today/mccc4j3", O(4264907), "prefix sum", "n=1e6", g_mccc4j3, "exact"),
    ("scout_batch_42/today/mccc5j3", O(5333116), "map+bsearch", "n=5e5", g_mccc5j3, "exact"),
    ("scout_batch_42/today/mccc6s3", O(6103125), "DSU jump", "n=5e5", g_mccc6s3, "exact"),
    ("scout_batch_41/nccc9s1", O(4051682), "math", "t=1e5", g_nccc9s1, "exact"),
    ("scout_batch_20/noi02p4", O(3741279), "ext_gcd", "n=15", g_noi02p4, "exact"),
    ("scout_batch_7/noi04p1", O(3738402), "fenwick+compress", "q=3e5", g_noi04p1, "exact"),
    ("scout_batch_22/noi09p2", O(5108899), "convex DP", "t=5, n=1e5", g_noi09p2, "exact"),
    ("scout_batch_27/noi10p3", O(5672180), "planar dual dijkstra", "500x500", g_noi10p3, "exact"),
    ("noi14p4", P("noi14p4_orig.cpp"), "KMP border", "t=5, n=1e6", g_noi14p4, "exact"),
    ("scout_batch_52/noi19p3", O(5669058), "greedy+heaps", "t=3, n=1e5", g_noi19p3, "exact"),
    ("noip09p2", P("noip09p2_orig.cpp"), "divisors+gcd", "t=2000", g_noip09p2, "exact"),
    ("wave2/noip18p2", O(5032734), "knapsack", "t=20, n=100", g_noip18p2, "exact"),
    ("scout_batch_2/noip18p3", O(4002963), "tree+bsearch+set", "n=5e4", g_noip18p3, "exact"),
    ("scout_batch_24/today/noip22p4", O(5445685), "lazy segtree", "n=q=2.5e5", g_noip22p4, "exact"),
]


# =============================================================================
# Group 3: Ports previously skipped under `needs a hand-written generator`
# =============================================================================

def g_aac4p2(p, big, _):
    rng = random.Random(3001)
    n, q = (200_000, 200_000) if big else (500, 500)
    a = [rng.choice([1, 2, 3, 4, 5, 6, 12]) for _ in range(n)]
    qs = [rng.randint(1, 10**9) for _ in range(q)]
    W(p, [f"{n} {q}", J(a)] + qs)


def g_aac5p3(p, big, _):
    rng = random.Random(3002)
    n, m = (200_000, 200_000) if big else (500, 500)
    edges = [f"{u} {v} {rng.randint(1, 10**9)}" for u, v in tree_edges(rng, n, True, 0.2)]
    xs = [rng.randint(1, 10**9) for _ in range(m)]
    W(p, [f"{n} {m}"] + edges + [J(xs)])


def g_acc7p3(p, big, _):
    rng = random.Random(3003)
    n, m = (100_000, 100_000) if big else (500, 500)
    a = [rng.randint(1, 10**9) for _ in range(n)]
    edges = [f"{rng.randint(1, n)} {rng.randint(1, n)}" for _ in range(m)]
    W(p, [f"{n} {m}", J(a)] + edges)


def g_apio07p3(p, big, _):
    rng = random.Random(3004)
    n, c = (10_000, 50_000) if big else (200, 500)
    lines = [f"{n} {c}"]
    for _ in range(c):
        e = rng.randint(1, n)
        view = [((e - 1 + d) % n) + 1 for d in range(5)]
        f = rng.randint(0, 2)
        l = rng.randint(0, 2)
        fears = rng.sample(view, f)
        loves = rng.sample(view, l)
        lines.append(f"{e} {f} {l} " + J(fears + loves))
    W(p, lines)


def g_btoi16p1(p, big, _):
    rng = random.Random(3005)
    n = 5000 if big else 100
    lines = [n]
    for i in range(1, n + 1):
        k = rng.randint(1, min(10, n - 1))
        bosses = set()
        if i > 1:
            bosses.add(i - 1)
        while len(bosses) < k:
            b = rng.randint(1, n)
            if b != i:
                bosses.add(b)
        lines.append(f"{len(bosses)} " + J(sorted(bosses)))
    W(p, lines)


def g_btoi17p3(p, big, _):
    rng = random.Random(3006)
    n, m, k = (100_000, 50_000, 5) if big else (500, 200, 3)
    edges = [f"{u} {v}" for u, v in tree_edges(rng, n, True, 0.2)]
    plans = []
    for _ in range(m):
        s = rng.randint(2, 6)
        cities = rng.sample(range(1, n + 1), s)
        plans.append(f"{s} " + J(cities))
    W(p, [f"{n} {m} {k}"] + edges + plans)


def g_btoi17p4(p, big, _):
    rng = random.Random(3007)
    n, pp, q = (2500, 10, 5) if big else (100, 5, 3)
    adj = [set() for _ in range(n)]
    for i in range(n):
        for d in range(1, 4):
            j = (i + d) % n
            if len(adj[i]) < pp + q - 2 and len(adj[j]) < pp + q - 2:
                adj[i].add(j); adj[j].add(i)
    lines = [f"{n} {pp} {q}"] + [f"{len(a)} " + J(sorted(a)) for a in adj]
    W(p, lines)


def g_btoi17p5(p, big, _):
    rng = random.Random(3008)
    n, m, k = (10**9, 10**9, 100_000) if big else (1000, 1000, 200)
    lines = [f"{n} {m} {k}"]
    seen = set()
    for _ in range(k):
        r, c = rng.randint(1, n), rng.randint(1, m)
        while (r, c) in seen:
            r, c = rng.randint(1, n), rng.randint(1, m)
        seen.add((r, c))
        # Consistent checkerboard parity so answer is nonzero
        ch = "+" if (r + c) % 2 == 0 else "-"
        lines.append(f"{ch} {r} {c}")
    W(p, lines)


def g_btoi18p1(p, big, _):
    rng = random.Random(3009)
    n = 100_000 if big else 500
    perm = list(range(n))
    rng.shuffle(perm)
    lines = [n] + [f"u{i} u{perm[i]}" for i in range(n)]
    W(p, lines)


def g_btoi18p4(p, big, _):
    rng = random.Random(3010)
    n, m = (100_000, 100_000) if big else (500, 500)
    lines = [f"{n} {m}"]
    for _ in range(m):
        a = rng.randint(1, n)
        b = rng.randint(1, n)
        if a == b: b = (b % n) + 1
        lines.append(f"{a} {b}")
    W(p, lines)


def g_ccc02s4hard(p, big, _):
    rng = random.Random(3011)
    m, n = (100_000, 100_000) if big else (20, 200)
    lines = [m, n]
    for i in range(n):
        lines.append(f"p{i}")
        lines.append(rng.randint(1, 10000))
    W(p, lines)


def g_ccc03s4(p, big, _):
    rng = random.Random(3012)
    t = 10 if big else 3
    lines = [t] + [letters(rng, 5000 if big else 200, "abc") for _ in range(t)]
    W(p, lines)


def g_ccc04s5(p, big, _):
    rng = random.Random(3013)
    cases = 10 if big else 3
    lines = []
    for _ in range(cases):
        m, n = (100, 100) if big else (20, 20)
        lines.append(f"{m} {n}")
        for r in range(m):
            row = [rng.choice("....*123456789") for _ in range(n)]
            if r == m - 1:
                row[0] = "."; row[n - 1] = "."
            lines.append("".join(row))
    lines.append("0 0")
    W(p, lines)


def g_ccc14s4(p, big, _):
    rng = random.Random(3014)
    n, t = (1000, 15) if big else (100, 5)
    lines = [n, t]
    for _ in range(n):
        x1 = rng.randint(0, 10**6)
        y1 = rng.randint(0, 10**6)
        x2 = x1 + rng.randint(1, 10**5)
        y2 = y1 + rng.randint(1, 10**5)
        lines.append(f"{x1} {y1} {x2} {y2} {rng.randint(1, 10)}")
    W(p, lines)


def g_ccc18s2(p, big, _):
    n = 100 if big else 20
    # Valid strictly increasing grid rotated 90 degrees
    g = [[i * 200 + j for j in range(n)] for i in range(n)]
    rot = [[g[n - 1 - j][i] for j in range(n)] for i in range(n)]
    W(p, [n] + [J(row) for row in rot])


def g_ccc19s5(p, big, _):
    rng = random.Random(3016)
    n, k = (3000, 1500) if big else (100, 30)
    lines = [f"{n} {k}"] + [J(rng.randint(1, 10**9) for _ in range(i)) for i in range(1, n + 1)]
    W(p, lines)


def g_ccc20s1(p, big, _):
    rng = random.Random(3017)
    n = 100_000 if big else 500
    ts = rng.sample(range(0, 10**9), n)
    lines = [n] + [f"{t} {rng.randint(-10**9, 10**9)}" for t in ts]
    W(p, lines)


def g_ccc20s2hard(p, big, _):
    rng = random.Random(3018)
    n, m = (1500, 1500) if big else (50, 50)
    lim = n * m
    rows = [J(rng.randint(1, lim) for _ in range(m)) for _ in range(n)]
    W(p, [n, m] + rows)


def g_ccc22s2(p, big, _):
    rng = random.Random(3019)
    g = 100_000 if big else 500
    x, y = (100_000, 100_000) if big else (500, 500)
    names = [f"s{i}" for i in range(3 * g)]
    xs = [f"{names[rng.randint(0, 3*g-1)]} {names[rng.randint(0, 3*g-1)]}" for _ in range(x)]
    ys = [f"{names[rng.randint(0, 3*g-1)]} {names[rng.randint(0, 3*g-1)]}" for _ in range(y)]
    gs = [f"{names[3*i]} {names[3*i+1]} {names[3*i+2]}" for i in range(g)]
    W(p, [x] + xs + [y] + ys + [g] + gs)


def g_ccc23s1(p, big, _):
    rng = random.Random(3020)
    n = 200_000 if big else 1000
    W(p, [n, J(rng.randint(0, 1) for _ in range(n)), J(rng.randint(0, 1) for _ in range(n))])


def g_ccc23s2(p, big, _):
    rng = random.Random(3021)
    n = 5000 if big else 200
    W(p, [n, J(rng.randint(0, 10**5) for _ in range(n))])


def g_ccc96s3(p, big, _):
    W(p, [3, "18 4", "16 5", "15 6"])


def g_cco03p4(p, big, _):
    rng = random.Random(3023)
    n = 15 if big else 9
    # Acyclic constraints x < y so valid permutations exist
    pairs = [(i, j) for i in range(1, n) for j in range(i + 1, n + 1)]
    rng.shuffle(pairs)
    chosen = pairs[:min(len(pairs), 20)]
    W(p, [n, len(chosen)] + [f"{u} {v}" for u, v in chosen])


def g_cco04p1(p, big, _):
    rng = random.Random(3024)
    lines = [26] + [f"{chr(ord('a') + i)} {rng.randint(1, 100)} {rng.randint(1, 5)}" for i in range(26)]
    n = 100_000 if big else 500
    lines.append(n)
    for _ in range(n):
        lines.append("".join(rng.choice("abcdefghijkl") for _ in range(rng.randint(3, 12))))
    W(p, lines)


def g_cco05p6(p, big, _):
    rng = random.Random(3025)
    n, q = (10_000, 10_000) if big else (200, 200)
    lines = [f"10000 10000", n]
    for _ in range(n):
        x1 = rng.randint(0, 9000)
        y1 = rng.randint(0, 9000)
        x2 = x1 + rng.randint(10, 1000)
        y2 = y1 + rng.randint(10, 1000)
        lines.append(f"{x1} {y2} {x2} {y1}")
    lines.append(q)
    for _ in range(q):
        lines.append(f"{rng.randint(0, 10000)} {rng.randint(0, 10000)}")
    W(p, lines)


def g_cco17p3(p, big, _):
    rng = random.Random(3026)
    n, q = (200_000, 200_000) if big else (500, 500)
    strokes = [f"{rng.randint(1, 10**18)} {rng.randint(1, 10**18)} {rng.randint(1, 10**6)}" for _ in range(n)]
    qs = [f"{rng.randint(1, 10**18)} {rng.randint(1, 10**18)}" for _ in range(q)]
    W(p, [f"{n} {q}"] + strokes + qs)


def g_cco18p6(p, big, _):
    rng = random.Random(3027)
    n = 4096 if big else 128
    a = list(range(1, n + 1)); rng.shuffle(a)
    b = list(range(1, n + 1)); rng.shuffle(b)
    W(p, [n, J(a), J(b)])


def g_cco19p4(p, big, _):
    rng = random.Random(3028)
    k, n = (3, 500_000) if big else (3, 1000)
    a = [rng.randint(1, min(n, 2000)) for _ in range(n)]
    W(p, [k, n] + a)


def g_cco20p4(p, big, _):
    rng = random.Random(3029)
    n = 1000 if big else 50
    lines = [n] + ["".join(rng.choice("RB") for _ in range(i)) for i in range(1, n)]
    W(p, lines)


def g_cco20p5(p, big, _):
    rng = random.Random(3030)
    q = 200_000 if big else 500
    active = []
    lines = [q]
    for _ in range(q):
        if not active or (len(active) < 50000 and rng.random() < 0.65):
            l = rng.randint(1, 500_000)
            r = rng.randint(l, min(500_000, l + 50_000))
            active.append((l, r))
            lines.append(f"A {l} {r}")
        else:
            idx = rng.randrange(len(active))
            l, r = active.pop(idx)
            lines.append(f"R {l} {r}")
    W(p, lines)


def g_cco21p5(p, big, _):
    rng = random.Random(3031)
    n, m = (200_000, 200_000) if big else (500, 500)
    edges = []
    for _ in range(m):
        u = rng.randint(1, n)
        v = rng.randint(1, n - 1)
        if v >= u: v += 1
        edges.append(f"{u} {v}")
    W(p, [f"{n} {m}"] + edges)


def g_cco22p4(p, big, _):
    rng = random.Random(3032)
    n, m = (2000, 2000) if big else (100, 100)
    # CCO 2022 P4 input order: n m, then m single node ids (1..n), then n lines of (left, right) children
    ch = [[] for _ in range(n + 1)]
    for i in range(2, n + 1):
        ch[i // 2].append(i)
    lines = [f"{n} {m}"]
    lines.append(J(rng.randint(1, n) for _ in range(m)))
    for i in range(1, n + 1):
        l = ch[i][0] if len(ch[i]) > 0 else 0
        r = ch[i][1] if len(ch[i]) > 1 else 0
        lines.append(f"{l} {r}")
    W(p, lines)


def g_cco22p5(p, big, _):
    rng = random.Random(3033)
    n, ma, mb, k = (100_000, 150_000, 150_000, 10**9) if big else (300, 500, 500, 10**6)
    def mk_edges(cnt):
        es = [f"{u} {v} {rng.randint(1, 10**6)}" for u, v in tree_edges(rng, n, True, 0.2)]
        while len(es) < cnt:
            es.append(f"{rng.randint(1, n)} {rng.randint(1, n)} {rng.randint(1, 10**6)}")
        return es
    W(p, [f"{n} {ma} {mb} {k}"] + mk_edges(ma) + mk_edges(mb))


def g_cco23p5(p, big, _):
    rng = random.Random(3034)
    n, k = (200_000, 3) if big else (500, 3)
    edges = [f"{u} {v}" for u, v in tree_edges(rng, n, True, 0.25)]
    w = [rng.randint(1, 10**9) for _ in range(n)]
    W(p, [f"{n} {k}"] + edges + [J(w)])


def g_cco24p2(p, big, _):
    rng = random.Random(3035)
    n = 1_000_000 if big else 1000
    a = [rng.randint(1, n) for _ in range(n)]
    b = list(a); rng.shuffle(b)
    W(p, [n, J(a), J(b)])


def g_cco24p5(p, big, _):
    rng = random.Random(3036)
    n = 500_000 if big else 1000
    a = [rng.randint(1, n) for _ in range(n)]
    W(p, [n, J(a)])


def g_ceoi19pp2(p, big, _):
    rng = random.Random(3037)
    q = 500_000 if big else 1000
    W(p, [q] + [rng.randint(0, 10**9 - 1) for _ in range(q)])


def g_ceoi22p5(p, big, _):
    rng = random.Random(3038)
    n, m, d = (200_000, 200_000, 1000) if big else (500, 500, 50)
    W(p, [f"{n} {m} {d}", J(rng.randint(1, 10**9) for _ in range(n)), J(rng.randint(1, 10**9) for _ in range(m))])


def g_coci21c1p1(p, big, _):
    rng = random.Random(3039)
    n = 100_000 if big else 500
    t = 0
    lines = [n]
    for _ in range(n):
        t += rng.randint(1, 15)
        a = rng.randint(1, 4)
        b = rng.randint(5, 8)
        if rng.random() < 0.5: a, b = b, a
        lines.append(f"{t} {a} {b}")
    W(p, lines)


def g_coci21c1p3(p, big, _):
    rng = random.Random(3040)
    n = 100_000 if big else 500
    # Connected unicyclic graph: tree + 1 extra edge
    es = tree_edges(rng, n, True, 0.25)
    es.append((1, n))
    W(p, [n] + [f"{u} {v}" for u, v in es])


def g_coci21c2p2(p, big, _):
    rng = random.Random(3041)
    n, m, q = (1000, 1000, 100_000) if big else (100, 50, 500)
    lines = [f"{n} {m} {q}"]
    for _ in range(m):
        perm = list(range(1, n + 1)); rng.shuffle(perm)
        lines.append(J(perm))
    for _ in range(q):
        lines.append(f"{rng.randint(1, n)} {rng.randint(1, n)}")
    W(p, lines)


def g_coci21c3p2(p, big, _):
    rng = random.Random(3042)
    n = 100_000 if big else 500
    # Forest of ~50 trees
    es = []
    block = max(2, n // 50)
    for start in range(1, n + 1, block):
        end = min(n, start + block - 1)
        for v in range(start + 1, end + 1):
            es.append(f"{rng.randint(start, v - 1)} {v}")
    W(p, [f"{n} {len(es)}"] + es)


def g_coci21c6p1(p, big, _):
    rng = random.Random(3043)
    n = 1000 if big else 50
    lines = [n] + [f"u{i} " + J(rng.randint(0, 100) for _ in range(5)) for i in range(n)]
    W(p, lines)


def g_coci21c6p3(p, big, _):
    rng = random.Random(3044)
    n, m = (200_000, 200_000) if big else (500, 1000)
    edges = []
    for _ in range(m):
        u = rng.randint(1, n - 1)
        v = rng.randint(u + 1, n)
        edges.append(f"{u} {v}")
    W(p, [f"{n} {m}"] + edges)


def g_coi06p2(p, big, _):
    rng = random.Random(3045)
    n, m, q = (100_000, 250_000, 200_000) if big else (500, 1000, 500)
    es = tree_edges(rng, n, True, 0.2)
    extra = [(rng.randint(1, n), rng.randint(1, n)) for _ in range(m - len(es))]
    all_e = [(u, v) for u, v in es + extra if u != v]
    qs = []
    for _ in range(q):
        a, b = rng.randint(1, n), rng.randint(1, n)
        if rng.random() < 0.5:
            u, v = all_e[rng.randrange(len(all_e))]
            qs.append(f"1 {a} {b} {u} {v}")
        else:
            c = rng.randint(1, n)
            while c == a or c == b:
                c = (c % n) + 1
            qs.append(f"2 {a} {b} {c}")
    W(p, [f"{n} {len(all_e)}"] + [f"{u} {v}" for u, v in all_e] + [q] + qs)


def g_dmopc20c4p4(p, big, _):
    rng = random.Random(3046)
    n = 1_000_000 if big else 1000
    a = [rng.randint(0, i) for i in range(n)]
    W(p, [n, J(a)])


def g_dmopc20c7p4(p, big, _):
    rng = random.Random(3047)
    n, q = (200_000, 200_000) if big else (500, 500)
    es = [f"{u} {v}" for u, v in tree_edges(rng, n, True, 0.25)]
    qs = []
    for _ in range(q):
        if rng.random() < 0.4:
            qs.append(f"1 {rng.randint(1, n)}")
        else:
            qs.append(f"2 {rng.randint(1, n)} {rng.randint(1, n)}")
    W(p, [f"{n} {q}"] + es + qs)


def g_dmopc20c7p5(p, big, _):
    rng = random.Random(3048)
    n, m = (500, 2000) if big else (50, 150)
    cost = [rng.randint(1, 1000) for _ in range(n)]
    pairs = []
    for _ in range(m):
        u = rng.randint(1, n)
        v = rng.randint(1, n - 1)
        if v >= u: v += 1
        pairs.append(f"{u} {v} {rng.randint(1, 1000)}")
    W(p, [f"{n} {m}", J(cost)] + pairs)


def g_dmopc21c2p2(p, big, _):
    rng = random.Random(3049)
    n, q = (300, 300_000) if big else (30, 500)
    lines = [f"{n} {q}"]
    for _ in range(q):
        r = rng.random()
        if r < 0.4:
            lines.append(f"B {rng.randint(1, n)} {rng.randint(1, n)}")
        elif r < 0.8:
            lines.append(f"E {rng.randint(1, n)} {rng.randint(1, n)}")
        else:
            perm = list(range(1, n + 1)); rng.shuffle(perm)
            lines.append("Q " + J(perm))
    W(p, lines)


def g_dmopc21c5p3(p, big, _):
    rng = random.Random(3050)
    t = 100 if big else 10
    W(p, [t] + [rng.randint(1, 10_000) for _ in range(t)])


def g_dmopc21c5p4(p, big, _):
    rng = random.Random(3051)
    t = 200 if big else 10
    W(p, [t] + [rng.randint(1, 5000) for _ in range(t)])


def g_dmopc21c9p3(p, big, _):
    rng = random.Random(3052)
    n, m, q = (3000, 3000, 100_000) if big else (100, 200, 500)
    es = [f"{u} {v}" for u, v in tree_edges(rng, n, True, 0.2)]
    while len(es) < m:
        es.append(f"{rng.randint(1, n)} {rng.randint(1, n)}")
    qs = [f"{rng.randint(1, n)} {rng.randint(1, n)} {rng.randint(1, 100)} {rng.randint(1, n)} {rng.randint(1, n)} {rng.randint(1, 100)}" for _ in range(q)]
    W(p, [f"{n} {m}"] + es + [q] + qs)


def g_dmpg18s5(p, big, _):
    rng = random.Random(3053)
    n, q = (200_000, 200_000) if big else (500, 500)
    a = [rng.randint(1, 200_000) for _ in range(n)]
    qs = []
    for _ in range(q):
        if rng.random() < 0.7:
            l = rng.randint(1, n); r = rng.randint(l, n)
            qs.append(f"1 {l} {r} {rng.randint(1, 500)}")
        else:
            qs.append(f"2 {rng.randint(1, n)} {rng.randint(1, 200_000)}")
    W(p, [f"{n} {q}", J(a)] + qs)


def g_dpi(p, big, _):
    rng = random.Random(3054)
    n = 2999 if big else 99
    W(p, [n, J(f"{rng.uniform(0.1, 0.9):.4f}" for _ in range(n))])


def g_dwite10c3p3(p, big, _):
    W(p, [30, 28, 26, 24, 22])


def g_egoi22p5(p, big, _):
    rng = random.Random(3056)
    n, q = (100_000, 100_000) if big else (500, 500)
    a = [rng.randint(1, 10**6) for _ in range(n)]
    qs = [f"{rng.randint(1, 10**6)} {rng.randint(1, n)}" for _ in range(q)]
    W(p, [f"{n} {q}", J(a)] + qs)


def g_egoi23p1(p, big, _):
    rng = random.Random(3057)
    n, q = (300_000, 300_000) if big else (500, 500)
    p0 = [rng.randint(1, 10**6) for _ in range(n)]
    qs = []
    for _ in range(q):
        if rng.random() < 0.5:
            qs.append(f"INFLATION {rng.randint(1, 1000)}")
        else:
            qs.append(f"SET {rng.randint(1, 10**6)} {rng.randint(1, 10**6)}")
    W(p, [n, J(p0), q] + qs)


def g_egoi23p7(p, big, _):
    rng = random.Random(3058)
    n, m1, m2 = (200_000, 200_000, 200_000) if big else (500, 500, 500)
    def dag_es(cnt):
        return [f"{rng.randint(1, n - 1)} {rng.randint(0, n - 2)}" for _ in range(cnt)]
    # Ensure child > parent so it's a DAG
    e1 = []
    for _ in range(m1):
        par = rng.randint(0, n - 2)
        ch = rng.randint(par + 1, n - 1)
        e1.append(f"{ch} {par}")
    e2 = []
    for _ in range(m2):
        par = rng.randint(0, n - 2)
        ch = rng.randint(par + 1, n - 1)
        e2.append(f"{ch} {par}")
    W(p, [f"{n} {m1} {m2}"] + e1 + e2)


def g_etsr23r1p3(p, big, _):
    rng = random.Random(3059)
    t = 100 if big else 5
    lines = [t]
    for _ in range(t):
        m = 26
        n = 5000 if big else 100
        lines.append(f"{m} " + "".join(chr(ord("a") + rng.randint(0, m - 1)) for _ in range(n)))
    W(p, lines)


def g_etsr23r1p4(p, big, _):
    rng = random.Random(3060)
    n, q = (200_000, 200_000) if big else (500, 500)
    a = [rng.randint(1, 10**9) for _ in range(n)]
    qs = []
    for _ in range(q):
        l = rng.randint(1, n); r = rng.randint(l, n)
        ty = rng.randint(1, 3)
        if ty == 3: qs.append(f"3 {l} {r}")
        else: qs.append(f"{ty} {l} {r} {rng.randint(1, 10**9)}")
    W(p, [f"{n} {q}", J(a)] + qs)


def g_etsr23r2p4(p, big, _):
    rng = random.Random(3061)
    t = 20 if big else 5
    lines = [t]
    for _ in range(t):
        n = 10_000 if big else 200
        lines.append(str(n))
        for _ in range(n):
            lines.append(f"{rng.randint(1, 10**6)} {rng.randint(1, 10**6)}")
    W(p, lines)


def g_etsr23r4p1(p, big, _):
    rng = random.Random(3062)
    n = 300_000 if big else 1000
    W(p, [n, J(rng.randint(1, 10**9) for _ in range(n))])


def g_etsr23r4p3(p, big, _):
    rng = random.Random(3063)
    n, m = (100_000, 100_000) if big else (500, 500)
    t_es = [f"{u} {v}" for u, v in tree_edges(rng, n, True, 0.25)]
    q_es = [f"{rng.randint(1, n)} {rng.randint(1, n)}" for _ in range(m)]
    W(p, [f"{n} {m}"] + t_es + q_es)


def g_ioi01p1(p, big, _):
    rng = random.Random(3064)
    n = 1024 if big else 64
    q = 100_000 if big else 500
    lines = [f"0 {n}"]
    for _ in range(q):
        if rng.random() < 0.5:
            lines.append(f"1 {rng.randint(0, n - 1)} {rng.randint(0, n - 1)} {rng.randint(1, 1000)}")
        else:
            x1, x2 = sorted([rng.randint(0, n - 1), rng.randint(0, n - 1)])
            y1, y2 = sorted([rng.randint(0, n - 1), rng.randint(0, n - 1)])
            lines.append(f"2 {x1} {y1} {x2} {y2}")
    lines.append("3")
    W(p, lines)


def g_ioi04p2hard(p, big, _):
    rng = random.Random(3065)
    n = 200_000 if big else 500
    lines = [n] + [f"{rng.randint(-10**9, 10**9)} {rng.randint(-10**9, 10**9)}" for _ in range(n)]
    W(p, lines)


def g_ioi05p2(p, big, _):
    rng = random.Random(3066)
    n = 10**9 if big else 1000
    q = 100_000 if big else 500
    lines = [n]
    for _ in range(q):
        if rng.random() < 0.5:
            a = rng.randint(1, n)
            b = rng.randint(a, n)
            lines.append(f"I {a} {b} {rng.randint(-1000, 1000)}")
        else:
            lines.append(f"Q {rng.randint(0, 10**9)}")
    lines.append("E")
    W(p, lines)


def g_noi03p4(p, big, _):
    rng = random.Random(3067)
    n = 200_000 if big else 500
    es = [f"{u} {v} {rng.randint(1, 10**6)}" for u, v in tree_edges(rng, n, True, 0.25)]
    W(p, [f"{n} {n - 1}"] + es)


def g_noi14p5(p, big, _):
    n, m = (5000, 5000) if big else (50, 50)
    q = 50_000 if big else 100
    rng = random.Random(3068)
    swaps = [f"{rng.randint(1, n * m)} {rng.randint(1, n * m)}" for _ in range(q)]
    W(p, [f"1 3 5 7 100000007", f"{n} {m} {q}"] + swaps)


def g_noi15p1(p, big, _):
    rng = random.Random(3069)
    t = 10 if big else 3
    lines = [t]
    for _ in range(t):
        q = 100_000 if big else 500
        lines.append(str(q))
        for _ in range(q):
            lines.append(f"{rng.randint(1, 10**6)} {rng.randint(1, 10**6)} {rng.randint(0, 1)}")
    W(p, lines)


def g_noi15p2(p, big, _):
    rng = random.Random(3070)
    n, q = (100_000, 100_000) if big else (500, 500)
    pars = [rng.randint(0, i - 1) for i in range(1, n)]
    qs = [f"{rng.choice(['install', 'uninstall'])} {rng.randint(0, n - 1)}" for _ in range(q)]
    W(p, [n, J(pars), q] + qs)


def g_noi17p1(p, big, _):
    rng = random.Random(3071)
    q = 300_000 if big else 1000
    # NOI 2017 P1 guarantees x >= 0 at all times; seed a high bit so random +/- never goes negative
    lines = [f"{q} 0 0 0", f"1 1000000000 100000"]
    for _ in range(q - 1):
        if rng.random() < 0.6:
            lines.append(f"1 {rng.randint(-10**9, 10**9)} {rng.randint(0, 99_900)}")
        else:
            lines.append(f"2 {rng.randint(0, 100_050)}")
    W(p, lines)


def g_noi18p1(p, big, _):
    rng = random.Random(3072)
    t = 2 if big else 1
    lines = [t]
    for _ in range(t):
        n, m, q = (100_000, 200_000, 200_000) if big else (300, 600, 300)
        es = [f"{u} {v} {rng.randint(1, 10000)} {rng.randint(1, 10**9)}" for u, v in tree_edges(rng, n, True, 0.2)]
        while len(es) < m:
            es.append(f"{rng.randint(1, n)} {rng.randint(1, n)} {rng.randint(1, 10000)} {rng.randint(1, 10**9)}")
        qs = [f"{rng.randint(1, n)} {rng.randint(0, 10**9)}" for _ in range(q)]
        lines += [f"{n} {m}"] + es + [f"{q} 1 1000000000"] + qs
    W(p, lines)


def g_noi21p5(p, big, _):
    rng = random.Random(3073)
    n, q = (100_000, 100_000) if big else (200, 200)
    s = "".join(rng.choice("WE") for _ in range(n))
    cur_len = n
    qs = []
    for _ in range(q):
        r = rng.random()
        if r < 0.34:
            qs.append(f"APPEND {rng.choice('WE')}")
            cur_len += 1
        elif r < 0.67:
            l = rng.randint(1, cur_len); rr = rng.randint(l, cur_len)
            qs.append(f"FLIP {l} {rr}")
        else:
            l = rng.randint(1, cur_len); rr = rng.randint(l, cur_len)
            qs.append(f"REVERSE {l} {rr}")
    W(p, [f"{n} {q}", s] + qs)


def g_noip04p1(p, big, _):
    W(p, [290, 230, 280, 200, 300, 170, 340, 50, 90, 80, 200, 60])


def g_noip10p1(p, big, _):
    rng = random.Random(3075)
    m, n = (100, 1000) if big else (20, 100)
    W(p, [f"{m} {n}", J(rng.randint(0, 1000) for _ in range(n))])


def g_noip19p2(p, big, _):
    rng = random.Random(3076)
    n = 500_000 if big else 1000
    s = "".join(rng.choice("()") for _ in range(n))
    pars = [rng.randint(max(1, i - 5), i - 1) for i in range(2, n + 1)]
    W(p, [n, s, J(pars)])


def g_noip19p6(p, big, _):
    rng = random.Random(3077)
    t = 5 if big else 2
    lines = [t]
    for _ in range(t):
        n = 100_000 if big else 300
        lines.append(str(n))
        lines += [f"{u} {v}" for u, v in tree_edges(rng, n, True, 0.25)]
    W(p, lines)


def g_noip22p1(p, big, _):
    rng = random.Random(3078)
    t = 5 if big else 2
    lines = [f"{t} 0"]
    for _ in range(t):
        n, m = (1000, 1000) if big else (40, 40)
        lines.append(f"{n} {m} 1 1")
        for _ in range(n):
            lines.append("".join("1" if rng.random() < 0.15 else "0" for _ in range(m)))
    W(p, lines)


def g_occ19g4(p, big, _):
    rng = random.Random(3079)
    n = 50_000 if big else 200
    lines = [n]
    for _ in range(n):
        x1, y1 = rng.randint(0, 10**6), rng.randint(0, 10**6)
        x2, y2 = x1 + rng.randint(1, 10**4), y1 + rng.randint(1, 10**4)
        lines.append(f"{x1} {y1} {x2} {y2}")
    W(p, lines)


def g_oly23practice5(p, big, _):
    rng = random.Random(3080)
    n = 300_000 if big else 1000
    W(p, [n, J(rng.randint(1, 50) for _ in range(n))])


def g_sac22cc2p3(p, big, _):
    rng = random.Random(3081)
    k, r = (18, 100) if big else (8, 10)
    n = (1 << k)
    lines = [f"{k} {r}"]
    for u in range(1, n):
        lines.append(f"{u} {2 * u} {2 * u + 1} {rng.randint(-1000, 1000)}")
    W(p, lines)


def g_sac22cc2p5(p, big, _):
    rng = random.Random(3082)
    n, q = (1_000_000, 500_000) if big else (1000, 1000)
    a = [rng.randint(-10**9, 10**9) for _ in range(n)]
    qs = []
    for _ in range(q):
        if rng.random() < 0.5:
            qs.append(f"1 {rng.randint(1, n)} {rng.randint(-10**9, 10**9)}")
        else:
            l = rng.randint(1, n); rr = rng.randint(l, n)
            qs.append(f"2 {l} {rr}")
    W(p, [f"{n} {q}", J(a)] + qs)


def g_sac22cc3p5(p, big, _):
    rng = random.Random(3083)
    n, m = (50_000, 1000) if big else (500, 50)
    # Problem guarantees distinct dictionary words
    seen = set()
    words = []
    while len(words) < n:
        w = letters(rng, rng.randint(4, 12), "abcdefghij")
        if w not in seen:
            seen.add(w)
            words.append(w)
    texts = [letters(rng, 200 if big else 50, "abcdefghij") for _ in range(m)]
    W(p, [f"{n} {m}"] + words + texts)


def g_shoi15p2(p, big, _):
    rng = random.Random(3084)
    n, q = (200_000, 200_000) if big else (500, 500)
    qs = []
    for _ in range(q):
        k = rng.randint(0, 2)
        l = rng.randint(1, n); r = rng.randint(l, n)
        if k == 1:
            l2 = rng.randint(1, n); r2 = rng.randint(l2, n)
            qs.append(f"1 {l} {r} {l2} {r2}")
        else:
            qs.append(f"{k} {l} {r}")
    W(p, [f"{n} {q}"] + qs)


def g_tsoc15c2p6(p, big, _):
    rng = random.Random(3085)
    n, q = (30_000, 30_000) if big else (500, 500)
    h = [rng.randint(1, 10**9) for _ in range(n)]
    qs = []
    for _ in range(q):
        l = rng.randint(1, n); r = rng.randint(l, n)
        qs.append(f"{l} {r} {rng.randint(1, 10**5)}")
    W(p, [f"{n} {q}", J(h)] + qs)


def g_ucrpc21e(p, big, _):
    rng = random.Random(3086)
    n, m = (100_000, 100_000) if big else (500, 500)
    def mk(cnt):
        out = []
        for _ in range(cnt):
            l = rng.randint(0, 10**6)
            r = l + rng.randint(1, 1000)
            out.append(f"{l} {r} {rng.randint(1, 3)}")
        return out
    W(p, [f"10000000 {n} {m}"] + mk(n) + mk(m))


def g_yac1p2(p, big, _):
    rng = random.Random(3087)
    n = 200_000 if big else 500
    s = "".join(rng.choice("BW") for _ in range(n))
    es = [f"{u} {v}" for u, v in tree_edges(rng, n, True, 0.25)]
    W(p, [n, s] + es)


def g_yac6p5(p, big, _):
    rng = random.Random(3088)
    n = 200_000 if big else 500
    lines = [n] + [f"{rng.randint(-10**9, 10**9)} {rng.randint(-10**9, 10**9)} {rng.randint(0, 3)}" for _ in range(n)]
    W(p, lines)


def g_yac6p6(p, big, _):
    rng = random.Random(3089)
    n, m, q = (100, 100, 100_000) if big else (20, 20, 200)
    rows = [J(rng.randint(-10**5, 10**5) for _ in range(m)) for _ in range(n)]
    qs = []
    for _ in range(q):
        x1 = rng.randint(1, n); x2 = rng.randint(x1, n)
        y1 = rng.randint(1, m); y2 = rng.randint(y1, m)
        qs.append(f"{x1} {y1} {x2} {y2} {rng.randint(0, 2)}")
    W(p, [f"{n} {m} {q}"] + rows + qs)


def g_yac7p2(p, big, _):
    rng = random.Random(3090)
    t = 100 if big else 5
    lines = [t] + [f"{rng.randint(2, 100)} {rng.randint(2, 100)}" for _ in range(t)]
    W(p, lines)


SPECS += [
    ("aac4p2", P("aac4p2_orig.cpp"), "bsearch+lcm", "n=q=2e5", g_aac4p2, "exact"),
    ("scout_batch_32/today/aac5p3", O(4239862), "tree+bsearch", "n=m=2e5", g_aac5p3, "exact"),
    ("scout_batch_27/acc7p3", O(3540475), "kruskal+DSU", "n=m=1e5", g_acc7p3, "exact"),
    ("hillclimb_dp_graph/apio07p3", P("hillclimb_dp_graph/apio07p3_orig.cpp"), "ring bitmask DP", "n=1e4, c=5e4", g_apio07p3, "exact"),
    ("scout_batch_33/btoi16p1", O(5323961), "all-roots BFS", "n=5000", g_btoi16p1, "exact"),
    ("btoi17p3", P("btoi17p3_orig.cpp"), "tree LCA+diff", "n=1e5, m=5e4", g_btoi17p3, "exact"),
    ("scout_batch_38/today/btoi17p4", O(4523876), "bounded search", "n=2500", g_btoi17p4, "exact"),
    ("scout_batch_20/btoi17p5", O(4456190), "parity map", "k=1e5", g_btoi17p5, "exact"),
    ("scout_batch_27/btoi18p1", O(4456169), "functional graph", "n=1e5", g_btoi18p1, "exact"),
    ("scout_batch_38/today/btoi18p4", O(4656863), "binlift greedy", "n=m=1e5", g_btoi18p4, "exact"),
    ("scout_batch_36/today/ccc02s4hard", O(4157744), "monotone DP", "n=m=1e5", g_ccc02s4hard, "exact"),
    ("scout_batch_3/ccc03s4", O(4182950), "suffix/distinct", "t=10, |s|=5000", g_ccc03s4, "exact"),
    ("scout_batch_19/today/ccc04s5", O(4806811), "grid DP", "10x 100x100", g_ccc04s5, "exact"),
    ("scout_batch_21/ccc14s4", O(5672281), "coord compress 2D", "n=1000", g_ccc14s4, "exact"),
    ("scout_batch_47/today/ccc18s2", O(4329456), "grid rotate", "n=100", g_ccc18s2, "exact"),
    ("ccc19s5", P("ccc19s5_orig.cpp"), "triangle RMQ", "n=3000, k=1500", g_ccc19s5, "exact"),
    ("scout_batch_51/today/ccc20s1", O(4323572), "sort+scan", "n=1e5", g_ccc20s1, "float"),
    ("scout_batch_47/today/ccc20s2hard", O(4324075), "grid BFS", "1500x1500", g_ccc20s2hard, "exact"),
    ("scout_batch_47/today/ccc22s2", O(4371343), "hashmap", "x=y=g=1e5", g_ccc22s2, "exact"),
    ("scout_batch_51/today/ccc23s1", O(5350804), "grid scan", "n=2e5", g_ccc23s1, "exact"),
    ("scout_batch_51/today/ccc23s2", O(5350805), "palindrome scan", "n=5000", g_ccc23s2, "exact"),
    ("scout_batch_51/today/ccc96s3", O(5515585), "backtrack", "C(18,4)+...", g_ccc96s3, "exact"),
    ("scout_batch_51/today/cco03p4", O(5024536), "bitmask DP", "n=15", g_cco03p4, "exact"),
    ("scout_batch_47/today/cco04p1", O(5430016), "freq count", "n=1e5", g_cco04p1, "exact"),
    ("scout_batch_33/cco05p6", O(5150823), "window stack", "n=q=1e4", g_cco05p6, "exact"),
    ("scout_batch_7/cco17p3", O(4843963), "2D trie", "n=q=2e5", g_cco17p3, "exact"),
    ("scout_batch_36/today/cco18p6", O(4575245), "DnC sort", "n=4096", g_cco18p6, "exact"),
    ("scout_batch_36/today/cco19p4", O(3617807), "CHT/convex DP", "n=5e5", g_cco19p4, "float"),
    ("scout_batch_33/cco20p4", O(4801875), "tournament DFS", "n=1000", g_cco20p4, "exact"),
    ("scout_batch_36/today/cco20p5", O(4562195), "segtree+offline", "q=2e5", g_cco20p5, "exact"),
    ("scout_batch_30/cco21p5", O(4554642), "lazy segtree", "n=m=2e5", g_cco21p5, "exact"),
    ("scout_batch_36/today/cco22p4", O(4801884), "tree DP mod", "n=m=2000", g_cco22p4, "exact"),
    ("scout_batch_36/today/cco22p5", O(4801885), "two-graph MST", "n=1e5, m=1.5e5", g_cco22p5, "exact"),
    ("scout_batch_47/today/cco23p5", O(5580402), "tree walk", "n=2e5, k=3", g_cco23p5, "exact"),
    ("scout_batch_47/today/cco24p2", O(6435481), "greedy+queues", "n=1e6", g_cco24p2, "exact"),
    ("scout_batch_47/today/cco24p5", O(6435483), "lazy segtree DP", "n=5e5", g_cco24p5, "exact"),
    ("scout_batch_48/today/ceoi19pp2", O(5343197), "ordered set", "q=5e5", g_ceoi19pp2, "exact"),
    ("scout_batch_32/today/ceoi22p5", O(4733784), "segtree", "n=m=2e5", g_ceoi22p5, "exact"),
    ("scout_batch_35/today/coci21c1p1", O(4080824), "sim", "n=1e5", g_coci21c1p1, "exact"),
    ("scout_batch_35/today/coci21c1p3", O(4080960), "unicyclic DP", "n=1e5", g_coci21c1p3, "exact"),
    ("scout_batch_35/today/coci21c2p2", O(4143417), "DSU", "n=m=1000, q=1e5", g_coci21c2p2, "exact"),
    ("scout_batch_35/today/coci21c3p2", O(4196157), "tree diameter", "n=1e5", g_coci21c3p2, "exact"),
    ("scout_batch_35/today/coci21c6p1", O(4529987), "sort/rank", "n=1000", g_coci21c6p1, "exact"),
    ("scout_batch_19/today/coci21c6p3", O(4530006), "toposort", "n=m=2e5", g_coci21c6p3, "exact"),
    ("hillclimb_dp_graph/coi06p2", P("hillclimb_dp_graph/coi06p2_orig.cpp"), "biconnected+LCA", "n=1e5, m=2.5e5, q=2e5", g_coi06p2, "exact"),
    ("scout_batch_32/today/dmopc20c4p4", O(3485716), "stack greedy", "n=1e6", g_dmopc20c4p4, "exact"),
    ("scout_batch_26/dmopc20c7p4", O(3686586), "HLD/LCA+BIT", "n=q=2e5", g_dmopc20c7p4, "exact"),
    ("scout_batch_15/dmopc20c7p5", O(4370353), "max flow", "n=500, m=2000", g_dmopc20c7p5, "exact"),
    ("scout_batch_34/today/dmopc21c2p2", O(3946515), "perm compose", "n=300, q=3e5", g_dmopc21c2p2, "exact"),
    ("scout_batch_44/today/dmopc21c5p3", O(4218397), "sieve+game", "t=100, n<=1e4", g_dmopc21c5p3, "exact"),
    ("scout_batch_19/today/dmopc21c5p4", O(4223525), "sieve+const", "t=200, n<=5000", g_dmopc21c5p4, "exact"),
    ("scout_batch_44/today/dmopc21c9p3", O(4564027), "APSP BFS", "n=m=3000, q=1e5", g_dmopc21c9p3, "exact"),
    ("scout_batch_39/today/dmpg18s5", O(3476383), "sqrt decomp", "n=q=2e5", g_dmpg18s5, "exact"),
    ("scout_batch_23/today/dpi", O(4131648), "prob DP", "n=2999", g_dpi, "float"),
    ("scout_batch_28/dwite10c3p3", O(4804312), "bitmask DP", "n<=30", g_dwite10c3p3, "exact"),
    ("scout_batch_48/today/egoi22p5", O(5203693), "bsearch+fenwick", "n=q=1e5", g_egoi22p5, "exact"),
    ("scout_batch_48/today/egoi23p1", O(5678049), "hashmap+offset", "n=q=3e5", g_egoi23p1, "exact"),
    ("scout_batch_38/today/egoi23p7", O(5678057), "DAG topo", "n=m1=m2=2e5", g_egoi23p7, "exact"),
    ("scout_batch_43/today/etsr23r1p3", O(5494303), "string DP", "t=100, n=5000", g_etsr23r1p3, "exact"),
    ("scout_batch_43/today/etsr23r1p4", O(5495163), "segtree beats", "n=q=2e5", g_etsr23r1p4, "exact"),
    ("scout_batch_43/today/etsr23r2p4", O(5494510), "greedy+sort", "t=20, n=1e4", g_etsr23r2p4, "exact"),
    ("scout_batch_43/today/etsr23r4p1", O(5521420), "array scan", "n=3e5", g_etsr23r4p1, "exact"),
    ("scout_batch_43/today/etsr23r4p3", O(5635626), "tree+edges", "n=m=1e5", g_etsr23r4p3, "exact"),
    ("scout_batch_7/ioi01p1", O(3477640), "2D fenwick", "1024x1024, q=1e5", g_ioi01p1, "exact"),
    ("scout_batch_32/today/ioi04p2hard", O(4182973), "geom/sort", "n=2e5", g_ioi04p2hard, "exact"),
    ("scout_batch_18/today/ioi05p2", O(5032753), "dynamic segtree", "n=1e9, q=1e5", g_ioi05p2, "exact"),
    ("scout_batch_50/today/noi03p4", O(5032732), "tree diameter", "n=2e5", g_noi03p4, "exact"),
    ("scout_batch_37/today/noi14p5", O(3601452), "grid greedy", "5000x5000, q=5e4", g_noi14p5, "exact"),
    ("scout_batch_27/noi15p1", O(3775616), "DSU+compress", "t=10, q=1e5", g_noi15p1, "exact"),
    ("scout_batch_2/noi15p2", O(3775038), "HLD+lazy segtree", "n=q=1e5", g_noi15p2, "exact"),
    ("scout_batch_50/today/noi17p1", O(5690633), "bitset/carry", "q=3e5", g_noi17p1, "exact"),
    ("scout_batch_18/today/noi18p1", O(6512702), "kruskal rebuild", "t=2, n=1e5, m=q=2e5", g_noi18p1, "exact"),
    ("scout_batch_52/today/noi21p5", O(3927923), "treap+mat", "n=q=1e5", g_noi21p5, "exact"),
    ("noip04p1", P("noip04p1_orig.cpp"), "sim", "12 months", g_noip04p1, "exact"),
    ("scout_batch_32/today/noip10p1", O(5445654), "queue sim", "m=100, n=1000", g_noip10p1, "exact"),
    ("blind_critic_3/noip19p2", O(5127828), "tree bracket DP", "n=5e5", g_noip19p2, "exact"),
    ("scout_batch_26/noip19p6", O(5148136), "tree centroid", "t=5, n=1e5", g_noip19p6, "exact"),
    ("blind_critic_3/noip22p1", O(5213639), "grid counting", "t=5, 1000x1000", g_noip22p1, "exact"),
    ("scout_batch_42/today/occ19g4", O(3578469), "sweep+segtree", "n=5e4", g_occ19g4, "exact"),
    ("scout_batch_41/today/oly23practice5", O(6415646), "alternating scan", "n=3e5", g_oly23practice5, "exact"),
    ("scout_batch_46/today/sac22cc2p3", O(4121049), "binary tree DP", "k=18", g_sac22cc2p3, "exact"),
    ("scout_batch_46/today/sac22cc2p5", O(4121099), "parity fenwick", "n=1e6, q=5e5", g_sac22cc2p5, "exact"),
    ("scout_batch_10/sac22cc3p5", O(4340958), "trie", "n=5e4, m=1000", g_sac22cc3p5, "exact"),
    ("scout_batch_45/today/shoi15p2", O(5450923), "lazy segtree", "n=q=2e5", g_shoi15p2, "exact"),
    ("scout_batch_7/tsoc15c2p6", O(3657404), "lazy segtree", "n=q=3e4", g_tsoc15c2p6, "exact"),
    ("scout_batch_32/today/ucrpc21e", O(3944571), "sweep line", "n=m=1e5", g_ucrpc21e, "exact"),
    ("scout_batch_46/today/yac1p2", O(4376253), "tree DP", "n=2e5", g_yac1p2, "exact"),
    ("scout_batch_46/today/yac6p5", O(5149823), "median sweep", "n=2e5", g_yac6p5, "exact"),
    ("scout_batch_39/today/yac6p6", O(5150025), "3-diag MITM DP", "100x100, q=1e5", g_yac6p6, "exact"),
    ("scout_batch_46/today/yac7p2", O(5705152), "grid snake", "t=100", g_yac7p2, "exact"),
]
